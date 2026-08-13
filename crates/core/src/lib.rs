//! calc-core — the single source of truth for calc's logic.
//!
//! Compiles to both `wasm32-unknown-unknown` (web/PWA/desktop) and native targets
//! (CLI/TUI). Stays pure: no I/O, no threads, no platform calls. Numerics per
//! ADR-0005.

use std::collections::HashMap;

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

/// Variable bindings available while evaluating an [`Expression`].
#[derive(Debug, Clone, Default)]
pub struct Env {
    bindings: HashMap<String, Value>,
}

impl Env {
    /// Look up a name.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name)
    }

    /// Bind a name to a value.
    pub fn set(&mut self, name: impl Into<String>, value: Value) {
        self.bindings.insert(name.into(), value);
    }
}

/// A parsed piece of mathematics that can be evaluated to a [`Value`] — a domain
/// noun (see `CONTEXT.md`). Public so it can be produced by multiple input
/// forms (plain text, LaTeX) and consumed by both [`eval`] and the graphing
/// Sampler; treated opaquely by tests.
#[derive(Debug, Clone)]
pub enum Expression {
    Literal(f64),
    Var(String),
    Call(String, Vec<Expression>),
    Neg(Box<Expression>),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Pow(Box<Expression>, Box<Expression>),
}

/// Errors crossing the calc-core seams.
#[derive(Debug, thiserror::Error)]
pub enum CalcError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("type error: {0}")]
    Type(String),
    #[error("unknown name: {0}")]
    UnknownName(String),
    #[error("domain error: {0}")]
    Domain(String),
    #[error("division by zero")]
    ZeroDivision,
}

/// Parse plain text into an [`Expression`] (the plain-text input seam).
///
/// Tokenizer + recursive-descent parser with precedence (additive below
/// multiplicative) and left-associative operator folding.
pub fn parse(text: &str) -> Result<Expression, CalcError> {
    let tokens = tokenize(text)?;
    let mut parser = Parser { tokens, pos: 0 };
    let expr = parser.parse_expression()?;
    if parser.peek().is_some() {
        return Err(CalcError::Parse("unexpected trailing input".into()));
    }
    Ok(expr)
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Comma,
    LParen,
    RParen,
}

