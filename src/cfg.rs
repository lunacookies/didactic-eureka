use crate::tac::{Instr, Register};
use std::fmt;

pub struct Cfg {
	pub blocks: Vec<BasicBlock>,
}

pub struct BasicBlock {
	pub arguments: Vec<Register>,
	pub instrs: Vec<Instr>,
	pub tail: BasicBlockTail,
}

pub enum BasicBlockTail {
	ConditionalBranch {
		condition: Register,
		true_branch: Label,
		false_branch: Label,
	},
	Branch {
		label: Label,
		arguments: Vec<Register>,
	},
	Return(Register),
	ReturnVoid,
}

#[derive(Clone, Copy)]
pub struct Label(pub u16);

impl fmt::Debug for Cfg {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for (i, bb) in self.blocks.iter().enumerate() {
			writeln!(f)?;
			let label = Label(i as u16);
			bb.debug(label, f)?;
		}

		Ok(())
	}
}

impl BasicBlock {
	fn debug(&self, label: Label, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{label:?}")?;

		if !self.arguments.is_empty() {
			write!(f, "(")?;
			for (i, arg) in self.arguments.iter().enumerate() {
				if i != 0 {
					write!(f, ", ")?;
				}
				write!(f, "{arg:?}")?;
			}
			write!(f, ")")?;
		}

		writeln!(f, ":")?;

		for instr in &self.instrs {
			writeln!(f, "  {instr:?}")?;
		}

		write!(f, "  ")?;
		match &self.tail {
			BasicBlockTail::ConditionalBranch {
				condition,
				true_branch,
				false_branch,
			} => {
				write!(
					f,
					"cond_br {condition:?} {true_branch:?} {false_branch:?}"
				)?;
			}
			BasicBlockTail::Branch { label, arguments } => {
				write!(f, "br {label:?}")?;
				if !arguments.is_empty() {
					write!(f, "(")?;
					for (i, arg) in arguments.iter().enumerate() {
						if i != 0 {
							write!(f, ", ")?;
						}
						write!(f, "{arg:?}")?;
					}
					write!(f, ")")?;
				}
			}
			BasicBlockTail::Return(reg) => write!(f, "ret {reg:?}")?,
			BasicBlockTail::ReturnVoid => write!(f, "ret")?,
		}

		Ok(())
	}
}

impl fmt::Debug for Label {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "l{}", self.0)
	}
}
