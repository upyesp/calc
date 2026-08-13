//! calc-core — the single source of truth for calc's logic.
//!
//! Compiles to both `wasm32-unknown-unknown` (web/PWA/desktop) and native targets
//! (CLI/TUI). Stays pure: no I/O, no threads, no platform calls. Numerics per
//! ADR-0005.

use bigdecimal::BigDecimal;
use num_complex::Complex;
use num_rational::BigRational;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// The result of evaluating an Expression — the project's single number
/// representation (ADR-0005). `Float` is the default fast path; the other
/// variants are opt-in exactness layers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Float(f64),
    Rational(BigRational),
    Decimal(Decimal),
    Big(BigDecimal),
    Complex(Complex<f64>),
}

impl Value {
    /// Wrap a plain number as the default `Float` variant.
    pub fn float(n: f64) -> Self {
        Value::Float(n)
    }
}

/// A parsed piece of mathematics that can be evaluated to a [`Value`] — a domain
/// noun (see `CONTEXT.md`). Public so it can be produced by multiple input
/// forms (plain text, LaTeX) and consumed by both [`eval`] and the graphing
/// Sampler; treated opaquely by tests.
#[derive(Debug, Clone)]
pub enum Expression {
    Literal(f64),
    Add(Box<Expression>, Box<Expression>),
}

/// Errors crossing the calc-core seams.
#[derive(Debug, thiserror::Error)]
pub enum CalcError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("type error: {0}")]
    Type(String),
}

/// Parse plain text into an [`Expression`] (the plain-text input seam).
pub fn parse(text: &str) -> Result<Expression, CalcError> {
    let text = text.trim();
    if let Some((left, right)) = text.split_once('+') {
        let lhs = parse(left)?;
        let rhs = parse(right)?;
        Ok(Expression::Add(Box::new(lhs), Box::new(rhs)))
    } else {
        let n: f64 = text
            .parse()
            .map_err(|_| CalcError::Parse(format!("invalid number: {text:?}")))?;
        Ok(Expression::Literal(n))
    }
}

/// Evaluate an [`Expression`] to a [`Value`] (the evaluation seam).
pub fn eval(expr: &Expression) -> Result<Value, CalcError> {
    match expr {
        Expression::Literal(n) => Ok(Value::float(*n)),
        Expression::Add(lhs, rhs) => {
            let lhs = eval(lhs)?;
            let rhs = eval(rhs)?;
            add(lhs, rhs)
        }
    }
}

/// Add two [`Value`]s. Only the default `Float` path is supported so far; other
/// variants error until a test asks for them.
fn add(lhs: Value, rhs: Value) -> Result<Value, CalcError> {
    match (&lhs, &rhs) {
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(*a + *b)),
        _ => Err(CalcError::Type(format!("cannot add {:?} and {:?}", lhs, rhs))),
    }
}
