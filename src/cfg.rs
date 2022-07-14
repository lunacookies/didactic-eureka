use crate::tac::Instr;

#[derive(Debug)]
pub struct BasicBlock(pub Vec<Instr>);
