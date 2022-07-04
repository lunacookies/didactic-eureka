use crate::ast;
use crate::errors::Error;
use std::collections::HashMap;

#[derive(Debug)]
pub struct LowerCtx {
    variables: HashMap<String, VariableId>,
    current_variable_id: VariableId,
    let_stmts: HashMap<VariableId, Expr>,
}

impl Default for LowerCtx {
    fn default() -> Self {
        LowerCtx {
            variables: HashMap::new(),
            current_variable_id: VariableId(0),
            let_stmts: HashMap::new(),
        }
    }
}

impl LowerCtx {
    pub fn lower_block(&mut self, ast: &ast::Block) -> Result<Block, Error> {
        let mut stmts = Vec::new();

        for stmt in &ast.0 {
            stmts.push(self.lower_stmt(stmt)?);
        }

        Ok(Block(stmts))
    }

    fn lower_stmt(&mut self, ast: &ast::Stmt) -> Result<Stmt, Error> {
        match ast {
            ast::Stmt::Let { name, val } => {
                let val = self.lower_expr(val)?;
                let new_id = VariableId(self.current_variable_id.0 + 1);
                self.variables.insert(name.clone(), new_id);
                self.let_stmts.insert(new_id, val);
                Ok(Stmt::Let(new_id))
            }
            ast::Stmt::Expr(e) => Ok(Stmt::Expr(self.lower_expr(e)?)),
        }
    }

    fn lower_expr(&self, ast: &ast::Expr) -> Result<Expr, Error> {
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
                let mut lowered_args = Vec::new();

                for arg in args {
                    lowered_args.push(self.lower_expr(arg)?);
                }

                Expr::Call { name: name.clone(), args: lowered_args }
            }
            ast::ExprKind::Binary { lhs, rhs, op } => Expr::Binary {
                lhs: Box::new(self.lower_expr(lhs)?),
                rhs: Box::new(self.lower_expr(rhs)?),
                op: *op,
            },
            ast::ExprKind::Prefix { expr, op } => {
                Expr::Prefix { expr: Box::new(self.lower_expr(expr)?), op: *op }
            }
        };

        Ok(e)
    }
}

#[derive(Debug)]
pub struct Block(pub Vec<Stmt>);

#[derive(Debug)]
pub enum Stmt {
    Let(VariableId),
    Expr(Expr),
}

#[derive(Debug)]
pub enum Expr {
    IntLiteral(u32),
    StringLiteral(String),
    CharLiteral(String),
    Variable(VariableId),
    Param { idx: usize },
    Call { name: String, args: Vec<Expr> },
    Binary { lhs: Box<Expr>, rhs: Box<Expr>, op: ast::BinaryOp },
    Prefix { expr: Box<Expr>, op: ast::PrefixOp },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariableId(u32);
