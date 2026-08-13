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

#[test]
fn subtraction_is_left_associative() {
    let result = eval(&parse("5 - 2 - 1").expect("parse")).expect("eval");
    assert_eq!(result, Value::float(2.0));
}

#[test]
fn division_is_left_associative() {
    let result = eval(&parse("8 / 4 / 2").expect("parse")).expect("eval");
    assert_eq!(result, Value::float(1.0));
}

#[test]
fn parentheses_group_expressions() {
    let result = eval(&parse("2 * (3 + 4)").expect("parse")).expect("eval");
    assert_eq!(result, Value::float(14.0));
}

#[test]
fn unary_minus_negates_a_number() {
    let result = eval(&parse("-2 + 5").expect("parse")).expect("eval");
    assert_eq!(result, Value::float(3.0));
}

#[test]
fn division_by_zero_is_an_error() {
    let result = eval(&parse("1 / 0").expect("parse"));
    assert!(result.is_err());
}

#[test]
fn exponentiation_is_right_associative() {
    let result = eval(&parse("2 ^ 3 ^ 2").expect("parse")).expect("eval");
    assert_eq!(result, Value::float(512.0));
}
