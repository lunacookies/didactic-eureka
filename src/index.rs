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
                let params = params.iter().map(|(name, ty)| (name.clone(), ty.clone())).collect();
                (name.clone(), Item::Function { params, return_ty: return_ty.clone() })
            }

            ast::ItemKind::Struct { name, fields } => {
                let fields = fields.iter().map(|(name, ty)| (name.clone(), ty.clone())).collect();
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

#[derive(Debug)]
pub enum Item {
    Function { params: Vec<(String, ast::Ty)>, return_ty: ast::Ty },
    Struct { fields: Vec<(String, ast::Ty)> },
}

impl Index {
    pub fn get(&self, name: &str) -> Option<&Item> {
        self.0.get(name)
    }
}
