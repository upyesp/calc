use std::io::Write;
use std::process::{Command, Stdio};

fn epher_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_epher-cli"))
}

/// Run a REPL session with piped stdin, returning its stdout.
fn repl_output(store_dir: &str, input: &str) -> String {
    let mut child = epher_bin()
        .env("EPHER_STORE_DIR", store_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn one_shot_evaluates_and_prints() {
    let out = epher_bin().arg("2 + 3 * 4").output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "14");
}

#[test]
fn one_shot_errors_on_bad_input() {
    let out = epher_bin().arg("2 +").output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("error"));
}

#[test]
fn repl_runs_scripts_and_keeps_state() {
    let dir = tempfile::tempdir().unwrap();
    let out = repl_output(dir.path().to_str().unwrap(), "x = 5; x + 1\ndef f(n) = n * 2\nf(x)\nquit\n");
    assert!(out.contains("6"), "stdout was: {out}");
    assert!(out.contains("10"), "stdout was: {out}");
    // the bare def produces no error line
    assert!(!out.contains("error"), "stdout was: {out}");
}

#[test]
fn repl_persists_functions_and_history_across_restarts() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    // session 1: define + save a function, evaluate, quit
    let out1 = repl_output(path, "def f(x) = x ^ 2\nsave f\nf(3)\nquit\n");
    assert!(out1.contains("saved f"), "stdout was: {out1}");
    assert!(out1.contains("= 9"), "stdout was: {out1}");

    // session 2: the saved function is loaded from the store
    let out2 = repl_output(path, "f(4)\nquit\n");
    assert!(out2.contains("= 16"), "stdout was: {out2}");

    // history persisted too (visible as the definition line on load? no —
    // history is display-only; check the store file exists)
    assert!(dir.path().join("function/f.json").exists());
    assert!(dir.path().join("setting/history.json").exists());
}

#[test]
fn repl_save_requires_a_definition_in_session() {
    let dir = tempfile::tempdir().unwrap();
    let out = repl_output(
        dir.path().to_str().unwrap(),
        "save nope\nquit\n",
    );
    assert!(out.contains("no definition for nope"), "stdout was: {out}");
}

#[test]
fn language_command_persists_the_setting() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    let out = repl_output(path, "language fr\nquit\n");
    assert!(out.contains("language set to fr"), "stdout was: {out}");
    // the preference is stored and reloaded on restart
    assert!(dir.path().join("setting/language.json").exists());
    let raw = std::fs::read_to_string(dir.path().join("setting/language.json")).unwrap();
    assert!(raw.contains("\"fr\""), "setting file was: {raw}");

    let out2 = repl_output(path, "quit\n");
    assert!(out2.contains("epher>"), "stdout was: {out2}");
}

#[test]
fn unsupported_language_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let out = repl_output(dir.path().to_str().unwrap(), "language xx\nquit\n");
    assert!(out.contains("unsupported language xx"), "stdout was: {out}");
}

#[test]
fn save_script_persists_and_reloads_the_last_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    // session 1: run a script, save it
    let out1 = repl_output(path, "x = 10; y = x + 5\nsave script setup\nquit\n");
    assert!(out1.contains("saved script setup"), "stdout was: {out1}");
    assert!(dir.path().join("script/setup.json").exists());

    // session 2: the saved script ran at startup (y is defined)
    let out2 = repl_output(path, "y\nquit\n");
    assert!(out2.contains("= 15"), "stdout was: {out2}");
}

#[test]
fn save_script_without_a_preceding_line_errors() {
    let dir = tempfile::tempdir().unwrap();
    let out = repl_output(dir.path().to_str().unwrap(), "save script empty\nquit\n");
    assert!(
        out.contains("nothing to save"),
        "stdout was: {out}"
    );
}
