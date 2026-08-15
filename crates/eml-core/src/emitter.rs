use std::fmt;

use thiserror::Error;

use crate::executor::execute;
use crate::rpn::{RpnInstruction, RpnProgram};

/// A symbolic EML expression tree node.
#[derive(Debug, Clone, PartialEq)]
pub enum EmlExpr {
    /// A constant value.
    Const(f64),
    /// The i-th input variable.
    Var(usize),
    /// `eml(left, right) = exp(left) - ln(right)`
    Eml(Box<EmlExpr>, Box<EmlExpr>),
}

impl fmt::Display for EmlExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmlExpr::Const(v) => write!(f, "{v}"),
            EmlExpr::Var(i) => write!(f, "x{i}"),
            EmlExpr::Eml(l, r) => write!(f, "eml({l}, {r})"),
        }
    }
}

impl EmlExpr {
    /// Convenience constructor: `eml(a, b)`
    pub fn eml(a: EmlExpr, b: EmlExpr) -> Self {
        Self::Eml(Box::new(a), Box::new(b))
    }

    /// Convenience constructor: constant `c`
    pub fn c(v: f64) -> Self {
        Self::Const(v)
    }

    /// Convenience constructor: variable `i`
    pub fn v(i: usize) -> Self {
        Self::Var(i)
    }

    fn max_var(&self) -> Option<usize> {
        match self {
            EmlExpr::Const(_) => None,
            EmlExpr::Var(i) => Some(*i),
            EmlExpr::Eml(left, right) => match (left.max_var(), right.max_var()) {
                (None, r) => r,
                (l, None) => l,
                (Some(a), Some(b)) => Some(a.max(b)),
            },
        }
    }

    fn emit_program(&self) -> RpnProgram {
        let mut instructions = Vec::new();
        let mut max_var = None;
        self.emit(&mut instructions, &mut max_var);
        let arity = max_var.map_or(0, |m| m + 1);
        RpnProgram::new(instructions, arity)
    }

    /// Fold a closed term to a real constant when [`execute`](crate::execute)
    /// returns a finite near-real. Intermediate `ln(0) = -inf` is allowed;
    /// a non-finite *final* value is not folded.
    pub fn fold(&self) -> Self {
        if self.max_var().is_none() {
            if let Ok(ball) = execute(&self.emit_program(), &[]) {
                if ball.center.im.abs() < 1e-10 {
                    return EmlExpr::Const(ball.center.re);
                }
            }
        }
        match self {
            EmlExpr::Const(v) => EmlExpr::Const(*v),
            EmlExpr::Var(i) => EmlExpr::Var(*i),
            EmlExpr::Eml(left, right) => EmlExpr::eml(left.fold(), right.fold()),
        }
    }

    /// Flatten the (folded) expression tree into a linear RPN program.
    ///
    /// Post-order traversal ensures operands are pushed before their operator.
    /// Variable indices are collected to compute arity.
    pub fn compile(&self) -> RpnProgram {
        self.fold().emit_program()
    }

    fn emit(&self, buf: &mut Vec<RpnInstruction>, max_var: &mut Option<usize>) {
        match self {
            EmlExpr::Const(v) => buf.push(RpnInstruction::PushConstant(*v)),
            EmlExpr::Var(i) => {
                *max_var = Some(max_var.map_or(*i, |m| m.max(*i)));
                buf.push(RpnInstruction::LoadVariable(*i));
            }
            EmlExpr::Eml(left, right) => {
                left.emit(buf, max_var);
                right.emit(buf, max_var);
                buf.push(RpnInstruction::EvalEml);
            }
        }
    }

    /// Parse the `Display` format: `eml(...)`, `xN`, or a float.
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let mut p = Parser {
            s: input.trim(),
            i: 0,
        };
        let expr = p.parse_expr()?;
        p.skip_ws();
        if p.i != p.s.len() {
            return Err(ParseError::TrailingJunk);
        }
        Ok(expr)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseError {
    #[error("unexpected end of EML expression")]
    UnexpectedEof,
    #[error("invalid EML token")]
    InvalidToken,
    #[error("trailing junk after EML expression")]
    TrailingJunk,
}

struct Parser<'a> {
    s: &'a str,
    i: usize,
}

impl Parser<'_> {
    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if !c.is_ascii_whitespace() {
                break;
            }
            self.i += c.len_utf8();
        }
    }

    fn peek(&self) -> Option<char> {
        self.s[self.i..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.i += c.len_utf8();
        Some(c)
    }

    fn expect(&mut self, want: char) -> Result<(), ParseError> {
        self.skip_ws();
        match self.bump() {
            Some(c) if c == want => Ok(()),
            Some(_) => Err(ParseError::InvalidToken),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    fn parse_expr(&mut self) -> Result<EmlExpr, ParseError> {
        self.skip_ws();
        match self.peek() {
            None => Err(ParseError::UnexpectedEof),
            Some('e') => self.parse_eml(),
            Some('x') => self.parse_var(),
            Some(c) if c == '-' || c == '+' || c.is_ascii_digit() || c == '.' => self.parse_number(),
            Some(_) => Err(ParseError::InvalidToken),
        }
    }

    fn parse_eml(&mut self) -> Result<EmlExpr, ParseError> {
        if !self.s[self.i..].starts_with("eml") {
            return Err(ParseError::InvalidToken);
        }
        self.i += 3;
        self.expect('(')?;
        let left = self.parse_expr()?;
        self.expect(',')?;
        let right = self.parse_expr()?;
        self.expect(')')?;
        Ok(EmlExpr::eml(left, right))
    }

    fn parse_var(&mut self) -> Result<EmlExpr, ParseError> {
        self.bump(); // 'x'
        let start = self.i;
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.bump();
        }
        if start == self.i {
            return Err(ParseError::InvalidToken);
        }
        let idx: usize = self.s[start..self.i]
            .parse()
            .map_err(|_| ParseError::InvalidToken)?;
        Ok(EmlExpr::v(idx))
    }

    fn parse_number(&mut self) -> Result<EmlExpr, ParseError> {
        let start = self.i;
        if matches!(self.peek(), Some('+') | Some('-')) {
            self.bump();
        }
        let mut saw_digit = false;
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            saw_digit = true;
            self.bump();
        }
        if self.peek() == Some('.') {
            self.bump();
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                saw_digit = true;
                self.bump();
            }
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            self.bump();
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.bump();
            }
            let exp_start = self.i;
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.bump();
            }
            if self.i == exp_start {
                return Err(ParseError::InvalidToken);
            }
        }
        if !saw_digit {
            return Err(ParseError::InvalidToken);
        }
        let v: f64 = self.s[start..self.i]
            .parse()
            .map_err(|_| ParseError::InvalidToken)?;
        Ok(EmlExpr::c(v))
    }
}
