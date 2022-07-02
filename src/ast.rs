#[derive(Debug)]
pub enum Item {
    Function { name: String, params: Vec<(String, Ty)>, return_ty: Ty },
    Struct { name: String, fields: Vec<(String, Ty)> },
}

#[derive(Debug)]
pub enum Ty {
    Void,
    Named(String),
    Pointer(Box<Ty>),
}
