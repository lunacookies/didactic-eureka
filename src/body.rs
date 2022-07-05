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
    pub(crate) local_defs: Arena<LocalDef>,
}

#[derive(Debug)]
pub struct Block(pub Vec<Stmt>);

#[derive(Debug, Clone, Copy)]
pub enum Stmt {
    Let(Id<LocalDef>),
    Expr(Id<Expr>),
}

#[derive(Debug, Clone)]
pub struct LocalDef {
    pub val: Id<Expr>,
    pub ty: ast::Ty,
}

#[derive(Debug)]
pub enum Expr {
    IntLiteral(u32),
    StringLiteral(String),
    CharLiteral(String),
    Local(Id<LocalDef>),
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
    locals: HashMap<String, Id<LocalDef>>,
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
                let (val, ty) = self.lower_expr(val)?;
                let id = self.body_db.local_defs.alloc(LocalDef { val, ty });
                self.locals.insert(name.clone(), id);
                Ok(Stmt::Let(id))
            }
            ast::Stmt::Expr(e) => {
                let (e, _) = self.lower_expr(e)?;
                Ok(Stmt::Expr(e))
            }
        }
    }

    fn lower_expr(&mut self, ast: &ast::Expr) -> Result<(Id<Expr>, ast::Ty), Error> {
        let (e, ty) = match &ast.kind {
            ast::ExprKind::IntLiteral(n) => {
                (Expr::IntLiteral(*n), ast::Ty::Named("int".to_string()))
            }
            ast::ExprKind::StringLiteral(s) => {
                (Expr::StringLiteral(s.clone()), ast::Ty::Named("string".to_string()))
            }
            ast::ExprKind::CharLiteral(c) => {
                (Expr::CharLiteral(c.clone()), ast::Ty::Named("char".to_string()))
            }
            ast::ExprKind::Local(name) => match self.locals.get(name) {
                Some(id) => {
                    let ty = self.body_db.local_defs[*id].ty.clone();
                    (Expr::Local(*id), ty)
                }
                None => {
                    return Err(Error {
                        message: format!("undefined variable `{name}`"),
                        range: ast.range.clone(),
                    })
                }
            },
            ast::ExprKind::Call { name, args } => {
                let (params, return_ty) = match self.index.get(name) {
                    Some(Item::Function { params, return_ty }) => {
                        let expected_arity = params.len();
                        let actual_arity = args.len();
                        if expected_arity != actual_arity {
                            return Err(Error {
                                message: format!("expected {expected_arity} arguments to `{name}`, got {actual_arity}"),
                                range:    ast.range.clone()
                            });
                        }
                        (params, return_ty)
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
                };

                let mut lowered_args = Vec::new();

                for (arg, (_, expected_ty)) in args.iter().zip(params) {
                    let (arg, actual_ty) = self.lower_expr(arg)?;
                    self.unify_tys(
                        expected_ty.clone(),
                        actual_ty,
                        self.body_db.expr_ranges[&arg].clone(),
                    )?;
                    lowered_args.push(arg);
                }

                (Expr::Call { name: name.clone(), args: lowered_args }, return_ty.clone())
            }
            ast::ExprKind::Binary { lhs, rhs, op } => {
                let (lhs, _) = self.lower_expr(lhs)?;
                let (rhs, _) = self.lower_expr(rhs)?;
                let ty = match op {
                    ast::BinaryOp::Add
                    | ast::BinaryOp::Sub
                    | ast::BinaryOp::Mul
                    | ast::BinaryOp::Div => ast::Ty::Named("int".to_string()),
                    ast::BinaryOp::Assign
                    | ast::BinaryOp::AddAssign
                    | ast::BinaryOp::SubAssign
                    | ast::BinaryOp::MulAssign
                    | ast::BinaryOp::DivAssign => ast::Ty::Void,
                    ast::BinaryOp::Eq
                    | ast::BinaryOp::NEq
                    | ast::BinaryOp::And
                    | ast::BinaryOp::Or
                    | ast::BinaryOp::Lt
                    | ast::BinaryOp::Gt
                    | ast::BinaryOp::LtEq
                    | ast::BinaryOp::GtEq => ast::Ty::Named("bool".to_string()),
                };

                (Expr::Binary { lhs, rhs, op: *op }, ty)
            }
            ast::ExprKind::Prefix { expr, op } => {
                let (expr, ty) = self.lower_expr(expr)?;
                let ty = match op {
                    ast::PrefixOp::Neg => {
                        self.unify_tys(
                            ast::Ty::Named("int".to_string()),
                            ty,
                            self.body_db.expr_ranges[&expr].clone(),
                        )?;
                        ast::Ty::Named("int".to_string())
                    }
                    ast::PrefixOp::Deref => match ty {
                        ast::Ty::Pointer(inner) => *inner,
                        _ => {
                            return Err(Error {
                                message: format!(
                                    "cannot dereference non-pointer type, found {}",
                                    print_ty(ty)
                                ),
                                range: self.body_db.expr_ranges[&expr].clone(),
                            })
                        }
                    },
                    ast::PrefixOp::AddrOf => ast::Ty::Pointer(Box::new(ty)),
                };
                (Expr::Prefix { expr, op: *op }, ty)
            }
        };

        let id = self.body_db.exprs.alloc(e);
        self.body_db.expr_ranges.insert(id, ast.range.clone());

        Ok((id, ty))
    }

    fn unify_tys(
        &self,
        expected: ast::Ty,
        actual: ast::Ty,
        range: Range<usize>,
    ) -> Result<(), Error> {
        if expected == actual {
            return Ok(());
        }

        Err(Error {
            message: format!(
                "mismatched types: expected {}, found {}",
                print_ty(expected),
                print_ty(actual)
            ),
            range,
        })
    }
}

fn print_ty(ty: ast::Ty) -> String {
    match ty {
        ast::Ty::Void => "void".to_string(),
        ast::Ty::Named(s) => s,
        ast::Ty::Pointer(ty) => format!("*{}", print_ty(*ty)),
    }
}
