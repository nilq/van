use std::rc::Rc;

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
    
    pub fn from(symtab: SymTab, typetab: TypeTab) -> Visitor {
        Visitor {
            symtab,
            typetab,
        }
    }

    pub fn visit_expression(&mut self, e: &Expression) -> Result<(), Response> {
        match *e {
            Expression::Identifier(ref n, ref position) => {
                match self.symtab.get_name(&*n) {
                    Some(_) => Ok(()),
                    None    => Err(Response::error(Some(ErrorLocation::new(*position, n.len())), format!("unexpected use of: {}", n)))
                }
            },
            
            Expression::Block(ref statements) => {
                for statement in statements {
                    self.visit_statement(statement)?
                }

                Ok(())
            },
            
            Expression::Array(ref content) => {
                for expression in content {
                    self.visit_expression(expression)?
                }
                Ok(())
            }

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
                None                 => Err(Response::error(Some(ErrorLocation::new(*position, n.len())), format!("undefined type of: {}", n)))
            },

            Expression::Array(ref content) => {
                let mut array_t = Type::Undefined;
                let mut flag    = false;
                
                for expression in content {
                    if !flag {
                        array_t = self.type_expression(expression)?;
                        flag    = true
                    } else {
                        if !array_t.equals(&self.type_expression(expression)?) {
                            return Err(Response::error(None, format!("[location] mismatching array elements")))
                        }
                    }
                }

                Ok(Type::Array(Rc::new(array_t), Some(Expression::Number(content.len() as f64))))
            },

            Expression::Index(Index {ref id, ref position, ..}) => {
                match *self.type_expression(id)?.unmut().unwrap() {
                    Type::Array(ref t, _) => {
                        Ok((**t).clone())
                    },
                    
                    _ => Err(Response::error(Some(ErrorLocation::new(*position, 1)), format!("can't index non-array: {:?}", id)))
                }
            },

            Expression::Block(ref statements) => {
                let mut block_t = Type::Undefined;
                let mut flag    = false;
                
                let mut acc     = 1;

                for statement in statements {
                    if acc == statements.len() {
                        match *statement {
                            Statement::Expression(ref expr) => {
                                if !flag {
                                    block_t = self.type_expression(expr)?;
                                    flag = true
                                } else {
                                    return Err(Response::error(None, format!("[location] mismatching return types of block")))
                                }
                            }
                            Statement::Return(ref expr) => {
                                if !flag {
                                    block_t = if let &Some(ref expr) = expr {
                                        self.type_expression(expr)?
                                    } else {
                                        Type::Identifier("nil".to_string())
                                    };

                                    flag = true
                                } else {
                                    return Err(Response::error(None, format!("[location] mismatching return types of block")))
                                }
                            },
                            _ => {
                                if !flag {
                                    block_t = Type::Identifier("nil".to_string());
                                    flag = true
                                }
                            }
                        }
                    } else {
                        match *statement {
                            Statement::Return(ref expr) => {
                                if !flag {
                                    block_t = if let &Some(ref expr) = expr {
                                        self.type_expression(expr)?
                                    } else {
                                        Type::Identifier("nil".to_string())
                                    };

                                    flag = true
                                } else {
                                    return Err(Response::error(None, format!("[location] mismatching return types of block")))
                                }
                            },

                            _ => (),
                        }
                    }
                    
                    acc += 1
                }
                
                Ok(block_t)
            },

            _ => Ok(Type::Undefined),
        }
    }

    pub fn type_arm(&mut self, arm: &MatchArm) -> Result<Type, Response> {
        self.type_expression(&*arm.body)
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
                        let t = if !t.unmut().is_some() {
                            Type::Mut(Some(Rc::new(right_t.clone())))
                        } else {
                            t.clone()
                        };

                        if !right_t.equals(&t.unmut().unwrap()) {
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

                        match t {
                            Type::Mut(_) => (),
                            _            => return Err(Response::error(Some(ErrorLocation::new(*position, name.len())), format!("reassignment of immutable: {:?}", name)))
                        }

                        if !self.type_expression(right)?.equals(&t) {
                            Err(Response::error(Some(ErrorLocation::new(*position, name.len())), format!("mismatched types, expected: {:?}", t)))
                        } else {
                            Ok(())
                        }
                    },

                    Expression::Index(Index {ref id, ref index, ref position}) => {
                        match self.type_expression(id)? {
                            Type::Mut(ref t) => match **t.as_ref().unwrap() {
                                Type::Array(ref t, _) => {
                                    if let Expression::Identifier(ref name, _) = **index {
                                        Err(Response::error(Some(ErrorLocation::new(*position, name.len())), format!("trying to index array with identifier: {:?}", name)))
                                    } else {
                                        if !self.type_expression(right)?.equals(&t) {
                                            Err(Response::error(Some(ErrorLocation::new(*position, 1)), format!("mismatched types, expected: {:?}", t)))
                                        } else {
                                            Ok(())
                                        }
                                    }
                                },

                                _ => Err(Response::error(Some(ErrorLocation::new(*position, 1)), format!("can't index non-array: {:?}", id))),
                            },

                            _ => Err(Response::error(Some(ErrorLocation::new(*position, 1)), "assigning immutable index".to_string())),
                        }
                    }
                    
                    _ => {
                        Response::warning(None, format!("potential unsafe assignment")).display(None);
                        Ok(())
                    }
                }
            },
            Statement::FunctionMatch(FunctionMatch {ref t, ref name, ref arms}) => {
                match *name.as_ref().unwrap() {
                    Expression::Identifier(ref name, ref position) => match self.symtab.get_name(&*name) {
                        // [todo] check if function and handle function variants
                        Some(_) => Err(Response::error(Some(ErrorLocation::new(*position, name.len())), format!("name already in use: {}", name))),
                        None    => {
                            let index = self.symtab.add_name(&name);
                            if index >= self.typetab.size() {
                                self.typetab.grow()
                            }

                            let local_symtab  = SymTab::new(Rc::new(self.symtab.clone()), &[]);
                            let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &Vec::new());

                            let mut local_visitor = Visitor::from(local_symtab, local_typetab);

                            let mut arm_t = Type::Undefined;
                            let mut flag  = false;

                            for arm in arms {                                
                                if !flag {
                                    arm_t = local_visitor.type_arm(arm)?;
                                    flag = true
                                } else {
                                    if arm_t != local_visitor.type_arm(&arm)? {
                                        return Err(Response::error(None, format!("[error location] mismatching arms of match function: {}", name)))
                                    }
                                }
                            }

                            if let &Some(ref t) = t {
                                if *t != arm_t {
                                    Err(Response::error(None, format!("[location] mismatching return types of function: {}", name)))
                                } else {
                                    local_visitor.typetab.set_type(index, 0, t.clone())
                                }
                            } else {
                                local_visitor.typetab.set_type(index, 0, arm_t.clone())
                            }
                        },
                    },
                    
                    _ => {
                        Response::warning(None, format!("potential unsafe match function")).display(None);
                        Ok(())
                    }
                }
            },
            Statement::Fun(Fun {ref t, ref name, ref params, ref body}) => {
                match *name.as_ref().unwrap() {
                    Expression::Identifier(ref name, ref position) => match self.symtab.get_name(&*name) {
                        // [todo] check if function and handle function variants
                        Some(_) => Err(Response::error(Some(ErrorLocation::new(*position, name.len())), format!("name already in use: {}", name))),
                        None    => {
                            let index = self.symtab.add_name(&name);
                            if index >= self.typetab.size() {
                                self.typetab.grow()
                            }
                            
                            self.typetab.set_type(index, 0, Type::Undefined)?;
                            
                            let mut param_names = Vec::new();
                            let mut param_types = Vec::new();

                            for param in params {
                                param_names.push(param.name.clone());
                                param_types.push(param.t.clone())
                            }

                            let local_symtab  = SymTab::new(Rc::new(self.symtab.clone()), &param_names.as_slice());
                            let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &param_types);

                            let mut local_visitor = Visitor::from(local_symtab, local_typetab);
                            
                            let body_expression = Expression::Block(body.clone());
                            
                            local_visitor.visit_expression(&body_expression)?;

                            let body_t = local_visitor.type_expression(&body_expression)?;

                            if let &Some(ref t) = t {
                                if !t.equals(&body_t) {
                                    Err(Response::error(None, format!("[location] mismatching return types of fun: {}", name)))
                                } else {
                                    local_visitor.typetab.set_type(index, 1, t.clone())?;
                                    self.typetab.set_type(index, 0, t.clone())
                                }
                            } else {
                                local_visitor.typetab.set_type(index, 1, body_t.clone())?;
                                self.typetab.set_type(index, 0, body_t.clone())
                            }
                        },
                    },

                    _ => {
                        Response::warning(None, format!("potential unsafe function")).display(None);
                        Ok(())
                    }
                }
            },
            _ => Ok(())
        }
    }
}