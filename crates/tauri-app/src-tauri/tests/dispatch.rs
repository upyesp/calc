//! The unified-binary command surface: argument parsing → [`Action`] (pure,
//! no side effects). The modes themselves are thin wrappers over the
//! frontends' library entry points.

use app_lib::dispatch::{action_from, Action, Args};
use clap::error::ErrorKind;
use clap::Parser;

fn parse(args: &[&str]) -> Action {
    let parsed = Args::try_parse_from(std::iter::once("epher").chain(args.iter().copied()))
        .expect("args should parse");
    action_from(&parsed)
}

#[test]
fn bare_epher_opens_the_gui() {
    assert_eq!(parse(&[]), Action::Gui);
}

#[test]
fn gui_subcommand_is_explicit_gui() {
    assert_eq!(parse(&["gui"]), Action::Gui);
}

#[test]
fn expression_is_one_shot() {
    assert_eq!(parse(&["2 + 2"]), Action::OneShot("2 + 2".to_string()));
}

#[test]
fn dash_reads_stdin() {
    assert_eq!(parse(&["-"]), Action::Stdin);
}

#[test]
fn negative_numbers_are_expressions_not_flags() {
    assert_eq!(parse(&["-5"]), Action::OneShot("-5".to_string()));
}

#[test]
fn subcommands_select_their_modes() {
    assert_eq!(parse(&["repl"]), Action::Repl);
    assert_eq!(parse(&["tui"]), Action::Tui);
}

#[test]
fn help_and_version_still_work_despite_hyphen_values() {
    let help = Args::try_parse_from(["epher", "--help"]).unwrap_err();
    assert_eq!(help.kind(), ErrorKind::DisplayHelp);
    let version = Args::try_parse_from(["epher", "--version"]).unwrap_err();
    assert_eq!(version.kind(), ErrorKind::DisplayVersion);
}

#[test]
fn repl_after_an_expression_is_rejected_not_silently_merged() {
    // `epher "1+1" repl` must not quietly pick one meaning: the positional
    // expression and the subcommands are mutually exclusive (an error that
    // goes to stderr, unlike --help/--version which go to stdout).
    let err = Args::try_parse_from(["epher", "1 + 1", "repl"]).unwrap_err();
    assert!(err.use_stderr(), "unexpected kind: {:?}", err.kind());
}
