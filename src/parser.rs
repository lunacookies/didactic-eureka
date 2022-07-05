use crate::ast::{BinaryOp, Block, Expr, ExprKind, Function, Item, ItemKind, PrefixOp, Stmt, Ty};
use crate::errors::Error;
use crate::lexer::{Token, TokenKind};
use std::ops::Range;

pub fn parse(tokens: &[Token<'_>]) -> Result<Vec<Item>, Error> {
    let mut parser = Parser { tokens, token_idx: 0 };
    let mut items = Vec::new();

    while !parser.at_eof() {
        items.push(parser.parse_item()?);
    }

    Ok(items)
}

struct Parser<'a> {
    tokens: &'a [Token<'a>],
    token_idx: usize,
}

impl Parser<'_> {
    fn parse_item(&mut self) -> Result<Item, Error> {
        match self.peek() {
            TokenKind::FnKw => {
                let (_, fn_range) = self.bump(TokenKind::FnKw);

                let (name, _) = self.expect(TokenKind::Ident)?;

                self.expect(TokenKind::LParen)?;
                let mut params = Vec::new();
                while self.peek() != TokenKind::RParen {
                    let (name, _) = self.expect(TokenKind::Ident)?;
                    let ty = self.parse_ty()?;
                    params.push((name, ty));

                    if self.peek() != TokenKind::RParen {
                        self.expect(TokenKind::Comma)?;
                    }
                }
                self.expect(TokenKind::RParen)?;

                let return_ty = self.parse_ty()?;

                let body = self.parse_block()?;

                let range = fn_range.start..body.range.end;
                Ok(Item {
                    kind: ItemKind::Function(Function { name, params, return_ty, body }),
                    range,
                })
            }

            TokenKind::StructKw => {
                let (_, struct_range) = self.bump(TokenKind::StructKw);

                let (name, _) = self.expect(TokenKind::Ident)?;

                self.expect(TokenKind::LBrace)?;
                let mut fields = Vec::new();
                while self.peek() != TokenKind::RBrace {
                    let (name, _) = self.expect(TokenKind::Ident)?;
                    let ty = self.parse_ty()?;
                    fields.push((name, ty));

                    if self.peek() != TokenKind::RBrace {
                        self.expect(TokenKind::Comma)?;
                    }
                }
                let (_, r_brace_range) = self.expect(TokenKind::RBrace)?;

                Ok(Item {
                    kind: ItemKind::Struct { name, fields },
                    range: struct_range.start..r_brace_range.end,
                })
            }

            _ => Err(self.error("item")),
        }
    }

    fn parse_block(&mut self) -> Result<Block, Error> {
        let (_, l_brace_range) = self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        while self.peek() != TokenKind::RBrace {
            stmts.push(self.parse_stmt()?);
        }
        let (_, r_brace_range) = self.bump(TokenKind::RBrace);

        Ok(Block { stmts, range: l_brace_range.start..r_brace_range.end })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, Error> {
        if self.peek() == TokenKind::LetKw {
            self.expect(TokenKind::LetKw)?;
            let (name, _) = self.expect(TokenKind::Ident)?;
            self.expect(TokenKind::Eq)?;
            let val = self.parse_expr()?;
            self.expect(TokenKind::Semi)?;
            return Ok(Stmt::Let { name, val });
        }

        let e = self.parse_expr()?;
        self.expect(TokenKind::Semi)?;
        Ok(Stmt::Expr(e))
    }

    fn parse_expr(&mut self) -> Result<Expr, Error> {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expr, Error> {
        let mut lhs = self.parse_lhs()?;

        loop {
            let (op, num_tokens) = match (self.peek(), self.lookahead()) {
                (TokenKind::Eq, TokenKind::Eq) => (BinaryOp::Eq, 2),
                (TokenKind::Bang, TokenKind::Eq) => (BinaryOp::NEq, 2),
                (TokenKind::Lt, TokenKind::Eq) => (BinaryOp::LtEq, 2),
                (TokenKind::Gt, TokenKind::Eq) => (BinaryOp::GtEq, 2),
                (TokenKind::Lt, _) => (BinaryOp::Lt, 1),
                (TokenKind::Gt, _) => (BinaryOp::Gt, 1),
                (TokenKind::Eq, _) => (BinaryOp::Assign, 1),
                (TokenKind::Plus, TokenKind::Eq) => (BinaryOp::AddAssign, 2),
                (TokenKind::Dash, TokenKind::Eq) => (BinaryOp::SubAssign, 2),
                (TokenKind::Star, TokenKind::Eq) => (BinaryOp::MulAssign, 2),
                (TokenKind::Slash, TokenKind::Eq) => (BinaryOp::DivAssign, 2),
                (TokenKind::Plus, _) => (BinaryOp::Add, 1),
                (TokenKind::Dash, _) => (BinaryOp::Sub, 1),
                (TokenKind::Star, _) => (BinaryOp::Mul, 1),
                (TokenKind::Slash, _) => (BinaryOp::Div, 1),
                (TokenKind::Pretzel, TokenKind::Pretzel) => (BinaryOp::And, 2),
                (TokenKind::Pipe, TokenKind::Pipe) => (BinaryOp::Or, 2),
                _ => break,
            };

            let bp = match op {
                BinaryOp::Eq
                | BinaryOp::NEq
                | BinaryOp::Lt
                | BinaryOp::Gt
                | BinaryOp::LtEq
                | BinaryOp::GtEq => 4,

                BinaryOp::Assign
                | BinaryOp::AddAssign
                | BinaryOp::SubAssign
                | BinaryOp::MulAssign
                | BinaryOp::DivAssign => 1,

                BinaryOp::Add | BinaryOp::Sub => 5,
                BinaryOp::Mul | BinaryOp::Div => 6,

                BinaryOp::And => 3,
                BinaryOp::Or => 2,
            };

            if bp < min_bp {
                break;
            }

            for _ in 0..num_tokens {
                self.bump_any();
            }

            let rhs = self.parse_expr_bp(bp + 1)?;
            let range = lhs.range.start..rhs.range.end;
            lhs = Expr {
                kind: ExprKind::Binary { lhs: Box::new(lhs), rhs: Box::new(rhs), op },
                range,
            };
        }

        Ok(lhs)
    }

    fn parse_lhs(&mut self) -> Result<Expr, Error> {
        match self.peek() {
            TokenKind::Number => {
                let (text, range) = self.bump(TokenKind::Number);
                Ok(Expr { kind: ExprKind::IntLiteral(text.parse().unwrap()), range })
            }
            TokenKind::String => {
                let (text, range) = self.bump(TokenKind::String);
                Ok(Expr {
                    kind: ExprKind::StringLiteral(text[1..text.len() - 1].to_string()),
                    range,
                })
            }
            TokenKind::Char => {
                let (text, range) = self.bump(TokenKind::Char);
                Ok(Expr { kind: ExprKind::CharLiteral(text[1..text.len() - 1].to_string()), range })
            }
            TokenKind::Ident => {
                if self.lookahead() == TokenKind::LParen {
                    return self.parse_call();
                }

                let (text, range) = self.bump(TokenKind::Ident);
                Ok(Expr { kind: ExprKind::Variable(text), range })
            }
            TokenKind::LParen => {
                let (_, l_range) = self.bump(TokenKind::LParen);
                let e = self.parse_expr()?;
                let (_, r_range) = self.expect(TokenKind::RParen)?;
                Ok(Expr { kind: e.kind, range: l_range.start..r_range.end })
            }
            TokenKind::Dash => {
                let (_, dash_range) = self.bump(TokenKind::Dash);
                let e = self.parse_lhs()?;
                let range = dash_range.start..e.range.end;
                Ok(Expr { kind: ExprKind::Prefix { expr: Box::new(e), op: PrefixOp::Neg }, range })
            }
            TokenKind::Star => {
                let (_, star_range) = self.bump(TokenKind::Star);
                let e = self.parse_lhs()?;
                let range = star_range.start..e.range.end;
                Ok(Expr {
                    kind: ExprKind::Prefix { expr: Box::new(e), op: PrefixOp::Deref },
                    range,
                })
            }
            TokenKind::Pretzel => {
                let (_, pretzel_range) = self.bump(TokenKind::Pretzel);
                let e = self.parse_lhs()?;
                let range = pretzel_range.start..e.range.end;
                Ok(Expr {
                    kind: ExprKind::Prefix { expr: Box::new(e), op: PrefixOp::AddrOf },
                    range,
                })
            }
            _ => Err(self.error("expression")),
        }
    }

    fn parse_call(&mut self) -> Result<Expr, Error> {
        let (name, range) = self.bump(TokenKind::Ident);
        self.bump(TokenKind::LParen);

        let mut args = Vec::new();
        while self.peek() != TokenKind::RParen {
            args.push(self.parse_expr()?);
            if self.peek() != TokenKind::RParen {
                self.expect(TokenKind::Comma)?;
            }
        }

        let (_, r_paren_range) = self.expect(TokenKind::RParen)?;

        Ok(Expr { kind: ExprKind::Call { name, args }, range: range.start..r_paren_range.end })
    }

    fn parse_ty(&mut self) -> Result<Ty, Error> {
        match self.peek() {
            TokenKind::VoidKw => {
                self.bump(TokenKind::VoidKw);
                Ok(Ty::Void)
            }
            TokenKind::Ident => {
                let (name, _) = self.bump(TokenKind::Ident);
                Ok(Ty::Named(name))
            }
            TokenKind::Star => {
                self.bump(TokenKind::Star);
                let ty = self.parse_ty()?;
                Ok(Ty::Pointer(Box::new(ty)))
            }
            _ => Err(self.error("type")),
        }
    }

    fn expect(&mut self, expected: TokenKind) -> Result<(String, Range<usize>), Error> {
        let actual = self.peek();

        if expected == actual {
            return Ok(self.bump(expected));
        }

        Err(self.error(&format!("{expected:?}")))
    }

    fn error(&self, expected: &str) -> Error {
        let actual = self.peek();
        let range = match self.tokens.get(self.token_idx) {
            Some(tok) => tok.range.clone(),
            None => self.tokens[self.token_idx - 1].range.clone(),
        };

        Error { message: format!("expected {expected} but got {actual:?}"), range }
    }

    fn bump(&mut self, kind: TokenKind) -> (String, Range<usize>) {
        assert_eq!(self.peek(), kind);
        self.bump_any()
    }

    fn bump_any(&mut self) -> (String, Range<usize>) {
        let tok = &self.tokens[self.token_idx];
        self.token_idx += 1;
        (tok.text.to_string(), tok.range.clone())
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
