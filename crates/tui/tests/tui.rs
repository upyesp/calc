use calc_tui::App;

#[test]
fn submit_evaluates_against_persistent_env() {
    let mut app = App::default();
    app.set_input("x = 5; x + 1");
    app.submit();
    assert_eq!(app.result(), "= 6");
    app.set_input("x * 2");
    app.submit();
    assert_eq!(app.result(), "= 10");
    assert_eq!(app.history().len(), 2);
}

#[test]
fn errors_are_shown_not_crashing() {
    let mut app = App::default();
    app.set_input("1/0");
    app.submit();
    assert_eq!(app.result(), "error: division by zero");
}

#[test]
fn empty_input_does_nothing() {
    let mut app = App::default();
    app.submit();
    assert_eq!(app.result(), "");
    assert_eq!(app.history().len(), 0);
}
