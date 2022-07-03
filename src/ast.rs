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
}

#[derive(Debug)]
pub enum Expr {
    IntLiteral(u32),
    StringLiteral(String),
    CharLiteral(String),
    Binary { lhs: Box<Expr>, rhs: Box<Expr>, op: BinaryOp },
    Prefix { expr: Box<Expr>, op: PrefixOp },
}

#[derive(Debug)]
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

#[derive(Debug)]
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
