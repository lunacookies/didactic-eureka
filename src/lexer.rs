use logos::Logos;
use std::fmt;

pub fn lex(text: &str) -> Vec<Token<'_>> {
	let mut tokens = Vec::new();
	let mut lexer = TokenKind::lexer(text);

	while let Some(kind) = lexer.next() {
		let token = Token { text: lexer.slice(), kind };
		tokens.push(token);
	}

	tokens
}

#[derive(Clone, Copy)]
pub struct Token<'a> {
	pub text: &'a str,
	pub kind: TokenKind,
}

impl<'a> fmt::Debug for Token<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?} {:?}", self.kind, self.text)
	}
}

#[derive(Debug, PartialEq, Clone, Copy, Logos)]
pub enum TokenKind {
	#[token("let")]
	LetKw,

	#[regex("[a-z][a-z0-9]*")]
	Ident,

	#[regex("[0-9]+")]
	Number,

	#[token("=")]
	Equals,

	#[token("+")]
	Plus,

	#[error]
	#[regex("[ \t\n]+", logos::skip)]
	Error,
}
