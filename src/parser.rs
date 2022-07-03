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

                let body = self.parse_block()?;

                Ok(ast::Item::Function { name, params, return_ty, body })
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

    fn parse_block(&mut self) -> ParseResult<ast::Block> {
        self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        while self.peek() != TokenKind::RBrace {
            stmts.push(self.parse_stmt()?);
        }
        self.bump(TokenKind::RBrace);

        Ok(ast::Block(stmts))
    }

    fn parse_stmt(&mut self) -> ParseResult<ast::Stmt> {
        self.expect(TokenKind::LetKw)?;
        let name = self.expect(TokenKind::Ident)?;
        self.expect(TokenKind::Eq)?;
        let val = self.parse_expr()?;

        Ok(ast::Stmt::Let { name, val })
    }

    fn parse_expr(&mut self) -> ParseResult<ast::Expr> {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> ParseResult<ast::Expr> {
        let mut lhs = self.parse_lhs()?;

        loop {
            let (op, num_tokens) = match (self.peek(), self.lookahead()) {
                (TokenKind::Eq, TokenKind::Eq) => (ast::BinaryOp::Eq, 2),
                (TokenKind::Bang, TokenKind::Eq) => (ast::BinaryOp::NEq, 2),
                (TokenKind::Lt, TokenKind::Eq) => (ast::BinaryOp::LtEq, 2),
                (TokenKind::Gt, TokenKind::Eq) => (ast::BinaryOp::GtEq, 2),
                (TokenKind::Lt, _) => (ast::BinaryOp::Lt, 1),
                (TokenKind::Gt, _) => (ast::BinaryOp::Gt, 1),
                (TokenKind::Eq, _) => (ast::BinaryOp::Assign, 1),
                (TokenKind::Plus, TokenKind::Eq) => (ast::BinaryOp::AddAssign, 2),
                (TokenKind::Dash, TokenKind::Eq) => (ast::BinaryOp::SubAssign, 2),
                (TokenKind::Star, TokenKind::Eq) => (ast::BinaryOp::MulAssign, 2),
                (TokenKind::Slash, TokenKind::Eq) => (ast::BinaryOp::DivAssign, 2),
                (TokenKind::Plus, _) => (ast::BinaryOp::Add, 1),
                (TokenKind::Dash, _) => (ast::BinaryOp::Sub, 1),
                (TokenKind::Star, _) => (ast::BinaryOp::Mul, 1),
                (TokenKind::Slash, _) => (ast::BinaryOp::Div, 1),
                (TokenKind::Pretzel, TokenKind::Pretzel) => (ast::BinaryOp::And, 2),
                (TokenKind::Pipe, TokenKind::Pipe) => (ast::BinaryOp::Or, 2),
                _ => break,
            };

            let bp = match op {
                ast::BinaryOp::Eq
                | ast::BinaryOp::NEq
                | ast::BinaryOp::Lt
                | ast::BinaryOp::Gt
                | ast::BinaryOp::LtEq
                | ast::BinaryOp::GtEq => 4,

                ast::BinaryOp::Assign
                | ast::BinaryOp::AddAssign
                | ast::BinaryOp::SubAssign
                | ast::BinaryOp::MulAssign
                | ast::BinaryOp::DivAssign => 1,

                ast::BinaryOp::Add | ast::BinaryOp::Sub => 5,
                ast::BinaryOp::Mul | ast::BinaryOp::Div => 6,

                ast::BinaryOp::And => 3,
                ast::BinaryOp::Or => 2,
            };

            if bp < min_bp {
                break;
            }

            for _ in 0..num_tokens {
                self.bump_any();
            }

            let rhs = self.parse_expr_bp(bp + 1)?;
            lhs = ast::Expr::Binary { lhs: Box::new(lhs), rhs: Box::new(rhs), op };
        }

        Ok(lhs)
    }

    fn parse_lhs(&mut self) -> ParseResult<ast::Expr> {
        match self.peek() {
            TokenKind::Number => {
                let text = self.bump(TokenKind::Number);
                Ok(ast::Expr::IntLiteral(text.parse().unwrap()))
            }
            TokenKind::String => {
                let text = self.bump(TokenKind::String);
                Ok(ast::Expr::StringLiteral(text[1..text.len() - 1].to_string()))
            }
            TokenKind::Char => {
                let text = self.bump(TokenKind::Char);
                Ok(ast::Expr::CharLiteral(text[1..text.len() - 1].to_string()))
            }
            TokenKind::LParen => {
                self.bump(TokenKind::LParen);
                let e = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(e)
            }
            TokenKind::Dash => {
                self.bump(TokenKind::Dash);
                let e = self.parse_lhs()?;
                Ok(ast::Expr::Prefix { expr: Box::new(e), op: ast::PrefixOp::Neg })
            }
            TokenKind::Star => {
                self.bump(TokenKind::Star);
                let e = self.parse_lhs()?;
                Ok(ast::Expr::Prefix { expr: Box::new(e), op: ast::PrefixOp::Deref })
            }
            TokenKind::Pretzel => {
                self.bump(TokenKind::Pretzel);
                let e = self.parse_lhs()?;
                Ok(ast::Expr::Prefix { expr: Box::new(e), op: ast::PrefixOp::AddrOf })
            }
            _ => Err(self.error("expression")),
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
        self.bump_any()
    }

    fn bump_any(&mut self) -> String {
        let text = self.tokens[self.token_idx].text;
        self.token_idx += 1;
        text.to_string()
    }

    fn peek(&self) -> TokenKind {
        self.get_kind(self.token_idx)
    }

    fn lookahead(&self) -> TokenKind {
        self.get_kind(self.token_idx + 1)
    }

    fn get_kind(&self, idx: usize) -> TokenKind {
        self.tokens.get(idx).map_or(TokenKind::Eof, |t| t.kind)
    }

    fn at_eof(&self) -> bool {
        self.token_idx == self.tokens.len()
    }
}
