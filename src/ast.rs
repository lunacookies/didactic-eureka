#[derive(Debug)]
pub struct SourceFile(pub Vec<Statement>);

#[derive(Debug)]
pub enum Statement {
	LocalDef { name: String, val: Expr },
	Expr(Expr),
}

#[derive(Debug)]
pub enum Expr {
	Number(u32),
	Variable(String),
	Add { lhs: Box<Expr>, rhs: Box<Expr> },
	If { condition: Box<Expr>, true_branch: Box<Expr>, false_branch: Box<Expr> },
}
