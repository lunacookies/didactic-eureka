use crate::ast;
use crate::errors::Error;
use crate::index::{Index, Item};
use id_arena::{Arena, Id};
use std::collections::HashMap;
use std::ops::Range;

#[derive(Debug, Default)]
pub struct BodyDb {
    pub(crate) exprs: Arena<Expr>,
    pub(crate) expr_ranges: HashMap<Id<Expr>, Range<usize>>,
    pub(crate) variable_defs: Arena<VariableDef>,
}

#[derive(Debug)]
pub struct Block(pub Vec<Stmt>);

#[derive(Debug, Clone, Copy)]
pub enum Stmt {
    Let(Id<VariableDef>),
    Expr(Id<Expr>),
}

#[derive(Debug, Clone, Copy)]
pub struct VariableDef {
    pub val: Id<Expr>,
}

#[derive(Debug)]
pub enum Expr {
    IntLiteral(u32),
    StringLiteral(String),
    CharLiteral(String),
    Variable(Id<VariableDef>),
    Param { idx: usize },
    Call { name: String, args: Vec<Id<Expr>> },
    Binary { lhs: Id<Expr>, rhs: Id<Expr>, op: ast::BinaryOp },
    Prefix { expr: Id<Expr>, op: ast::PrefixOp },
}

pub fn lower(ast: &ast::Block, index: &Index) -> Result<(Block, BodyDb), Error> {
    let mut ctx = LowerCtx { body_db: BodyDb::default(), index, variables: HashMap::new() };
    let b = ctx.lower_block(ast)?;
    Ok((b, ctx.body_db))
}

#[derive(Debug)]
struct LowerCtx<'a> {
    body_db: BodyDb,
    index: &'a Index,
    variables: HashMap<String, Id<VariableDef>>,
}

impl LowerCtx<'_> {
    fn lower_block(&mut self, ast: &ast::Block) -> Result<Block, Error> {
        let mut stmts = Vec::new();

        for stmt in &ast.stmts {
            stmts.push(self.lower_stmt(stmt)?);
        }

        Ok(Block(stmts))
    }

    fn lower_stmt(&mut self, ast: &ast::Stmt) -> Result<Stmt, Error> {
        match ast {
            ast::Stmt::Let { name, val } => {
                let val = self.lower_expr(val)?;
                let id = self.body_db.variable_defs.alloc(VariableDef { val });
                self.variables.insert(name.clone(), id);
                Ok(Stmt::Let(id))
            }
            ast::Stmt::Expr(e) => Ok(Stmt::Expr(self.lower_expr(e)?)),
        }
    }

    fn lower_expr(&mut self, ast: &ast::Expr) -> Result<Id<Expr>, Error> {
        let e = match &ast.kind {
            ast::ExprKind::IntLiteral(n) => Expr::IntLiteral(*n),
            ast::ExprKind::StringLiteral(s) => Expr::StringLiteral(s.clone()),
            ast::ExprKind::CharLiteral(c) => Expr::CharLiteral(c.clone()),
            ast::ExprKind::Variable(name) => match self.variables.get(name) {
                Some(id) => Expr::Variable(*id),
                None => {
                    return Err(Error {
                        message: format!("undefined variable `{name}`"),
                        range: ast.range.clone(),
                    })
                }
            },
            ast::ExprKind::Call { name, args } => {
                match self.index.get(name) {
                    Some(Item::Function { params, .. }) => {
                        let expected_arity = params.len();
                        let actual_arity = args.len();
                        if expected_arity != actual_arity {
                            return Err(Error {
                                message: format!("expected {expected_arity} arguments to `{name}`, got {actual_arity}"),
                                range:ast.range.clone()
                            });
                        }
                    }
                    Some(Item::Struct { .. }) => {
                        return Err(Error {
                            message: format!("tried to call struct `{name}`"),
                            range: ast.range.clone(),
                        })
                    }
                    None => {
                        return Err(Error {
                            message: format!("undefined function `{name}`"),
                            range: ast.range.clone(),
                        })
                    }
                }

                let mut lowered_args = Vec::new();

                for arg in args {
                    lowered_args.push(self.lower_expr(arg)?);
                }

                Expr::Call { name: name.clone(), args: lowered_args }
            }
            ast::ExprKind::Binary { lhs, rhs, op } => {
                Expr::Binary { lhs: self.lower_expr(lhs)?, rhs: self.lower_expr(rhs)?, op: *op }
            }
            ast::ExprKind::Prefix { expr, op } => {
                Expr::Prefix { expr: self.lower_expr(expr)?, op: *op }
            }
        };

        let id = self.body_db.exprs.alloc(e);
        self.body_db.expr_ranges.insert(id, ast.range.clone());

        Ok(id)
    }
}
