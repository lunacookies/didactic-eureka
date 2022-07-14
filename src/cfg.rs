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
			write!(f, "{label:?}")?;

			if !bb.arguments.is_empty() {
				write!(f, "(")?;
				for (i, arg) in bb.arguments.iter().enumerate() {
					if i != 0 {
						write!(f, ", ")?;
					}
					write!(f, "{arg:?}")?;
				}
				write!(f, ")")?;
			}

			writeln!(f, ":")?;

			for instr in &bb.instrs {
				writeln!(f, "  {instr:?}")?;
			}

			write!(f, "  ")?;
			match &bb.tail {
				BasicBlockTail::ConditionalBranch {
					condition,
					true_branch,
					false_branch,
				} => {
					write!(
						f,
						"\x1b[35mcond_br\x1b[0m {condition:?} {true_branch:?} {false_branch:?}"
					)?;
				}
				BasicBlockTail::Branch { label, arguments } => {
					write!(f, "\x1b[35mbr\x1b[0m {label:?}")?;
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
				BasicBlockTail::Return(reg) => {
					write!(f, "\x1b[35mret\x1b[0m {reg:?}")?
				}
				BasicBlockTail::ReturnVoid => write!(f, "\x1b[35mret\x1b[0m")?,
			}
		}

		Ok(())
	}
}

impl fmt::Debug for Label {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b[33ml{}\x1b[0m", self.0)
	}
}