fn tokenize(text: &str) -> Result<Vec<Token>, CalcError> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                chars.next();
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Slash);
                chars.next();
            }
            '^' => {
                tokens.push(Token::Caret);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            c if c.is_ascii_digit() || c == '.' => {
                let mut num = String::new();
                while let Some(&c2) = chars.peek() {
                    if c2.is_ascii_digit() || c2 == '.' {
                        num.push(c2);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let n: f64 = num
                    .parse()
                    .map_err(|_| CalcError::Parse(format!("invalid number: {num:?}")))?;
                tokens.push(Token::Number(n));
            }
            c if c.is_alphabetic() => {
                let mut ident = String::new();
                while let Some(&c2) = chars.peek() {
                    if c2.is_alphabetic() || c2 == '_' {
                        ident.push(c2);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(ident));
            }
            other => return Err(CalcError::Parse(format!("unexpected character: {other:?}"))),
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    /// Additive level: `+` and `-`, folded left-associatively.
    fn parse_expression(&mut self) -> Result<Expression, CalcError> {
        let mut left = self.parse_term()?;
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.next();
                    let right = self.parse_term()?;
                    left = Expression::Add(Box::new(left), Box::new(right));
                }
                Some(Token::Minus) => {
                    self.next();
                    let right = self.parse_term()?;
                    left = Expression::Sub(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// Multiplicative level: `*` and `/`, folded left-associatively.
    fn parse_term(&mut self) -> Result<Expression, CalcError> {
        let mut left = self.parse_pow()?;
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.next();
                    let right = self.parse_pow()?;
                    left = Expression::Mul(Box::new(left), Box::new(right));
                }
                Some(Token::Slash) => {
                    self.next();
                    let right = self.parse_pow()?;
                    left = Expression::Div(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// Power level: `^`, right-associative, binds tighter than `*` and `/`.
    fn parse_pow(&mut self) -> Result<Expression, CalcError> {
        let base = self.parse_factor()?;
        if matches!(self.peek(), Some(Token::Caret)) {
            self.next();
            let exponent = self.parse_pow()?;
            Ok(Expression::Pow(Box::new(base), Box::new(exponent)))
        } else {
            Ok(base)
        }
    }

    fn parse_factor(&mut self) -> Result<Expression, CalcError> {
        match self.next() {
            Some(Token::Number(n)) => Ok(Expression::Literal(n)),
            Some(Token::Ident(name)) => {
                if matches!(self.peek(), Some(Token::LParen)) {
                    self.next(); // consume '(' — call syntax
                    let mut args = Vec::new();
                    if matches!(self.peek(), Some(Token::RParen)) {
                        self.next(); // zero-argument call
                    } else {
                        loop {
                            let arg = self.parse_expression()?;
                            args.push(arg);
                            match self.next() {
                                Some(Token::Comma) => continue,
                                Some(Token::RParen) => break,
                                Some(other) => {
                                    return Err(CalcError::Parse(format!(
                                        "expected ',' or ')', found {other:?}"
                                    )));
                                }
                                None => {
                                    return Err(CalcError::Parse(
                                        "unexpected end of input".into(),
                                    ));
                                }
                            }
                        }
                    }
                    Ok(Expression::Call(name, args))
                } else {
                    Ok(Expression::Var(name))
                }
            }
            Some(Token::Minus) => {
                let inner = self.parse_factor()?;
                Ok(Expression::Neg(Box::new(inner)))
            }
            Some(Token::LParen) => {
                let expr = self.parse_expression()?;
                match self.next() {
                    Some(Token::RParen) => Ok(expr),
                    Some(other) => {
                        Err(CalcError::Parse(format!("expected ')', found {other:?}")))
                    }
                    None => Err(CalcError::Parse("unexpected end of input".into())),
                }
            }
            Some(other) => Err(CalcError::Parse(format!("expected a number, found {other:?}"))),
            None => Err(CalcError::Parse("unexpected end of input".into())),
        }
    }
}

/// Evaluate an [`Expression`] to a [`Value`] against an [`Env`] (the evaluation
/// seam).
pub fn eval(expr: &Expression, env: &Env) -> Result<Value, CalcError> {
    match expr {
        Expression::Literal(n) => Ok(Value::float(*n)),
        Expression::Var(name) => env
            .get(name)
            .cloned()
            .or_else(|| builtin_const(name))
            .ok_or_else(|| CalcError::UnknownName(name.clone())),
        Expression::Neg(inner) => match eval(inner, env)? {
            Value::Float(n) => Ok(Value::Float(-n)),
            other => Err(CalcError::Type(format!("cannot negate {other:?}"))),
        },
        Expression::Add(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, |a, b| a + b),
        Expression::Sub(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, |a, b| a - b),
        Expression::Mul(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, |a, b| a * b),
        Expression::Div(lhs, rhs) => {
            let l = eval(lhs, env)?;
            let r = eval(rhs, env)?;
            if let Value::Float(b) = r {
                if b == 0.0 {
                    return Err(CalcError::ZeroDivision);
                }
            }
            binop(l, r, |a, b| a / b)
        }
        Expression::Pow(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, |a, b| a.powf(b)),
        Expression::Call(name, args) => {
            let mut values = Vec::with_capacity(args.len());
            for arg in args {
                values.push(eval(arg, env)?);
            }
            call_builtin(name, values)
        }
    }
}

/// Built-in constants (π, e), resolved when a name isn't in the environment.
fn builtin_const(name: &str) -> Option<Value> {
    match name {
        "pi" => Some(Value::float(std::f64::consts::PI)),
        "e" => Some(Value::float(std::f64::consts::E)),
        _ => None,
    }
}

/// Dispatch a builtin function call. User-defined functions (L2) will share
/// this seam later.
fn call_builtin(name: &str, args: Vec<Value>) -> Result<Value, CalcError> {
    match name {
        "sqrt" => {
            let [x] = args.as_slice() else {
                return Err(CalcError::Type(format!(
                    "sqrt expects 1 argument, got {}",
                    args.len()
                )));
            };
            match x {
                Value::Float(n) => {
                    if *n < 0.0 {
                        Err(CalcError::Domain(format!("sqrt of negative number {n}")))
                    } else {
                        Ok(Value::Float(n.sqrt()))
                    }
                }
                other => Err(CalcError::Type(format!(
                    "sqrt expects a number, got {other:?}"
                ))),
            }
        }
        _ => Err(CalcError::UnknownName(name.to_string())),
    }
}

/// Apply a float binary op to two [`Value`]s. Only the default `Float` path is
/// supported so far; other variants error until a test asks for them.
fn binop(lhs: Value, rhs: Value, op: impl Fn(f64, f64) -> f64) -> Result<Value, CalcError> {
    match (&lhs, &rhs) {
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(op(*a, *b))),
        _ => Err(CalcError::Type(format!("cannot combine {:?} and {:?}", lhs, rhs))),
    }
}
