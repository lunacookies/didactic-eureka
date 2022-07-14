use crate::ast::{Expr, SourceFile, Statement};
use crate::cfg::{BasicBlock, BasicBlockTail, Cfg, Label};
use crate::tac::{Instr, Register};
use std::collections::HashMap;
use std::mem;

pub fn lower(source_file: &SourceFile) -> Cfg {
	let mut ctx = Ctx {
		arguments: Vec::new(),
		instrs: Vec::new(),
		cfg: Cfg { blocks: Vec::new() },
		name_map: HashMap::new(),
		current_register: Register(0),
	};
	ctx.lower_source_file(source_file);

	ctx.cfg
}

struct Ctx {
	arguments: Vec<Register>,
	instrs: Vec<Instr>,
	cfg: Cfg,
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
		self.finish_basic_block(BasicBlockTail::ReturnVoid);
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
				let lhs = self.lower_expr(lhs);
				let rhs = self.lower_expr(rhs);
				let dst = self.next_register();
				self.instrs.push(Instr::Add { dst, lhs, rhs });
				dst
			}

			Expr::If { condition, true_branch, false_branch } => {
				let current_label = self.current_basic_block_label();
				let true_branch_label = Label(current_label.0 + 1);
				let false_branch_label = Label(current_label.0 + 2);
				let join_label = Label(current_label.0 + 3);

				let condition = self.lower_expr(condition);

				self.finish_basic_block(BasicBlockTail::ConditionalBranch {
					condition,
					true_branch: true_branch_label,
					false_branch: false_branch_label,
				});

				let true_branch = self.lower_expr(true_branch);
				self.finish_basic_block(BasicBlockTail::Branch {
					label: join_label,
					arguments: vec![true_branch],
				});

				let false_branch = self.lower_expr(false_branch);
				self.finish_basic_block(BasicBlockTail::Branch {
					label: join_label,
					arguments: vec![false_branch],
				});

				let result = self.next_register();
				self.arguments.push(result);
				result
			}
		}
	}

	fn finish_basic_block(&mut self, tail: BasicBlockTail) {
		let arguments = mem::take(&mut self.arguments);
		let instrs = mem::take(&mut self.instrs);
		self.cfg.blocks.push(BasicBlock { arguments, instrs, tail });
	}

	fn next_register(&mut self) -> Register {
		let register = self.current_register;
		self.current_register.0 += 1;
		register
	}

	fn current_basic_block_label(&self) -> Label {
		Label(self.cfg.blocks.len() as u16)
	}
}
