use std::fmt;

use crate::rpn::{RpnInstruction, RpnProgram};

/// A symbolic EML expression tree node.
#[derive(Debug, Clone)]
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

    /// Flatten the expression tree into a linear RPN program.
    ///
    /// Post-order traversal ensures operands are pushed before their operator.
    /// Variable indices are collected to compute arity.
    pub fn compile(&self) -> RpnProgram {
        let mut instructions = Vec::new();
        let mut max_var = 0usize;
        self.emit(&mut instructions, &mut max_var);
        // arity = highest variable index + 1
        let arity = if max_var == 0 { 0 } else { max_var + 1 };
        RpnProgram::new(instructions, arity)
    }

    fn emit(&self, buf: &mut Vec<RpnInstruction>, max_var: &mut usize) {
        match self {
            EmlExpr::Const(v) => buf.push(RpnInstruction::PushConstant(*v)),
            EmlExpr::Var(i) => {
                *max_var = (*max_var).max(*i);
                buf.push(RpnInstruction::LoadVariable(*i));
            }
            EmlExpr::Eml(left, right) => {
                left.emit(buf, max_var);
                right.emit(buf, max_var);
                buf.push(RpnInstruction::EvalEml);
            }
        }
    }
}
