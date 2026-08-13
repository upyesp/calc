use calc_core::{eval, parse, Value};

#[test]
fn literal_number_evaluates_to_float_value() {
    let expr = parse("2").expect("parsing a number literal should succeed");
    let result = eval(&expr).expect("evaluating a literal should succeed");
    assert_eq!(result, Value::float(2.0));
}

#[test]
fn addition_of_two_numbers() {
    let result = eval(&parse("2 + 3").expect("parse")).expect("eval");
    assert_eq!(result, Value::float(5.0));
}

#[test]
fn multiplication_of_two_numbers() {
    let result = eval(&parse("2 * 3").expect("parse")).expect("eval");
    assert_eq!(result, Value::float(6.0));
}

#[test]
fn multiplication_binds_tighter_than_addition() {
    let result = eval(&parse("2 + 3 * 4").expect("parse")).expect("eval");
    assert_eq!(result, Value::float(14.0));
}
