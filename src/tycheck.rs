use crate::ast::{Expr, SourceFile, Statement};
use crate::cfg::{BasicBlock, Cfg, Label, TerminatorInstr};
use crate::tac::{Instr, Register};
use std::collections::HashMap;

pub fn lower(source_file: &SourceFile) -> Cfg {
	let mut ctx = Ctx {
		current_basic_block_label: Label(0),
		cfg: Cfg { bbs: vec![BasicBlock::default()] },
		name_map: HashMap::new(),
		current_register: Register(0),
	};
	ctx.lower_source_file(source_file);

	ctx.cfg
}

struct Ctx {
	current_basic_block_label: Label,
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
	}

	fn lower_expr(&mut self, expr: &Expr) -> Register {
		match expr {
			Expr::Number(val) => {
				let dst = self.next_register();
				self.push_instr(Instr::Const { dst, val: *val });
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
				self.push_instr(Instr::Add { dst, lhs, rhs });
				dst
			}

			Expr::If { condition, true_branch, false_branch } => {
				let condition = self.lower_expr(condition);
				let true_branch_label = self.reserve_basic_block();
				let false_branch_label = self.reserve_basic_block();
				let join_label = self.reserve_basic_block();
				self.set_terminator(TerminatorInstr::ConditionalBranch {
					condition,
					true_branch: true_branch_label,
					false_branch: false_branch_label,
				});

				self.make_current(true_branch_label);
				let true_branch = self.lower_expr(true_branch);
				self.set_terminator(TerminatorInstr::Branch {
					label: join_label,
					arguments: vec![true_branch],
				});

				self.make_current(false_branch_label);
				let false_branch = self.lower_expr(false_branch);
				self.set_terminator(TerminatorInstr::Branch {
					label: join_label,
					arguments: vec![false_branch],
				});

				self.make_current(join_label);
				let result = self.next_register();
				self.push_argument(result);
				result
			}
		}
	}

	fn push_instr(&mut self, instr: Instr) {
		self.current_basic_block().instrs.push(instr);
	}

	fn push_argument(&mut self, arg: Register) {
		self.current_basic_block().arguments.push(arg);
	}

	fn set_terminator(&mut self, terminator: TerminatorInstr) {
		self.current_basic_block().terminator = terminator;
	}

	fn current_basic_block(&mut self) -> &mut BasicBlock {
		&mut self.cfg.bbs[self.current_basic_block_label.0 as usize]
	}

	fn reserve_basic_block(&mut self) -> Label {
		let i = self.cfg.bbs.len();
		self.cfg.bbs.push(BasicBlock::default());
		Label(i as u16)
	}

	fn make_current(&mut self, label: Label) {
		self.current_basic_block_label = label;
	}

	fn next_register(&mut self) -> Register {
		let register = self.current_register;
		self.current_register.0 += 1;
		register
	}
}
