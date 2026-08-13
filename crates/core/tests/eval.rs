use calc_core::{
    eval, evaluate, parse, parse_latex, parse_script, run, sample, sample_parametric,
    sample_polar, Sample, Env, Session, Value,
};
use bigdecimal::BigDecimal;
use num_rational::BigRational;
use rust_decimal::Decimal;
use std::str::FromStr;

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

#[test]
fn assignment_then_use_in_next_statement() {
    let mut env = Env::default();
    let script = parse_script("x = 5; x + 1").expect("parse_script");
    let result = run(&script, &mut env).expect("run").expect("value");
    assert_eq!(result, Value::float(6.0));
    assert_eq!(env.get("x"), Some(&Value::float(5.0)));
}

#[test]
fn user_defined_function() {
    let mut env = Env::default();
    let script = parse_script("def f(x) = x ^ 2; f(3)").expect("parse_script");
    let result = run(&script, &mut env).expect("run").expect("value");
    assert_eq!(result, Value::float(9.0));
}

#[test]
fn comparison_operators_produce_booleans() {
    assert_eq!(eval_str("2 > 1"), Value::Bool(true));
    assert_eq!(eval_str("2 < 1"), Value::Bool(false));
    assert_eq!(eval_str("2 >= 2"), Value::Bool(true));
    assert_eq!(eval_str("2 <= 1"), Value::Bool(false));
    assert_eq!(eval_str("2 == 2"), Value::Bool(true));
    assert_eq!(eval_str("2 != 2"), Value::Bool(false));
}

#[test]
fn if_expression_picks_branch_by_condition() {
    assert_eq!(eval_str("if 2 > 1 then 10 else 20"), Value::float(10.0));
    assert_eq!(eval_str("if 2 < 1 then 10 else 20"), Value::float(20.0));
}

#[test]
fn user_function_recurses() {
    let mut env = Env::default();
    let script =
        parse_script("def fact(n) = if n <= 1 then 1 else n * fact(n - 1); fact(5)")
            .expect("parse_script");
    let result = run(&script, &mut env).expect("run").expect("value");
    assert_eq!(result, Value::float(120.0));
}

#[test]
fn evaluate_convenience_adapter() {
    assert_eq!(evaluate("2 + 3 * 4").expect("evaluate"), Value::float(14.0));
}

#[test]
fn exact_rational_arithmetic() {
    assert_eq!(
        eval_str("frac(1, 3) + frac(1, 3)"),
        Value::Rational(BigRational::new(2.into(), 3.into()))
    );
}

#[test]
fn float_promotes_to_rational_when_mixed() {
    assert_eq!(
        eval_str("frac(1, 3) * 3"),
        Value::Rational(BigRational::new(1.into(), 1.into()))
    );
}

#[test]
fn while_loop_repeats_until_condition_fails() {
    let mut env = Env::default();
    let script = parse_script("x = 0; while x < 3 do x = x + 1; x").expect("parse_script");
    let result = run(&script, &mut env).expect("run").expect("value");
    assert_eq!(result, Value::float(3.0));
}

#[test]
fn decimal_arithmetic_is_exact() {
    assert_eq!(
        eval_str("dec(0.1) + dec(0.2)"),
        Value::Decimal(Decimal::from_str("0.3").unwrap())
    );
}

#[test]
fn float_promotes_to_decimal_when_mixed() {
    assert_eq!(
        eval_str("dec(0.5) * 2"),
        Value::Decimal(Decimal::from_str("1.0").unwrap())
    );
}

#[test]
fn boolean_operators() {
    assert_eq!(eval_str("2 > 1 and 3 > 2"), Value::Bool(true));
    assert_eq!(eval_str("2 > 1 or 3 < 2"), Value::Bool(true));
    assert_eq!(eval_str("not 2 > 1"), Value::Bool(false));
    assert_eq!(eval_str("not (2 > 3)"), Value::Bool(true));
}

#[test]
fn runaway_loop_hits_the_step_limit() {
    let mut env = Env::default();
    let script = parse_script("x = 0; while x < 100001 do x = x + 1").expect("parse_script");
    assert!(run(&script, &mut env).is_err());
}

#[test]
fn big_layer_arbitrary_precision() {
    assert_eq!(
        eval_str("big(0.1) + big(0.2)"),
        Value::Big(BigDecimal::from_str("0.3").unwrap())
    );
}

