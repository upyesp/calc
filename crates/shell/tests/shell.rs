//! epher-shell policy tests: classification, preparation (validation and
//! source resolution), and the native persist path.

use epher_core::Session;
use epher_i18n::Localizer;
use epher_shell::{classify, plain, prepare, run_command, Command};
use epher_store::{DocStore, MemoryStore};

fn en() -> Localizer {
    Localizer::resolve(Some("en"), &[])
}

#[test]
fn classifies_the_three_commands() {
    assert_eq!(
        classify("save fib"),
        Some(Command::SaveFunction { name: "fib".into() })
    );
    assert_eq!(
        classify("  save script   count  "),
        Some(Command::SaveScript { name: "count".into() })
    );
    // "save script" must win over the shorter "save " prefix
    assert_eq!(
        classify("save script foo"),
        Some(Command::SaveScript { name: "foo".into() })
    );
    assert_eq!(
        classify("language fr"),
        Some(Command::Language { code: "fr".into() })
    );
}

#[test]
fn non_command_lines_are_none() {
    assert_eq!(classify("1 + 1"), None);
    assert_eq!(classify("saving"), None);
    assert_eq!(classify("save"), None);
    assert_eq!(classify("language"), None);
    assert_eq!(classify("def f(x) = x"), None);
    assert_eq!(classify(""), None);
}

#[test]
fn prepare_resolves_function_source_from_the_session() {
    let mut s = Session::new();
    s.submit("def f(x) = x ^ 2");
    let p = prepare(&Command::SaveFunction { name: "f".into() }, &s, &en()).unwrap();
    assert_eq!(
        p,
        epher_shell::Prepared::SaveFunction { name: "f".into(), source: "def f(x) = x ^ 2".into() }
    );
}

#[test]
fn prepare_reports_missing_definition() {
    let s = Session::new();
    let err = plain(prepare(&Command::SaveFunction { name: "g".into() }, &s, &en()).unwrap_err());
    assert_eq!(err, "no definition for g in this session");
}

#[test]
fn prepare_uses_the_last_submitted_line_for_scripts() {
    let mut s = Session::new();
    s.submit("x = 0; while x < 5 do x = x + 1; x");
    let p = prepare(&Command::SaveScript { name: "count".into() }, &s, &en()).unwrap();
    assert!(matches!(p, epher_shell::Prepared::SaveScript { .. }));
}

#[test]
fn prepare_rejects_scripts_when_nothing_qualifies() {
    let s = Session::new();
    let err = plain(prepare(&Command::SaveScript { name: "x".into() }, &s, &en()).unwrap_err());
    assert_eq!(err, "nothing to save (no preceding script line)");

    let mut s = Session::new();
    s.submit("save fib"); // a previous command line is not a script
    let err = plain(prepare(&Command::SaveScript { name: "x".into() }, &s, &en()).unwrap_err());
    assert_eq!(err, "nothing to save (no preceding script line)");
}

#[test]
fn prepare_validates_language_codes() {
    let s = Session::new();
    let p = prepare(&Command::Language { code: "fr".into() }, &s, &en()).unwrap();
    assert_eq!(p, epher_shell::Prepared::Language { code: "fr".into() });

    let err = plain(prepare(&Command::Language { code: "xx".into() }, &s, &en()).unwrap_err());
    assert!(err.starts_with("unsupported language xx"));
}

#[test]
fn run_command_persists_a_function() {
    let mut s = Session::new();
    s.submit("def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)");
    let store = DocStore::new(MemoryStore::default());
    let out = plain(run_command(
        &Command::SaveFunction { name: "fib".into() },
        &mut s,
        &store,
        &en(),
    )
    .message);
    assert_eq!(out, "saved fib");
    let docs = store.list_functions().unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].name, "fib");
    assert!(docs[0].source.starts_with("def fib(n)"));
}

#[test]
fn run_command_persists_language_and_answers() {
    let mut s = Session::new();
    let store = DocStore::new(MemoryStore::default());
    let handled = run_command(&Command::Language { code: "es".into() }, &mut s, &store, &en());
    let out = plain(handled.message);
    assert_eq!(handled.language, Some("es".into()));
    assert_eq!(out, "language set to es");
    assert_eq!(
        epher_store::persist::load_language(&store).unwrap(),
        Some("es".into())
    );
}

#[test]
fn run_command_surfaces_prepare_errors_without_persisting() {
    let mut s = Session::new();
    let store = DocStore::new(MemoryStore::default());
    let out = plain(
        run_command(&Command::SaveFunction { name: "nope".into() }, &mut s, &store, &en()).message,
    );
    assert_eq!(out, "no definition for nope in this session");
    assert!(store.list_functions().unwrap().is_empty());
}
