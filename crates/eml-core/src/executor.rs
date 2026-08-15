use thiserror::Error;

use crate::complex_ball::{ComplexBall, EmlError};
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
                let r = ComplexBall::eml(x, y)?;
                stack.push(r);
            }
        }
    }

    let result = stack.pop().ok_or(ExecError::EmptyResult)?;
    if result.center.re.is_nan() || result.center.im.is_nan() {
        return Err(EmlError::NanResult.into());
    }
    if !result.is_finite() {
        return Err(EmlError::Overflow.into());
    }
    Ok(result)
}

#[derive(Debug, Error)]
pub enum ExecError {
    #[error("stack underflow")]
    StackUnderflow,
    #[error("stack empty after execution")]
    EmptyResult,
    #[error(transparent)]
    Eml(#[from] EmlError),
    #[error("need {expected} arguments, got {got}")]
    MissingArguments { expected: usize, got: usize },
    #[error("variable index {0} out of bounds")]
    VariableIndexOutOfBounds(usize),
}
