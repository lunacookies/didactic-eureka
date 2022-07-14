use crate::tac::{Instr, Register};
use std::fmt::{self, Write};

pub struct Cfg {
	pub bbs: Vec<BasicBlock>,
}

pub struct BasicBlock {
	pub arguments: Vec<Register>,
	pub instrs: Vec<Instr>,
	pub terminator: TerminatorInstr,
}

pub enum TerminatorInstr {
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
		for (i, bb) in self.bbs.iter().enumerate() {
			writeln!(f)?;
			let label = Label(i as u16);
			write!(f, "{}", bb.debug(label))?;
		}

		Ok(())
	}
}

impl fmt::Display for Cfg {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			r#"digraph {{
	node [fontname="Menlo,monospace", shape=box]"#
		)?;

		for (i, bb) in self.bbs.iter().enumerate() {
			write!(
				f,
				"\n\t{i} [label=\"{}\\l\"]",
				bb.debug(Label(i as u16)).replace('\n', "\\l")
			)?;
			match bb.terminator {
				TerminatorInstr::ConditionalBranch {
					true_branch,
					false_branch,
					..
				} => {
					write!(
						f,
						"\n\t{i} -> {} [label=\"true\"]",
						true_branch.0
					)?;
					write!(
						f,
						"\n\t{i} -> {} [label=\"false\"]",
						false_branch.0
					)?;
				}
				TerminatorInstr::Branch { label, .. } => {
					write!(f, "\n\t{i} -> {}", label.0)?
				}
				TerminatorInstr::Return(_) | TerminatorInstr::ReturnVoid => {}
			}
		}

		write!(f, "\n}}")?;

		Ok(())
	}
}

impl BasicBlock {
	fn debug(&self, label: Label) -> String {
		let mut s = String::new();

		write!(s, "{label:?}").unwrap();

		if !self.arguments.is_empty() {
			write!(s, "(").unwrap();
			for (i, arg) in self.arguments.iter().enumerate() {
				if i != 0 {
					write!(s, ", ").unwrap();
				}
				write!(s, "{arg:?}").unwrap();
			}
			write!(s, ")").unwrap();
		}

		writeln!(s, ":").unwrap();

		for instr in &self.instrs {
			writeln!(s, "  {instr:?}").unwrap();
		}

		write!(s, "  ").unwrap();
		match &self.terminator {
			TerminatorInstr::ConditionalBranch {
				condition,
				true_branch,
				false_branch,
			} => {
				write!(
					s,
					"cond_br {condition:?} {true_branch:?} {false_branch:?}"
				)
				.unwrap();
			}
			TerminatorInstr::Branch { label, arguments } => {
				write!(s, "br {label:?}").unwrap();
				if !arguments.is_empty() {
					write!(s, "(").unwrap();
					for (i, arg) in arguments.iter().enumerate() {
						if i != 0 {
							write!(s, ", ").unwrap();
						}
						write!(s, "{arg:?}").unwrap();
					}
					write!(s, ")").unwrap();
				}
			}
			TerminatorInstr::Return(reg) => write!(s, "ret {reg:?}").unwrap(),
			TerminatorInstr::ReturnVoid => write!(s, "ret").unwrap(),
		}

		s
	}
}

impl fmt::Debug for Label {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "l{}", self.0)
	}
}
