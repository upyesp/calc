use calc_core::{eval, parse, Value};

#[test]
fn literal_number_evaluates_to_float_value() {
    let expr = parse("2").expect("parsing a number literal should succeed");
    let result = eval(&expr).expect("evaluating a literal should succeed");
    assert_eq!(result, Value::float(2.0));
}
