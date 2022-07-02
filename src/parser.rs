use crate::ast;
use crate::lexer::{Token, TokenKind};

pub fn parse(tokens: &[Token<'_>]) -> ParseResult<Vec<ast::Item>> {
    let mut parser = Parser { tokens, token_idx: 0 };
    let mut items = Vec::new();

    while !parser.at_eof() {
        items.push(parser.parse_item()?);
    }

    Ok(items)
}

type ParseResult<T> = Result<T, SyntaxError>;

#[derive(Debug)]
pub struct SyntaxError {
    message: String,
}

struct Parser<'a> {
    tokens: &'a [Token<'a>],
    token_idx: usize,
}

impl Parser<'_> {
    fn parse_item(&mut self) -> ParseResult<ast::Item> {
        match self.peek() {
            TokenKind::FnKw => {
                self.bump(TokenKind::FnKw);
                let name = self.expect(TokenKind::Ident)?;
                self.expect(TokenKind::LParen)?;
                self.expect(TokenKind::RParen)?;
                self.expect(TokenKind::LBrace)?;
                self.expect(TokenKind::RBrace)?;

                Ok(ast::Item::Function { name })
            }
            _ => Err(self.error("item")),
        }
    }

    fn expect(&mut self, expected: TokenKind) -> ParseResult<String> {
        let actual = self.peek();

        if expected == actual {
            return Ok(self.bump(expected));
        }

        Err(self.error(&format!("{expected:?}")))
    }

    fn error(&self, expected: &str) -> SyntaxError {
        let actual = self.peek();
        SyntaxError { message: format!("expected {expected} but got {actual:?}") }
    }

    fn bump(&mut self, kind: TokenKind) -> String {
        assert_eq!(self.peek(), kind);
        let text = self.tokens[self.token_idx].text;
        self.token_idx += 1;
        text.to_string()
    }

    fn peek(&self) -> TokenKind {
        if self.token_idx >= self.tokens.len() {
            return TokenKind::Eof;
        }

        self.tokens[self.token_idx].kind
    }

    fn at_eof(&self) -> bool {
        self.token_idx == self.tokens.len()
    }
}
