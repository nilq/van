use super::*;

use std::collections::HashMap;

pub struct Visitor {
    pub typetab: TypeTab,
    pub symtab:  SymTab,
}

impl Visitor {
    pub fn new() -> Visitor {
        Visitor {
            typetab: TypeTab::new_global(),
            symtab:  SymTab::new_global(),
        }
    }

    pub fn type_expr(&self, e: &Expression) -> Result<Type, Response> {
        match *e {
            Expression::Number(_) => Ok(Type::Identifier("number".to_string())),
            Expression::Str(_)    => Ok(Type::Identifier("string".to_string())),
            Expression::Bool(_)   => Ok(Type::Identifier("boolean".to_string())),
            _                     => Ok(Type::Undefined),
        }
    }
}
