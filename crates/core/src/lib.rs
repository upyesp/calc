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
    Neg(Box<Expression>),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
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
    Plus,
    Minus,
    Star,
    Slash,
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
        let mut left = self.parse_factor()?;
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.next();
                    let right = self.parse_factor()?;
                    left = Expression::Mul(Box::new(left), Box::new(right));
                }
                Some(Token::Slash) => {
                    self.next();
                    let right = self.parse_factor()?;
                    left = Expression::Div(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expression, CalcError> {
        match self.next() {
            Some(Token::Number(n)) => Ok(Expression::Literal(n)),
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

/// Evaluate an [`Expression`] to a [`Value`] (the evaluation seam).
pub fn eval(expr: &Expression) -> Result<Value, CalcError> {
    match expr {
        Expression::Literal(n) => Ok(Value::float(*n)),
        Expression::Neg(inner) => match eval(inner)? {
            Value::Float(n) => Ok(Value::Float(-n)),
            other => Err(CalcError::Type(format!("cannot negate {other:?}"))),
        },
        Expression::Add(lhs, rhs) => binop(eval(lhs)?, eval(rhs)?, |a, b| a + b),
        Expression::Sub(lhs, rhs) => binop(eval(lhs)?, eval(rhs)?, |a, b| a - b),
        Expression::Mul(lhs, rhs) => binop(eval(lhs)?, eval(rhs)?, |a, b| a * b),
        Expression::Div(lhs, rhs) => binop(eval(lhs)?, eval(rhs)?, |a, b| a / b),
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