#[test]
fn latex_input_parses() {
    let env = Env::default();
    let frac = parse_latex(r"\frac{1}{2} + \frac{1}{2}").expect("parse_latex");
    assert_eq!(eval(&frac, &env).expect("eval"), Value::float(1.0));
    let sqrt = parse_latex(r"\sqrt{16}").expect("parse_latex");
    assert_eq!(eval(&sqrt, &env).expect("eval"), Value::float(4.0));
    let nested = parse_latex(r"\frac{\frac{1}{2}}{2}").expect("parse_latex");
    assert_eq!(eval(&nested, &env).expect("eval"), Value::float(0.25));
}

#[test]
fn sampler_binds_x_and_evaluates() {
    let expr = parse("x ^ 2").expect("parse");
    let env = Env::default();
    let samples = sample(&expr, 0.0, 2.0, 3, &env).expect("sample");
    assert_eq!(
        samples,
        vec![
            Sample { x: 0.0, y: 0.0 },
            Sample { x: 1.0, y: 1.0 },
            Sample { x: 2.0, y: 4.0 },
        ]
    );
}

#[test]
fn sampler_skips_points_where_eval_errors() {
    let expr = parse("1 / x").expect("parse");
    let env = Env::default();
    let samples = sample(&expr, -1.0, 1.0, 3, &env).expect("sample");
    // x = -1, 0, 1 — the x = 0 point errors (division by zero) and is skipped
    assert_eq!(samples, vec![Sample { x: -1.0, y: -1.0 }, Sample { x: 1.0, y: 1.0 }]);
}

#[test]
fn parametric_sampler_binds_t() {
    let x = parse("t").expect("parse");
    let y = parse("t ^ 2").expect("parse");
    let samples = sample_parametric(&x, &y, 0.0, 2.0, 3, &Env::default()).expect("sample");
    assert_eq!(
        samples,
        vec![
            Sample { x: 0.0, y: 0.0 },
            Sample { x: 1.0, y: 1.0 },
            Sample { x: 2.0, y: 4.0 },
        ]
    );
}

#[test]
fn polar_sampler_converts_to_xy() {
    // r = 1 (unit circle): θ = 0 → (1, 0); θ = π/2 → (0, 1)
    let r = parse("1").expect("parse");
    let samples =
        sample_polar(&r, 0.0, std::f64::consts::FRAC_PI_2, 2, &Env::default()).expect("sample");
    assert_eq!(samples.len(), 2);
    assert!((samples[0].x - 1.0).abs() < 1e-12 && samples[0].y.abs() < 1e-12);
    assert!(samples[1].x.abs() < 1e-12 && (samples[1].y - 1.0).abs() < 1e-12);
}

#[test]
fn values_display_cleanly() {
    assert_eq!(Value::float(5.0).to_string(), "5");
    assert_eq!(Value::float(0.5).to_string(), "0.5");
    assert_eq!(Value::Bool(true).to_string(), "true");
    assert_eq!(
        eval_str("frac(1, 3)").to_string(),
        "1/3"
    );
    assert_eq!(
        eval_str("dec(0.1) + dec(0.2)").to_string(),
        "0.3"
    );
}

#[test]
fn session_submits_and_keeps_history() {
    let mut session = Session::new();
    assert_eq!(session.submit("x = 5; x + 1"), "= 6");
    assert_eq!(session.submit("x * 2"), "= 10");
    assert_eq!(session.history().len(), 2);
    assert_eq!(session.submit(""), "");
    assert_eq!(session.history().len(), 2);
}

#[test]
fn session_def_only_line_produces_no_output_and_records_source() {
    let mut session = Session::new();
    assert_eq!(session.submit("def f(x) = x ^ 2"), "");
    assert!(!session.history().iter().any(|h| h.contains("error")));
    assert_eq!(
        session.def_sources().get("f").map(String::as_str),
        Some("def f(x) = x ^ 2")
    );
}

#[test]
fn session_with_history_seeds_and_submit_appends() {
    let mut session = Session::with_history(vec!["old  = 1".to_string()]);
    assert_eq!(session.history().len(), 1);
    assert_eq!(session.submit("1 + 1"), "= 2");
    assert_eq!(session.history().len(), 2);
}

#[test]
fn session_tracks_last_submitted_line() {
    let mut session = Session::new();
    assert_eq!(session.last_line(), None);
    session.submit("x = 1; y = x + 1");
    assert_eq!(session.last_line(), Some("x = 1; y = x + 1"));
}
