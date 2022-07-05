use crate::ast::{BinaryOp, Ty};
use crate::body::{self, Block, BodyDb, Expr, LocalDef, Stmt};
use crate::index::Item;
use id_arena::Id;

pub fn codegen(items: &[(String, Item)]) -> String {
    let mut emitter = Emitter { buf: String::new(), indentation: 0 };

    for (name, item) in items {
        match item {
            Item::Function { params, return_ty } => {
                emitter.emit_function(name, params, return_ty, body, body_db)
            }
            Item::Struct { fields } => emitter.emit_struct(name, fields),
        }
    }

    emitter.buf
}

struct Emitter {
    buf: String,
    indentation: usize,
}

impl Emitter {
    fn emit_function(
        &mut self,
        name: &str,
        params: &[(String, Ty)],
        return_ty: &Ty,
        body: &Block,
        body_db: &BodyDb,
    ) {
        self.emit_ty(return_ty);
        self.emit(" ");
        self.emit(name);
        self.emit("(");
        for (param_name, param_ty) in params {
            self.emit_ty(param_ty);
            self.emit(" ");
            self.emit(param_name);
        }
        self.emit(") ");
        self.emit_block(body, body_db);
    }

    fn emit_struct(&mut self, name: &str, fields: &[(String, Ty)]) {
        self.emit("struct ");
        self.emit(name);
        self.emit(" {");

        for (field_name, field_ty) in fields {
            self.emit_newline();
            self.emit_ty(field_ty);
            self.emit(" ");
            self.emit(field_name);
            self.emit(";");
        }
    }

    fn emit_block(&mut self, block: &Block, body_db: &BodyDb) {
        self.emit("{");
        self.indentation += 1;
        self.emit_newline();

        for stmt in &block.0 {
            self.emit_stmt(stmt, body_db);
        }

        self.indentation -= 1;
        self.emit_newline();
        self.emit("}");
    }

    fn emit_stmt(&mut self, stmt: &Stmt, body_db: &BodyDb) {
        match stmt {
            Stmt::Let(id) => self.emit_local_def(*id, body_db),
            Stmt::Expr(e) => {
                self.emit_expr(*e, body_db);
                self.emit(";");
            }
        }
    }

    fn emit_local_def(&mut self, id: Id<LocalDef>, body_db: &BodyDb) {
        let local_def = body_db.local_defs[id];
        self.emit_ty(&local_def.ty);
        self.emit(" l");
        self.emit(&id.index().to_string());
        self.emit(" = ");
        self.emit_expr(local_def.val, body_db);
    }

    fn emit_expr(&mut self, id: Id<Expr>, body_db: &BodyDb) {
        match &body_db.exprs[id] {
            Expr::IntLiteral(n) => self.emit(&n.to_string()),
            Expr::StringLiteral(s) => self.emit(&format!("\"{s}\"")),
            Expr::CharLiteral(c) => self.emit(&format!("'{c}'")),
            Expr::Local(id) => self.emit(&format!("l{}", id.index())),
            Expr::Param { idx } => self.emit(&format!("p{}", idx)),
            Expr::Call { name, args } => {
                self.emit(&name);
                self.emit("(");
                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        self.emit(", ");
                    }
                    self.emit_expr(*arg, body_db);
                }
                self.emit(")");
            }
            Expr::Binary { lhs, rhs, op } => {
                self.emit_expr(*lhs, body_db);

                self.emit(" ");
                self.emit(match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Assign => "=",
                    BinaryOp::AddAssign => "+=",
                    BinaryOp::SubAssign => "-=",
                    BinaryOp::MulAssign => "*=",
                    BinaryOp::DivAssign => "/=",
                    BinaryOp::Eq => "==",
                    BinaryOp::NEq => "!=",
                    BinaryOp::And => "&&",
                    BinaryOp::Or => "||",
                    BinaryOp::Lt => "<",
                    BinaryOp::Gt => ">",
                    BinaryOp::LtEq => "<=",
                    BinaryOp::GtEq => ">=",
                });
                self.emit(" ");

                self.emit_expr(*rhs, body_db);
            }
            Expr::Prefix { expr, op } => todo!(),
        }
    }

    fn emit_ty(&mut self, ty: &Ty) {
        match ty {
            Ty::Void => self.emit("void"),
            Ty::Named(n) => self.emit(n),
            Ty::Pointer(inner) => {
                self.emit_ty(inner);
                self.emit("*");
            }
        }
    }

    fn emit_newline(&mut self) {
        self.emit("\n");
        for _ in 0..self.indentation {
            self.emit("\t");
        }
    }

    fn emit(&mut self, s: &str) {
        assert!(!s.contains('\n'));
        self.buf.push_str(s);
    }
}
