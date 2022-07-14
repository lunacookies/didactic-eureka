use crate::ast::{Expr, SourceFile, Statement};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub struct Block(pub Vec<Instr>);

pub enum Instr {
	Const { dst: Register, val: u32 },
	Add { dst: Register, lhs: Register, rhs: Register },
}

#[derive(Clone, Copy)]
pub struct Register(u16);

pub fn lower(source_file: &SourceFile) -> Block {
	let mut ctx =
		Ctx { instrs: Vec::new(), name_map: HashMap::new(), current_register: Register(0) };
	ctx.lower_source_file(source_file);

	Block(ctx.instrs)
}

struct Ctx {
	instrs: Vec<Instr>,
	name_map: HashMap<String, Register>,
	current_register: Register,
}

impl Ctx {
	fn lower_source_file(&mut self, source_file: &SourceFile) {
		for statement in &source_file.0 {
			match statement {
				Statement::LocalDef { name, val } => {
					let val_register = self.lower_expr(val);
					self.name_map.insert(name.clone(), val_register);
				}
				Statement::Expr(e) => {
					self.lower_expr(e);
				}
			}
		}
	}

	fn lower_expr(&mut self, expr: &Expr) -> Register {
		match expr {
			Expr::Number(val) => {
				let dst = self.next_register();
				self.instrs.push(Instr::Const { dst, val: *val });
				dst
			}
			Expr::Variable(name) => self.name_map[name],
			Expr::Add { lhs, rhs } => {
				let lhs_reg = self.lower_expr(lhs);
				let rhs_reg = self.lower_expr(rhs);
				let dst = self.next_register();
				self.instrs.push(Instr::Add { dst, lhs: lhs_reg, rhs: rhs_reg });
				dst
			}
		}
	}

	fn next_register(&mut self) -> Register {
		let register = self.current_register;
		self.current_register.0 += 1;
		register
	}
}

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
