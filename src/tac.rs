use std::fmt;

#[derive(Debug)]
pub struct Block(pub Vec<Instr>);

pub enum Instr {
	Const { dst: Register, val: u32 },
	Add { dst: Register, lhs: Register, rhs: Register },
}

#[derive(Clone, Copy)]
pub struct Register(pub u16);

impl fmt::Debug for Instr {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Instr::Const { dst, val } => write!(f, "{dst:?} = \x1b[92m{val}\x1b[0m"),
			Instr::Add { dst, lhs, rhs } => {
				write!(f, "{dst:?} = \x1b[1;33madd\x1b[0m {lhs:?} {rhs:?}")
			}
		}
	}
}

impl fmt::Debug for Register {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b[36m%{}\x1b[0m", self.0)
	}
}
