use crate::ast::{Expr, SourceFile, Statement};
use crate::cfg::BasicBlock;
use crate::tac::{Instr, Register};
use std::collections::HashMap;

pub fn lower(source_file: &SourceFile) -> BasicBlock {
	let mut ctx =
		Ctx { instrs: Vec::new(), name_map: HashMap::new(), current_register: Register(0) };
	ctx.lower_source_file(source_file);

	BasicBlock(ctx.instrs)
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
			Expr::Variable(name) => match self.name_map.get(name) {
				Some(reg) => *reg,
				None => panic!("undefined variable `{name}`"),
			},
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
