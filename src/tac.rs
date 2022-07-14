use std::fmt;

pub enum Instr {
	Const { dst: Register, val: u32 },
	Add { dst: Register, lhs: Register, rhs: Register },
}

#[derive(Clone, Copy)]
pub struct Register(pub u16);

impl fmt::Debug for Instr {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Instr::Const { dst, val } => {
				write!(f, "{dst:?} = #{val}")
			}
			Instr::Add { dst, lhs, rhs } => {
				write!(f, "{dst:?} = add {lhs:?} {rhs:?}")
			}
		}
	}
}

impl fmt::Debug for Register {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "r{}", self.0)
	}
}
