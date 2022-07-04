use crate::ast;
use crate::errors::Error;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Index(HashMap<String, Item>);

pub fn index(ast: &[ast::Item]) -> Result<Index, Error> {
    let mut map = HashMap::new();

    for item_ast in ast {
        let (name, item) = match &item_ast.kind {
            ast::ItemKind::Function { name, params, return_ty, body: _ } => {
                let params = params.iter().map(|(name, ty)| (name.clone(), lower_ty(ty))).collect();
                let return_ty = lower_ty(return_ty);
                (name.clone(), Item::Function { params, return_ty })
            }

            ast::ItemKind::Struct { name, fields } => {
                let fields = fields.iter().map(|(name, ty)| (name.clone(), lower_ty(ty))).collect();
                (name.clone(), Item::Struct { fields })
            }
        };

        if map.contains_key(&name) {
            return Err(Error {
                message: format!("`{name}` already defined"),
                range: item_ast.range.clone(),
            });
        }

        map.insert(name, item);
    }

    Ok(Index(map))
}

fn lower_ty(ty: &ast::Ty) -> Ty {
    match ty {
        ast::Ty::Void => Ty::Void,
        ast::Ty::Named(name) => Ty::Named(name.clone()),
        ast::Ty::Pointer(ty) => Ty::Pointer(Box::new(lower_ty(ty))),
    }
}

#[derive(Debug)]
pub enum Item {
    Function { params: Vec<(String, Ty)>, return_ty: Ty },
    Struct { fields: Vec<(String, Ty)> },
}

#[derive(Debug)]
pub enum Ty {
    Void,
    Named(String),
    Pointer(Box<Ty>),
}

impl Index {
    pub fn get(&self, name: &str) -> Option<&Item> {
        self.0.get(name)
    }
}
