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
                let mut params = Vec::new();
                while self.peek() != TokenKind::RParen {
                    let name = self.expect(TokenKind::Ident)?;
                    let ty = self.parse_ty()?;
                    params.push((name, ty));

                    if self.peek() != TokenKind::RParen {
                        self.expect(TokenKind::Comma)?;
                    }
                }
                self.expect(TokenKind::RParen)?;

                let return_ty = self.parse_ty()?;

                self.expect(TokenKind::LBrace)?;
                self.expect(TokenKind::RBrace)?;

                Ok(ast::Item::Function { name, params, return_ty })
            }

            TokenKind::StructKw => {
                self.bump(TokenKind::StructKw);

                let name = self.expect(TokenKind::Ident)?;

                self.expect(TokenKind::LBrace)?;
                let mut fields = Vec::new();
                while self.peek() != TokenKind::RBrace {
                    let name = self.expect(TokenKind::Ident)?;
                    let ty = self.parse_ty()?;
                    fields.push((name, ty));

                    if self.peek() != TokenKind::RBrace {
                        self.expect(TokenKind::Comma)?;
                    }
                }
                self.expect(TokenKind::RBrace)?;

                Ok(ast::Item::Struct { name, fields })
            }

            _ => Err(self.error("item")),
        }
    }

    fn parse_ty(&mut self) -> ParseResult<ast::Ty> {
        match self.peek() {
            TokenKind::VoidKw => {
                self.bump(TokenKind::VoidKw);
                Ok(ast::Ty::Void)
            }
            TokenKind::Ident => {
                let name = self.bump(TokenKind::Ident);
                Ok(ast::Ty::Named(name))
            }
            TokenKind::Star => {
                self.bump(TokenKind::Star);
                let ty = self.parse_ty()?;
                Ok(ast::Ty::Pointer(Box::new(ty)))
            }
            _ => Err(self.error("type")),
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
