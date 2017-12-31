use std::rc::Rc;
use std::collections::HashMap;

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

    fn alias_type(&self, t: &Type) -> Result<Type, Response> {
        let mut acc_t = t.clone();
        let mut acc   = 0;
        loop {
            match acc_t {
                Type::Mut(_) => {
                    acc += 1;
                    acc_t = (*acc_t.unmut().unwrap()).clone()
                },

                ref t => {
                    let t = match *t {
                        Type::Identifier(ref name) => self.typetab.get_alias(name, 1)?,
                        _                             => t.clone(),
                    };

                    let mut new_t = t;
                    
                    for _ in 0 .. acc {
                        new_t = Type::Mut(Some(Rc::new(new_t.clone())))
                    }
                    
                    return Ok(new_t)
                },
            }
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
            },
            
            Expression::Unless(ref a) => match **a {
                Unless {ref base} => self.visit_expression(&Expression::If(Rc::new(base.clone()))),
            },
            
            Expression::MatchPattern(MatchPattern {ref matching, ref arms}) => {
                self.visit_expression(&matching)?;

                let mut arm_t = Type::Nil;
                let mut flag  = false;

                for arm in arms {
                    self.visit_arm(arm)?;
                    
                    if !self.type_expression(&*arm.param)?.equals(&self.type_expression(matching)?) {
                        return Err(Response::error(None, format!("[location] mismatching arm parameter of match expression")))
                    }

                    if !flag {
                        let a = self.type_arm(arm)?;

                        arm_t = self.alias_type(&a)?;
                        flag = true
                    } else {
                        if arm_t != self.type_arm(&arm)? {
                            return Err(Response::error(None, format!("[location] mismatching arms of match expression")))
                        }
                    }
                }

                Ok(())
            }

            Expression::If(ref a) => match **a {
                If {ref condition, ref body, ref elses} => {
                    self.visit_expression(condition)?;

                    if self.type_expression(condition)? != Type::Bool {
                        return Err(Response::error(None, format!("[location] invalid non-bool if condition")))
                    }
                    
                    let local_symtab  = SymTab::new(Rc::new(self.symtab.clone()), &[]);
                    let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &Vec::new(), &HashMap::new());

                    let mut local_visitor = Visitor::from(local_symtab, local_typetab);
                    
                    local_visitor.visit_expression(&Expression::Block(body.clone()))?;

                    if let &Some(ref elses) = elses {
                        for arm in elses { 
                            if let Some(ref condition) = arm.0 {
                                local_visitor.visit_expression(&condition)?
                            }
                        }
                    }

                    Ok(())
                }
            }

            Expression::BinaryOp(ref op) => {
                let left_t  = (*self.type_expression(&op.left)?.unmut().unwrap()).clone();
                let right_t = (*self.type_expression(&op.right)?.unmut().unwrap()).clone();

                use self::Type::*;
                use self::Operand::*;

                match (left_t, &op.op, right_t) {
                    (a, &Add, b) => match (a, b) {
                        (Number, Number) => Ok(()),
                        (a, b) => Err(Response::error(None, format!("[location] can't add {} and {}", a, b))),
                    },

                    (a, &Sub, b) => match (a, b) {
                        (Number, Number) => Ok(()),
                        (a, b) => Err(Response::error(None, format!("[location] can't subtract {} and {}", a, b))),
                    },

                    (a, &Mul, b) => match (a, b) {
                        (Number, Number) => Ok(()),
                        (a, b) => Err(Response::error(None, format!("[location] can't multiply {} and {}", a, b))),
                    },

                    (a, &Div, b) => match (a, b) {
                        (Number, Number) => Ok(()),
                        (a, b) => Err(Response::error(None, format!("[location] can't divide {} and {}", a, b))),
                    },

                    (a, &Pow, b) => match (a, b) {
                        (Number, Number) => Ok(()),
                        (a, b) => Err(Response::error(None, format!("[location] can't put {} to the power of {}", a, b))),
                    },

                    (a, &Equal, b)   |
                    (a, &NEqual, b)  |
                    (a, &Lt, b)      |
                    (a, &Gt, b)      |
                    (a, &LtEqual, b) |
                    (a, &GtEqual, b) => match (a, b) {
                        (Nil, a) |
                        (a, Nil) => Err(Response::error(None, format!("[location] can't compare {} to nothing", a))),
                        _ => Ok(()),
                    },

                    (a, &Concat, b) => match (a, b) {
                        (Str, Str)    |
                        (Number, Str) |
                        (Str, Bool)   |
                        (Str, Number) => Ok(()),
                        (a, b) => Err(Response::error(None, format!("[location] can't concat {} and {}", a, b))),
                    },

                    (a, &PipeRight, b) => match (self.alias_type(&a)?, self.alias_type(&b)?) {
                        (a, b @ Fun(_, _)) => match b {
                            Type::Fun(ref params, _) => {
                                if params.len() != 1 {
                                    return Err(Response::error(None, format!("[location] function given {} arguments, expected: 1", params.len())))
                                }

                                if self.alias_type(params.get(0).unwrap())? != a {
                                    return Err(Response::error(None, format!("[location] mismatching argument: {:?}", params.get(0).unwrap())))
                                }

                                Ok(())
                            },

                            ref c => Err(Response::error(None, format!("[location] can't call non-fun: {:?} of {:?}", b, c)))
                        },
                        
                        _ => panic!(),
                    },

                    (a, o, b) => Err(Response::error(None, format!("[location] unimplemented operation: {} {:?} {}", a, o, b))),
                }
            },

            Expression::Initialization(ref a) => match **a {
                Initialization {ref id, ref values} => {
                    let id_t = self.type_expression(id)?;
                    let a    = self.alias_type(&id_t)?;
                    if let Type::Struct(ref hash) = a {
                        for def in values {
                            let mut found = false;
                            for (name, t) in hash.iter() {
                                match *def.left {
                                    Expression::Identifier(ref n, _) =>
                                        if n == name {
                                            let right_t = self.type_expression(&def.right)?;
                                            
                                            if self.alias_type(t)?.equals(&self.alias_type(&right_t)?) {
                                                found = true
                                            } else {
                                                return Err(Response::error(None, format!("[location] {} expected \"{}\", found: {}", name, **t, right_t)))
                                            }
                                        } else {
                                            continue
                                        },

                                    ref c => return Err(Response::error(None, format!("[location] can't set invalid key: {:?}", c)))
                                }
                            }

                            if !found {
                                return Err(Response::error(None, format!("[location] invalid initialization of: {:?}", def.left)))
                            }
                        }
                        
                        Ok(())
                    } else {
                        Err(Response::error(None, format!("[location] can't initialize: {}", a)))
                    }
                }
            },

            Expression::Call(Call {ref callee, ref args}) => {
                let callee_t = self.type_expression(callee)?;
                match self.alias_type(&callee_t)? {
                    Type::Fun(ref params, _) => {
                        let mut acc = 0;
                        
                        if params.len() != args.len() {
                            return Err(Response::error(None, format!("[location] function given {} arguments, expected: {}", params.len(), args.len())))
                        }
                        
                        for param in params {
                            let arg = &*self.type_expression(&*args[acc])?.unmut().unwrap();
                            if self.alias_type(param)? != self.alias_type(arg)? {
                                return Err(Response::error(None, format!("[location] mismatching argument: {:?}", param)))
                            }

                            acc += 1
                        }

                        Ok(())
                    },

                    ref c => Err(Response::error(None, format!("[location] can't call non-fun: {:?} of {:?}", callee, c)))
                }
            },

            Expression::FunctionMatch(ref a) => match **a {
                FunctionMatch {ref t, ref arms, ..} => {
                    let local_symtab  = SymTab::new(Rc::new(self.symtab.clone()), &[]);
                    let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &Vec::new(), &HashMap::new());

                    let mut local_visitor = Visitor::from(local_symtab, local_typetab);

                    let mut arm_t = Type::Nil;
                    let mut flag  = false;

                    for arm in arms {
                        local_visitor.visit_arm(arm)?;    
                        if !flag {
                            arm_t = self.alias_type(&local_visitor.type_arm(arm)?)?;
                            flag = true
                        } else {
                            if arm_t != local_visitor.type_arm(&arm)? {
                                return Err(Response::error(None, format!("[location] mismatching arms of match function expression")))
                            }
                        }
                    }

                    if let &Some(ref t) = t {
                        let t = self.alias_type(t)?;
                        if t != arm_t {
                            Err(Response::error(None, format!("[location] mismatching return types of function expression")))
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                }
            },

            Expression::Fun(ref a) => match **a {
                Fun {ref t, ref params, ref body, ..} => {
                    let mut param_names = Vec::new();
                    let mut param_types = Vec::new();

                    for param in params {
                        param_names.push(param.name.clone());
                        param_types.push(param.t.clone())
                    }

                    let local_symtab  = SymTab::new(Rc::new(self.symtab.clone()), &param_names.as_slice());
                    let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &param_types, &HashMap::new());

                    let mut local_visitor = Visitor::from(local_symtab, local_typetab);

                    let body_expression = Expression::Block(body.clone());

                    local_visitor.visit_expression(&body_expression)?;

                    if let &Some(ref t) = t {
                        let t      = self.alias_type(&t)?;
                        let body_t = self.alias_type(&local_visitor.type_expression(&body_expression)?)?;

                        if !t.equals(&body_t) {
                            Err(Response::error(None, format!("[location] mismatching return types of fun expression")))
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                }
            },

            _ => Ok(())
        }
    }

    pub fn type_expression(&mut self, e: &Expression) -> Result<Type, Response> {
        match *e {
            Expression::Number(_) => Ok(Type::Number),
            Expression::Str(_)    => Ok(Type::Str),
            Expression::Bool(_)   => Ok(Type::Bool),
            Expression::Identifier(ref n, ref position) => match self.symtab.get_name(&*n) {
                Some((i, env_index)) => self.typetab.get_type(i, env_index),
                None                 => Err(Response::error(Some(ErrorLocation::new(*position, n.len())), format!("undefined type of: {}", n)))
            },

            Expression::Initialization(ref a) => match **a {
                Initialization {ref id, ..} => {
                    let a = self.type_expression(id)?;
                    self.alias_type(&a)
                }
            }

            Expression::Array(ref content) => {
                let mut array_t = Type::Nil;
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

            Expression::Index(Index {ref id, ref position, ref index}) => {
                let a = self.type_expression(id)?;

                match *self.alias_type(&a)?.unmut().unwrap() {
                    Type::Array(ref t, _) => {
                        Ok((**t).clone())
                    },

                    Type::Struct(ref defs) => {
                        if let Expression::Identifier(ref name, _) = **index {
                            if let Some(a) = defs.get(name) {
                                self.alias_type(a)
                            } else {
                                Err(Response::error(Some(ErrorLocation::new(*position, 1)), format!("invalid key: {}", name)))
                            }
                        } else {
                            Err(Response::error(Some(ErrorLocation::new(*position, 1)), format!("can't access struct with: {}", self.type_expression(&*index)?)))
                        }
                    }

                    _ => Err(Response::error(Some(ErrorLocation::new(*position, 1)), format!("can't index non-indexable: {:?}", id)))
                }
            },

            Expression::Unless(ref a) => match **a {
                Unless {ref base} => self.type_expression(&Expression::If(Rc::new(base.clone()))),
            },

            Expression::If(ref a) => match **a {
                If {ref body, ref elses, ..} => {
                    let local_symtab  = SymTab::new(Rc::new(self.symtab.clone()), &[]);
                    let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &Vec::new(), &HashMap::new());

                    let mut local_visitor = Visitor::from(local_symtab, local_typetab);
                    let mut arm_t         = local_visitor.type_expression(&Expression::Block(body.clone()))?;
                    
                    if let &Some(ref elses) = elses {
                        for arm in elses { 
                            if arm_t != local_visitor.type_expression(&Expression::Block(arm.1.clone()))? {
                                return Err(Response::error(None, format!("[location] mismatching branches of if expression")))
                            }
                        }
                    }

                    Ok(arm_t)
                }
            }
            
            Expression::MatchPattern(MatchPattern {ref matching, ref arms}) => {
                self.visit_expression(&matching)?;

                let mut arm_t = Type::Nil;
                let mut flag  = false;

                for arm in arms {
                    self.visit_arm(arm)?;
                    
                    if !self.type_expression(&*arm.param)?.equals(&self.type_expression(matching)?) {
                        return Err(Response::error(None, format!("[location] mismatching arm parameter of match expression")))
                    }

                    if !flag {
                        let a = self.type_arm(arm)?;

                        arm_t = self.alias_type(&a)?;
                        flag = true
                    } else {
                        if arm_t != self.type_arm(&arm)? {
                            return Err(Response::error(None, format!("[location] mismatching arms of match expression")))
                        }
                    }
                }

                Ok(arm_t)
            }
            
            Expression::BinaryOp(ref op) => {
                let left_t  = (*self.type_expression(&op.left)?.unmut().unwrap()).clone();
                let right_t = (*self.type_expression(&op.right)?.unmut().unwrap()).clone();

                use self::Type::*;
                use self::Operand::*;

                match (left_t, &op.op, right_t) {
                    (a, &Add, b) => match (a, b) {
                        (Number, Number) => Ok(Number),
                        (a, b) => Err(Response::error(None, format!("[location] can't add {} and {}", a, b))),
                    },

                    (a, &Sub, b) => match (a, b) {
                        (Number, Number) => Ok(Number),
                        (a, b) => Err(Response::error(None, format!("[location] can't subtract {} and {}", a, b))),
                    },

                    (a, &Mul, b) => match (a, b) {
                        (Number, Number) => Ok(Number),
                        (a, b) => Err(Response::error(None, format!("[location] can't multiply {} and {}", a, b))),
                    },

                    (a, &Div, b) => match (a, b) {
                        (Number, Number) => Ok(Number),
                        (a, b) => Err(Response::error(None, format!("[location] can't divide {} and {}", a, b))),
                    },

                    (a, &Pow, b) => match (a, b) {
                        (Number, Number) => Ok(Number),
                        (a, b) => Err(Response::error(None, format!("[location] can't put {} to the power of {}", a, b))),
                    },

                    (a, &Equal, b)   |
                    (a, &NEqual, b)  |
                    (a, &Lt, b)      |
                    (a, &Gt, b)      |
                    (a, &LtEqual, b) |
                    (a, &GtEqual, b) => match (a, b) {
                        (Nil, a) |
                        (a, Nil) => Err(Response::error(None, format!("[location] can't compare {} to nothing", a))),
                        _ => Ok(Bool),
                    },

                    (a, &Concat, b) => match (a, b) {
                        (Str, Str) |
                        (Number, Str) |
                        (Str, Bool)   |
                        (Str, Number) => Ok(Str),
                        (a, b) => Err(Response::error(None, format!("[location] can't concat {} and {}", a, b))),
                    },

                    (a, &PipeRight, b) => match (self.alias_type(&a)?, self.alias_type(&b)?) {
                        (a, b @ Fun(_, _)) => match b {
                            Type::Fun(_, ref retty) => {
                                if let &Some(ref retty) = retty {
                                    Ok(retty.as_ref().clone())
                                } else {
                                    Ok(Type::Nil)
                                }
                            },

                            ref c => Err(Response::error(None, format!("[location] can't call non-fun: {:?} of {:?}", b, c)))
                        },
                        
                        _ => panic!(),
                    },

                    (a, o, b) => Err(Response::error(None, format!("[location] unimplemented operation: {} {:?} {}", a, o, b))),
                }
            },
            
            Expression::Call(Call {ref callee, ..}) => {
                let a = self.type_expression(callee)?;

                match self.alias_type(&a)? {
                    Type::Fun(_, ref retty) => {
                        if let &Some(ref retty) = retty {
                            Ok(retty.as_ref().clone())
                        } else {
                            Ok(Type::Nil)
                        }
                    },
                    
                    ref c => Err(Response::error(None, format!("[location] can't call non-fun: {:?} of {:?}", callee, c)))
                }
            },
            
            Expression::Fun(ref a) => match **a {
                Fun {ref t, ref params, ref body, ..} => {
                    let mut param_names = Vec::new();
                    let mut param_types = Vec::new();

                    for param in params {
                        param_names.push(param.name.clone());
                        param_types.push(param.t.clone())
                    }

                    let local_symtab  = SymTab::new(Rc::new(self.symtab.clone()), &param_names.as_slice());
                    let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &param_types, &HashMap::new());

                    let mut local_visitor = Visitor::from(local_symtab, local_typetab);

                    let body_expression = Expression::Block(body.clone());

                    local_visitor.visit_expression(&body_expression)?;

                    let body_t = self.alias_type(&local_visitor.type_expression(&body_expression)?)?;
                    
                    if let &Some(ref t) = t {
                        let t = self.alias_type(t)?;

                        if !t.equals(&body_t) {
                            Err(Response::error(None, format!("[location] mismatching return types of fun expression")))
                        } else {
                            let t = Type::Fun(param_types, Some(Rc::new(t.clone())));
                            Ok(t.clone())
                        }
                    } else {
                        let t = Type::Fun(param_types, Some(Rc::new(self.alias_type(&body_t)?)));
                        
                        Ok(t.clone())
                    }
                }
            },
            
            Expression::FunctionMatch(ref a) => match **a {
                FunctionMatch {ref t, ref arms, ..} => {
                    let local_symtab  = SymTab::new(Rc::new(self.symtab.clone()), &[]);
                    let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &Vec::new(), &HashMap::new());

                    let mut local_visitor = Visitor::from(local_symtab, local_typetab);

                    let mut arm_t   = Type::Nil;
                    let mut param_t = Type::Nil;

                    let mut flag    = false;

                    for arm in arms {
                        if !flag {
                            arm_t   = self.alias_type(&local_visitor.type_arm(arm)?)?;
                            param_t = self.alias_type(&local_visitor.type_expression(&*arm.param)?)?;
                            flag = true
                        } else {
                            if arm_t != local_visitor.type_arm(&arm)? {
                                return Err(Response::error(None, format!("[location] mismatching arms of match function expression")))
                            }
                            
                            if param_t != local_visitor.type_expression(&*arm.param)? {
                                return Err(Response::error(None, format!("[location] mismatching arm parameters of match function expression")))
                            }
                        }
                    }

                    if let &Some(ref t) = t {
                        let t = self.alias_type(t)?;
                        if t != arm_t {
                            Err(Response::error(None, format!("[location] mismatching return types of function expression")))
                        } else {
                            Ok(Type::Fun(vec!(param_t), Some(Rc::new(t.clone()))))
                        }
                    } else {
                        Ok(Type::Fun(vec!(param_t), Some(Rc::new(arm_t.clone()))))
                    }
                }
            },

            Expression::Block(ref statements) => {
                let mut block_t = Type::Nil;
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
                                        Type::Nil
                                    };

                                    flag = true
                                } else {
                                    return Err(Response::error(None, format!("[location] mismatching return types of block")))
                                }
                            },
                            _ => {
                                if !flag {
                                    block_t = Type::Nil;
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
                                        Type::Nil
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

            _ => Ok(Type::Nil),
        }
    }

    pub fn type_arm(&mut self, arm: &MatchArm) -> Result<Type, Response> {
        let mut param: Vec<String> = Vec::new();

        if let Expression::Identifier(ref name, _) = *arm.param {
            param.push(name.to_owned())
        }

        let local_symtab  = SymTab::new(Rc::new(self.symtab.clone()), &param.as_slice());
        let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &Vec::new(), &HashMap::new());

        let mut local_visitor = Visitor::from(local_symtab, local_typetab);

        local_visitor.type_expression(&*arm.body)
    }

    pub fn visit_arm(&mut self, arm: &MatchArm) -> Result<(), Response> {
        self.visit_expression(&*arm.body)
    }

    pub fn visit_statement(&mut self, s: &Statement) -> Result<(), Response> {
        match *s {
            Statement::Extern(ref statement) => self.visit_statement(statement),
            Statement::Expression(ref e)     => self.visit_expression(e),
            Statement::Struct(Struct {ref name, ref body}) => match self.symtab.get_name(&*name) {
                Some(_) => Err(Response::error(None, format!("[location] struct's name already in use: {}", name))),
                None    => {
                    let index = self.symtab.add_name(&name);
                    if index >= self.typetab.size() {
                        self.typetab.grow()
                    }

                    let mut types = HashMap::new();

                    for def in body {
                        types.insert(def.name.clone(), Rc::new(Type::Mut(Some(Rc::new(def.t.clone())))));
                    }

                    self.typetab.set_alias(0, &name, Type::Struct(types.clone()))?;
                    self.typetab.set_type(index, 0, Type::Identifier(name.clone()))
                },
            },

            Statement::Definition(Definition {ref t, ref name, ref right, ref position}) => {
                let index = self.symtab.add_name(&name);
                if index >= self.typetab.size() {
                    self.typetab.grow()
                }

                if let &Some(ref right) = right {
                    self.visit_expression(&*right)?;

                    let a = self.type_expression(&*right)?;
                    let right_t = self.alias_type(&a)?;

                    if let &Some(ref t) = t {
                        let t = if !t.is_empty_mut() {
                            if t.is_mut() {
                                Type::Mut(Some(Rc::new(self.alias_type(&t.unmut().unwrap())?)))
                            } else {
                                self.alias_type(&t)?
                            }
                        } else {
                            if t.is_mut() {
                                Type::Mut(Some(Rc::new(self.alias_type(&right_t.clone())?)))
                            } else {
                                self.alias_type(&right_t)?
                            }
                        };

                        if !right_t.equals(&t.unmut().unwrap()) {
                            Err(Response::error(Some(ErrorLocation::new(*position, name.len())), format!("mismatched types, expected \"{}\", found: {}", t, right_t)))
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

                        // hmm
                        let a = self.type_expression(left)?;
                        let t = self.alias_type(&a)?;

                        match t {
                            Type::Mut(_) => (),
                            _            => return Err(Response::error(Some(ErrorLocation::new(*position, name.len())), format!("reassignment of immutable: {}", name)))
                        }

                        self.visit_expression(&right)?;

                        let right_t = self.type_expression(right)?;

                        if !self.alias_type(&right_t)?.equals(&t) {
                            Err(Response::error(Some(ErrorLocation::new(*position, name.len())), format!("mismatched types, expected: {}", t)))
                        } else {
                            Ok(())
                        }
                    },

                    Expression::Index(Index {ref id, ref index, ref position}) => {
                        let t = self.type_expression(id)?;
                        
                        match self.alias_type(&t)? {
                            Type::Mut(ref t) => match self.alias_type(&*t.as_ref().unwrap())? {
                                Type::Array(ref t, _) => {
                                    if let Expression::Identifier(ref name, _) = **index {
                                        Err(Response::error(Some(ErrorLocation::new(*position, name.len())), format!("trying to index array with identifier: {}", name)))
                                    } else {
                                        if !self.type_expression(right)?.equals(&t) {
                                            Err(Response::error(Some(ErrorLocation::new(*position, 1)), format!("mismatched types, expected: {}", t)))
                                        } else {
                                            Ok(())
                                        }
                                    }
                                },

                                Type::Struct(ref defs) => {
                                    if let Expression::Identifier(ref name, _) = **index {
                                        let t = self.alias_type(defs.get(name).unwrap())?;

                                        let a = self.type_expression(right)?;
                                        if !self.alias_type(&a)?.equals(&t) {
                                            Err(Response::error(Some(ErrorLocation::new(*position, 1)), format!("mismatched types, expected '{}', found: {}", t, a)))
                                        } else {
                                            Ok(())
                                        }
                                    } else {
                                        Err(Response::error(Some(ErrorLocation::new(*position, 1)), format!("can't access struct with: {}", self.type_expression(&*index)?)))
                                    }
                                },

                                c => Err(Response::error(Some(ErrorLocation::new(*position, 1)), format!("can't index: {}", c))),
                            },

                            c => Err(Response::error(Some(ErrorLocation::new(*position, 1)), format!("assigning immutable index: {}", c))),
                        }
                    }
                    
                    _ => {
                        Response::warning(None, format!("potential unsafe assignment")).display(None);
                        Ok(())
                    }
                }
            },
            
            Statement::Unless(ref unless) => self.visit_expression(&Expression::If(Rc::new(unless.base.clone()))),
            Statement::If(ref base)       => self.visit_expression(&Expression::If(Rc::new(base.clone()))),
            
            Statement::While(ref base) => {
                self.visit_expression(&base.condition)?;

                if self.type_expression(&base.condition)? != Type::Bool {
                    return Err(Response::error(None, format!("[location] invalid non-bool while condition")))
                }
                
                self.visit_expression(&Expression::Block(base.body.clone()))
            } 
            
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
                            let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &Vec::new(), &HashMap::new());

                            let mut local_visitor = Visitor::from(local_symtab, local_typetab);

                            let mut arm_t   = Type::Nil;
                            let mut param_t = Type::Nil;
                            let mut flag    = false;

                            for arm in arms {
                                if !flag {
                                    arm_t   = self.alias_type(&local_visitor.type_arm(arm)?)?;
                                    param_t = self.alias_type(&local_visitor.type_expression(&*arm.param)?)?;
                                    flag = true
                                } else {
                                    if arm_t != local_visitor.type_arm(&arm)? {
                                        return Err(Response::error(None, format!("[location] mismatching arms of match function expression")))
                                    }
                                    
                                    if param_t != local_visitor.type_expression(&*arm.param)? {
                                        return Err(Response::error(None, format!("[location] mismatching arm parameters of match function expression")))
                                    }
                                }
                            }

                            if let &Some(ref t) = t {
                                let t = self.alias_type(t)?;
                                if t != arm_t {
                                    Err(Response::error(None, format!("[location] mismatching return types of function: {}", name)))
                                } else {
                                    local_visitor.typetab.set_type(index, 0, Type::Fun(vec!(param_t), Some(Rc::new(t.clone()))))
                                }
                            } else {
                                local_visitor.typetab.set_type(index, 0, Type::Fun(vec!(param_t), Some(Rc::new(arm_t.clone()))))
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

                            self.typetab.set_type(index, 0, Type::Nil)?;

                            let mut param_names = Vec::new();
                            let mut param_types = Vec::new();

                            for param in params {
                                param_names.push(param.name.clone());
                                param_types.push(param.t.clone())
                            }

                            let local_symtab  = SymTab::new(Rc::new(self.symtab.clone()), &param_names.as_slice());
                            let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &param_types, &HashMap::new());

                            let mut local_visitor = Visitor::from(local_symtab, local_typetab);

                            let body_expression = Expression::Block(body.clone());

                            local_visitor.visit_expression(&body_expression)?;

                            let body_t = self.alias_type(&local_visitor.type_expression(&body_expression)?)?;

                            if let &Some(ref t) = t {
                                let t = self.alias_type(t)?;

                                if !t.equals(&body_t) {
                                    Err(Response::error(None, format!("[location] mismatching return types of fun: {} of {:?}\n\t:: {:?}", name, t, body_t)))
                                } else {                                    
                                    let t = Type::Fun(param_types, Some(Rc::new(t.clone())));

                                    local_visitor.typetab.set_type(index, 1, t.clone())?;
                                    self.typetab.set_type(index, 0, t.clone())
                                }
                            } else {
                                let t = Type::Fun(param_types, Some(Rc::new(body_t.clone())));
                                
                                local_visitor.typetab.set_type(index, 1, t.clone())?;
                                self.typetab.set_type(index, 0, t.clone())
                            }
                        },
                    },

                    _ => {
                        Response::warning(None, format!("potential unsafe function")).display(None);
                        Ok(())
                    }
                }
            },
            Statement::Return(ref expr) => if let &Some(ref expr) = expr {
                self.visit_expression(&expr)
            } else {
                Ok(())
            }
            _ => Ok(())
        }
    }
}
