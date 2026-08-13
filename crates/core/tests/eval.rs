use calc_core::{eval, parse, Env, Value};

/// Evaluate source text with an empty environment — the common case in these
/// tests (the CLI's future `evaluate(text)` convenience does the same).
fn eval_str(src: &str) -> Value {
    let env = Env::default();
    eval(&parse(src).expect("parse"), &env).expect("eval")
}

#[test]
fn literal_number_evaluates_to_float_value() {
    assert_eq!(eval_str("2"), Value::float(2.0));
}

#[test]
fn addition_of_two_numbers() {
    assert_eq!(eval_str("2 + 3"), Value::float(5.0));
}

#[test]
fn multiplication_of_two_numbers() {
    assert_eq!(eval_str("2 * 3"), Value::float(6.0));
}

#[test]
fn multiplication_binds_tighter_than_addition() {
    assert_eq!(eval_str("2 + 3 * 4"), Value::float(14.0));
}

#[test]
fn subtraction_is_left_associative() {
    assert_eq!(eval_str("5 - 2 - 1"), Value::float(2.0));
}

#[test]
fn division_is_left_associative() {
    assert_eq!(eval_str("8 / 4 / 2"), Value::float(1.0));
}

#[test]
fn parentheses_group_expressions() {
    assert_eq!(eval_str("2 * (3 + 4)"), Value::float(14.0));
}

#[test]
fn unary_minus_negates_a_number() {
    assert_eq!(eval_str("-2 + 5"), Value::float(3.0));
}

#[test]
fn division_by_zero_is_an_error() {
    assert!(eval(&parse("1 / 0").expect("parse"), &Env::default()).is_err());
}

#[test]
fn exponentiation_is_right_associative() {
    assert_eq!(eval_str("2 ^ 3 ^ 2"), Value::float(512.0));
}

#[test]
fn variable_resolves_from_environment() {
    let mut env = Env::default();
    env.set("x", Value::float(3.0));
    let result = eval(&parse("x + 2").expect("parse"), &env).expect("eval");
    assert_eq!(result, Value::float(5.0));
}

#[test]
fn unknown_variable_is_an_error() {
    assert!(eval(&parse("q").expect("parse"), &Env::default()).is_err());
}

#[test]
fn builtin_function_call() {
    assert_eq!(eval_str("sqrt(16)"), Value::float(4.0));
}

#[test]
fn function_call_args_are_expressions() {
    assert_eq!(eval_str("sqrt(9 + 7)"), Value::float(4.0));
}

#[test]
fn builtin_constants_pi_and_e() {
    assert_eq!(eval_str("pi"), Value::float(3.141592653589793));
    assert_eq!(eval_str("e"), Value::float(2.718281828459045));
}

#[test]
fn builtin_function_with_two_arguments() {
    assert_eq!(eval_str("min(2, 3)"), Value::float(2.0));
}

#[test]
fn unary_minus_binds_looser_than_power() {
    assert_eq!(eval_str("-2 ^ 2"), Value::float(-4.0));
    assert_eq!(eval_str("2 ^ -2"), Value::float(0.25));
}
