use std::fmt;

/// A single RPN instruction for the EML stack machine.
///
/// The instruction set is deliberately minimal — just constants,
/// variable loads, and the single EML operator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RpnInstruction {
    /// Push a constant floating-point value onto the stack.
    PushConstant(f64),
    /// Load the i-th input variable.
    LoadVariable(usize),
    /// Evaluate `eml(x, y) = exp(x) - ln(y)`.
    /// Pops y first, then x, pushes result.
    EvalEml,
}

impl fmt::Display for RpnInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RpnInstruction::PushConstant(v) => write!(f, "PUSH {v}"),
            RpnInstruction::LoadVariable(i) => write!(f, "LOAD x{i}"),
            RpnInstruction::EvalEml => write!(f, "EML"),
        }
    }
}

/// A compiled RPN program: flat, zero-allocation on hot path.
#[derive(Debug, Clone)]
pub struct RpnProgram {
    pub instructions: Vec<RpnInstruction>,
    /// Expected number of input variables.
    pub arity: usize,
}

impl RpnProgram {
    pub fn new(instructions: Vec<RpnInstruction>, arity: usize) -> Self {
        Self {
            instructions,
            arity,
        }
    }
}

impl fmt::Display for RpnProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "RPN  (arity={}, {} instr):",
            self.arity,
            self.instructions.len()
        )?;
        for (i, instr) in self.instructions.iter().enumerate() {
            writeln!(f, "  {i:03}:  {instr}")?;
        }
        Ok(())
    }
}
