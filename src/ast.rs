use std::ops::Range;

#[derive(Debug)]
pub enum Item {
    Function { name: String, params: Vec<(String, Ty)>, return_ty: Ty, body: Block },
    Struct { name: String, fields: Vec<(String, Ty)> },
}

#[derive(Debug)]
pub struct Block(pub Vec<Stmt>);

#[derive(Debug)]
pub enum Stmt {
    Let { name: String, val: Expr },
    Expr(Expr),
}

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub range: Range<usize>,
}

#[derive(Debug)]
pub enum ExprKind {
    IntLiteral(u32),
    StringLiteral(String),
    CharLiteral(String),
    Variable(String),
    Call { name: String, args: Vec<Expr> },
    Binary { lhs: Box<Expr>, rhs: Box<Expr>, op: BinaryOp },
    Prefix { expr: Box<Expr>, op: PrefixOp },
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Eq,
    NEq,
    And,
    Or,
    Lt,
    Gt,
    LtEq,
    GtEq,
}

#[derive(Debug, Clone, Copy)]
pub enum PrefixOp {
    Neg,
    Deref,
    AddrOf,
}

#[derive(Debug)]
pub enum Ty {
    Void,
    Named(String),
    Pointer(Box<Ty>),
}
