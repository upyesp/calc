use std::io::Write;
use std::process::{Command, Stdio};

fn calc_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_calc"))
}

#[test]
fn one_shot_evaluates_and_prints() {
    let out = calc_bin().arg("2 + 3 * 4").output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "14");
}

#[test]
fn one_shot_errors_on_bad_input() {
    let out = calc_bin().arg("2 +").output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("error"));
}

#[test]
fn repl_runs_scripts_and_keeps_state() {
    let mut child = calc_bin()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"x = 5; x + 1\ndef f(n) = n * 2\nf(x)\nquit\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("6"), "stdout was: {stdout}");
    assert!(stdout.contains("10"), "stdout was: {stdout}");
}
