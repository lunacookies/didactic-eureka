use crate::ast::{Expr, SourceFile, Statement};
use crate::lexer::{Token, TokenKind};

pub fn parse(tokens: &[Token<'_>]) -> SourceFile {
	let mut p = Parser { tokens, cursor: 0 };
	p.parse_source_file()
}

struct Parser<'a> {
	tokens: &'a [Token<'a>],
	cursor: usize,
}

impl Parser<'_> {
	fn parse_source_file(&mut self) -> SourceFile {
		let mut statements = Vec::new();

		while !self.at_eof() {
			statements.push(self.parse_statement());
		}

		SourceFile(statements)
	}

	fn parse_statement(&mut self) -> Statement {
		if self.peek() == TokenKind::LetKw {
			self.bump();
			let name = self.expect(TokenKind::Ident);
			self.expect(TokenKind::Equals);
			let val = self.parse_expr();
			return Statement::LocalDef { name, val };
		}

		Statement::Expr(self.parse_expr())
	}

	fn parse_expr(&mut self) -> Expr {
		self.parse_expr_bp(0)
	}

	fn parse_expr_bp(&mut self, min_bp: u8) -> Expr {
		let mut lhs = self.parse_lhs();

		loop {
			if self.at_eof() {
				break;
			}

			let op = match self.peek() {
				TokenKind::Plus => Op::Add,
				_ => break,
			};
			let bp = op.bp();

			if bp < min_bp {
				break;
			}

			self.bump();

			let rhs = self.parse_expr_bp(bp + 1);
			lhs = match op {
				Op::Add => {
					Expr::Add { lhs: Box::new(lhs), rhs: Box::new(rhs) }
				}
			};
		}

		lhs
	}

	fn parse_lhs(&mut self) -> Expr {
		match self.peek() {
			TokenKind::Number => {
				Expr::Number(self.expect(TokenKind::Number).parse().unwrap())
			}
			TokenKind::Ident => Expr::Variable(self.expect(TokenKind::Ident)),
			TokenKind::IfKw => {
				self.bump();
				let condition = self.parse_expr();
				let true_branch = self.parse_expr();
				self.expect(TokenKind::ElseKw);
				let false_branch = self.parse_expr();
				Expr::If {
					condition: Box::new(condition),
					true_branch: Box::new(true_branch),
					false_branch: Box::new(false_branch),
				}
			}
			_ => panic!("expected expression"),
		}
	}

	fn expect(&mut self, kind: TokenKind) -> String {
		if self.peek() != kind {
			panic!("expected {kind:?}");
		}

		let text = self.tokens[self.cursor].text.to_string();
		self.cursor += 1;
		text
	}

	fn at_eof(&self) -> bool {
		self.cursor == self.tokens.len()
	}

	fn bump(&mut self) {
		self.cursor += 1;
	}

	#[track_caller]
	fn peek(&self) -> TokenKind {
		self.tokens[self.cursor].kind
	}
}

#[derive(Clone, Copy)]
enum Op {
	Add,
}

impl Op {
	fn bp(self) -> u8 {
		match self {
			Op::Add => 1,
		}
	}
}
