use crate::complex_ball::ComplexBall;
use crate::rpn::{RpnInstruction, RpnProgram};

/// Zero-copy RPN stack machine.
///
/// Executes a compiled `RpnProgram` against a slice of input variables.
/// The hot path (instruction loop) performs no heap allocations.
pub fn execute(program: &RpnProgram, inputs: &[ComplexBall]) -> Result<ComplexBall, ExecError> {
    if inputs.len() < program.arity {
        return Err(ExecError::MissingArguments {
            expected: program.arity,
            got: inputs.len(),
        });
    }

    // Pre-allocate a small stack (typical expression depth < 8)
    let mut stack = Vec::with_capacity(16);

    for instr in &program.instructions {
        match *instr {
            RpnInstruction::PushConstant(v) => {
                stack.push(ComplexBall::from_real(v));
            }
            RpnInstruction::LoadVariable(i) => {
                let v = inputs.get(i).ok_or(ExecError::VariableIndexOutOfBounds(i))?;
                stack.push(*v);
            }
            RpnInstruction::EvalEml => {
                let y = stack.pop().ok_or(ExecError::StackUnderflow)?;
                let x = stack.pop().ok_or(ExecError::StackUnderflow)?;
                let r = ComplexBall::eml(x, y).map_err(|_| ExecError::EmlEvalError)?;
                stack.push(r);
            }
        }
    }

    stack.pop().ok_or(ExecError::EmptyResult)
}

#[derive(Debug)]
pub enum ExecError {
    StackUnderflow,
    EmptyResult,
    EmlEvalError,
    MissingArguments { expected: usize, got: usize },
    VariableIndexOutOfBounds(usize),
}

impl core::fmt::Display for ExecError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::StackUnderflow => write!(f, "stack underflow"),
            Self::EmptyResult => write!(f, "stack empty after execution"),
            Self::EmlEvalError => write!(f, "EML evaluation error (singularity/overflow)"),
            Self::MissingArguments { expected, got } => {
                write!(f, "need {expected} arguments, got {got}")
            }
            Self::VariableIndexOutOfBounds(i) => {
                write!(f, "variable index {i} out of bounds")
            }
        }
    }
}

impl std::error::Error for ExecError {}
