use super::*;

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

    pub fn visit_expression(&mut self, e: &Expression) -> Result<(), Response> {
        match *e {
            Expression::Identifier(ref n, ref position) => match self.symtab.get_name(&*n) {
                Some(_) => Ok(()),
                None    => Err(Response::error(Some(ErrorLocation::new(*position, n.len())), format!("unexpected use of: {}", n)))
            },

            _ => Ok(())
        }
    }

    pub fn type_expression(&mut self, e: &Expression) -> Result<Type, Response> {
        match *e {
            Expression::Number(_) => Ok(Type::Identifier("number".to_string())),
            Expression::Str(_)    => Ok(Type::Identifier("string".to_string())),
            Expression::Bool(_)   => Ok(Type::Identifier("boolean".to_string())),
            Expression::Identifier(ref n, ref position) => match self.symtab.get_name(&*n) {
                Some((i, env_index)) => self.typetab.get_type(i, env_index),
                None                 => Err(Response::error(Some(ErrorLocation::new(*position, n.len())), format!("unexpected use of: {}", n)))
            },
            _ => Ok(Type::Undefined),
        }
    }

    pub fn visit_statement(&mut self, s: &Statement) -> Result<(), Response> {
        match *s {
            Statement::Expression(ref e) => self.visit_expression(e),
            Statement::Definition(Definition {ref t, ref name, ref right, ref position}) => {
                let index = self.symtab.add_name(&name);
                if index >= self.typetab.size() {
                    self.typetab.grow()
                }

                if let &Some(ref right) = right {
                    let right_t = self.type_expression(&*right)?;
                    
                    if let &Some(ref t) = t {
                        if right_t != *t {
                            Err(Response::error(Some(ErrorLocation::new(*position, name.len())), format!("mismatched types, expected: {:?}", t)))
                        } else {
                            self.typetab.set_type(index, 0, t.clone())
                        }
                    } else {
                        self.typetab.set_type(index, 0, right_t.clone())
                    }
                } else {
                    if let &Some(ref t) = t {
                        self.typetab.set_type(index, 0, t.clone())
                    } else {
                        unreachable!()
                    }
                }
            },
            Statement::Assignment(Assignment {ref left, ref right, ..}) => {
                match **left {
                    Expression::Identifier(ref name, ref position) => {
                        self.visit_expression(left)?;
                        let t = self.type_expression(left)?;

                        if self.type_expression(right)? != t {
                            Err(Response::error(Some(ErrorLocation::new(*position, name.len())), format!("mismatched types, expected: {:?}", t)))
                        } else {
                            Ok(())
                        }
                    },
                    
                    _ => {
                        Response::warning(None, format!("potential unsafe assignment")).display(None);
                        Ok(())
                    }
                }
            },
            _ => Ok(())
        }
    }
}
