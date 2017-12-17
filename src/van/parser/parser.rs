use std::rc::Rc;

use super::*;

pub struct Parser {
    traveler: Traveler,
    inside:   String,
}

impl Parser {
    pub fn new(traveler: Traveler) -> Parser {
        Parser {
            traveler,
            inside: String::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, Response> {
        let mut stack = Vec::new();

        while self.traveler.remaining() > 1 {
            stack.push(self.statement()?)
        }

        Ok(stack)
    }

    fn skip_whitespace(&mut self) {
        loop {
            if self.traveler.current().token_type == TokenType::Whitespace {
                self.traveler.next();
            } else {
                 break
            }

            if self.traveler.remaining() < 2 {
                break
            }
        }
    }
    
    fn skip_whitespace_eol(&mut self) {
        loop {
            match self.traveler.current().token_type {
                TokenType::Whitespace |
                TokenType::EOL => { self.traveler.next(); }
                _ => break,
            }

            if self.traveler.remaining() < 2 {
                break
            }
        }
    }
    
    fn back_whitespace(&mut self) {
        loop {
            match self.traveler.current().token_type {
                TokenType::Whitespace |
                TokenType::EOL => { self.traveler.prev(); },
                _ => break,
            }

            if self.traveler.top < 2 {
                break
            }
        }
    }
    
    fn get_fun_type(&mut self) -> Result<Type, Response> {
        self.traveler.next();
        self.skip_whitespace();
        
        let mut params = Vec::new();
        
        let mut initial = true;
        
        while self.traveler.current_content() != "->" && self.traveler.current_content() != "\n" {
            if !initial {
                self.traveler.expect_content(",")?;
                self.traveler.next();
                self.skip_whitespace()
            }

            params.push(self.get_type()?);
            self.skip_whitespace();

            initial = false
        }
        
        let retty = if self.traveler.current_content() == "->" {
            self.traveler.next();
            
            self.skip_whitespace();
            
            Some(Rc::new(self.get_type()?))
        } else {
            None
        };

        Ok(Type::Fun(params, retty))
    }

    fn get_type(&mut self) -> Result<Type, Response> {
        match self.traveler.current_content().as_str() {
            "mut" => {
                self.traveler.next();
                
                self.skip_whitespace();

                let a = self.traveler.expect(TokenType::Identifier)?;
                self.traveler.next();
                
                Ok(Type::Mut(Some(Rc::new(Type::Identifier(a)))))
            },
            
            "fun" => self.get_fun_type(),

            _ => {
                let a = Type::Identifier(self.traveler.expect(TokenType::Identifier)?);
                self.traveler.next();
                Ok(a)
            }
        }
    }
    
    fn match_arm(self: &mut Self) -> Result<Option<MatchArm>, Response> {
        self.skip_whitespace_eol();
        
        if self.traveler.current_content() == "|" {
            self.traveler.next();
            self.skip_whitespace_eol();
            
            let param = Rc::new(self.expression()?);

            self.skip_whitespace_eol();

            if self.traveler.current_content() == "->" {
                self.traveler.next();
            }
            
            self.skip_whitespace_eol();

            let body = Rc::new(self.expression()?);
            
            self.skip_whitespace_eol();
            
            Ok(Some(MatchArm {
                param,
                body,
            }))
        } else {
            Ok(None)
        }
    }

    fn match_pattern(&mut self) -> Result<MatchPattern, Response> {
        self.traveler.next();

        self.skip_whitespace_eol();

        let matching = Rc::new(self.expression()?);

        self.skip_whitespace_eol();

        if self.traveler.current_content() == "{" {
            let arms = self.block_of(&Self::match_arm, ("{", "}"))?;

            Ok(MatchPattern {
                matching,
                arms,
            })

        } else {
            panic!("pattern: {:?} : {:#?}", self.traveler.current().token_type, self.traveler.current_content())
        }
    }

    fn block_of<B>(&mut self, match_with: &Fn(&mut Self) -> Result<Option<B>, Response>, delimeters: (&str, &str)) -> Result<Vec<B>, Response> {
        let backup_inside = self.inside.clone();
        self.inside       = delimeters.0.to_owned();
        
        if self.traveler.current_content() == delimeters.0 {
            self.traveler.next();
        }

        let mut stack  = Vec::new();
        let mut nested = 1;

        while nested != 0 {
            if self.traveler.current_content() == delimeters.1 {
                nested -= 1
            } else if self.traveler.current_content() == delimeters.0 {
                nested += 1
            }
            
            if nested == 0 {
                break
            }

            stack.push(self.traveler.current().clone());
            self.traveler.next();
        }

        self.traveler.next();

        let mut parser  = Parser::new(Traveler::new(stack));
        parser.inside   = self.inside.clone();

        let mut stack_b = Vec::new();
        
        while let Some(n) = match_with(&mut parser)? {
            stack_b.push(n)
        }

        self.inside = backup_inside;

        Ok(stack_b)
    }

    fn expression(&mut self) -> Result<Expression, Response> {
        let expr = self.atom()?;

        if expr == Expression::EOF {
            return Ok(expr)
        }

        self.skip_whitespace();
        
        if self.traveler.current().token_type == TokenType::Operator {
            return self.operation(expr)
        }

        Ok(expr)
    }

    fn call(self: &mut Self, callee: Rc<Expression>) -> Result<Call, Response> {
        let mut args = Vec::new();

        while self.traveler.remaining() > 1 {
            args.push(Rc::new(self.expression()?));
            self.skip_whitespace_eol()
        }

        Ok(Call {
            callee,
            args,
        })
    }

    fn atom(&mut self) -> Result<Expression, Response> {       
        self.skip_whitespace();

        if self.traveler.remaining() == 1 {
            return Ok(Expression::EOF)
        }

        match self.traveler.current().token_type {
            TokenType::Int => {
                let a = Expression::Number(self.traveler.current_content().parse::<f64>().unwrap());
                self.traveler.next();
                Ok(a)
            }

            TokenType::Bool => {
                let a = Expression::Bool(self.traveler.current_content() == "true");
                self.traveler.next();
                Ok(a)
            }

            TokenType::Str => {
                let a = Expression::Str(self.traveler.current_content().clone());
                self.traveler.next();
                Ok(a)
            }

            TokenType::Char => {
                let a = Expression::Char(self.traveler.current_content().clone().remove(0));
                self.traveler.next();
                Ok(a)
            }

            TokenType::Identifier => {
                let a = Expression::Identifier(self.traveler.current_content().clone(), self.traveler.current().position);
                self.traveler.next();
                self.skip_whitespace();

                if self.traveler.current().token_type == TokenType::Whitespace {
                    self.traveler.next();
                }

                match self.traveler.current().token_type {
                    TokenType::Int        |
                    TokenType::Identifier |
                    TokenType::Bool       |
                    TokenType::Str        |
                    TokenType::Char       |
                    TokenType::Symbol     => {
                        self.skip_whitespace();

                        if self.traveler.current().token_type == TokenType::Symbol {
                            if self.traveler.current_content() != "(" {
                                return Ok(a)
                            }
                        }

                        let mut stack = Vec::new();

                        let mut nested = 0;

                        if self.inside == "(" {
                            nested = 1
                        }

                        while self.traveler.current().token_type != TokenType::Operator || nested != 0 {
                            if self.traveler.current_content() == "\n" || self.traveler.remaining() < 2 {
                                break
                            }

                            if self.traveler.current_content() == "(" {
                                nested += 1
                            } else if self.traveler.current_content() == ")" {
                                nested -= 1
                            }

                            stack.push(self.traveler.current().clone());
                            self.traveler.next();
                        }

                        let mut parser = Parser::new(Traveler::new(stack));

                        Ok(Expression::Call(Self::call(&mut parser, Rc::new(a))?))
                    }
                    _ => Ok(a)
                }
            },

            TokenType::Keyword => match self.traveler.current_content().as_str() {
                "match"  => Ok(Expression::MatchPattern(self.match_pattern()?)),
                "if"     => Ok(Expression::If(Rc::new(self.if_pattern()?))),
                "unless" => Ok(Expression::Unless(Rc::new(Unless { base: self.if_pattern()? } ))),
                
                ref c => Err(Response::error(Some(ErrorLocation::new(self.traveler.current().position, c.len())), format!("bad keyword: {:?}", c))),
            }

            TokenType::Symbol => match self.traveler.current_content().as_str() {
                "("   => Ok(self.block_of(&Self::expression_, ("(", ")"))?.get(0).unwrap().clone()),
                "["   => Ok(Expression::Array(self.try_array()?.unwrap())),
                ref c => Err(Response::error(Some(ErrorLocation::new(self.traveler.current().position, c.len())), format!("bad symbol: {:?}", c))),
            },

            _ => Err(Response::error(Some(ErrorLocation::new(self.traveler.current().position, self.traveler.current_content().len())), format!("unexpected: {:?}", self.traveler.current_content()))),
        }
    }

    fn try_array(&mut self) -> Result<Option<Vec<Expression>>, Response> {
        if self.traveler.current_content() == "[" {
            self.traveler.next();
        }

        let mut nested = 1;

        let mut stack    = Vec::new();
        let mut is_array = false;
        
        let checkpoint = self.traveler.top;

        while nested != 0 {
            if self.traveler.current_content() == "]" {
                nested -= 1
            } else if self.traveler.current_content() == "[" {
                nested += 1
            }

            if nested == 0 {
                break
            }

            if nested == 1 {
                stack.push(self.expression()?);
                
                if is_array {
                    self.traveler.expect_content(",")?;
                    self.traveler.next();
                } else {
                    if self.traveler.current_content() == "," {
                        is_array = true;
                        self.traveler.next();
                    } else {
                        self.skip_whitespace_eol();

                        if self.traveler.current_content() != "]" {
                            return Err(Response::error(Some(ErrorLocation::new(self.traveler.current().position, 1)), "something's wrong in this index".to_owned()))
                        } else {
                            self.traveler.top = checkpoint;
                            return Ok(None)
                        }
                    }
                }
            }
        }
        
        self.traveler.expect_content("]")?;
        self.traveler.next();

        if is_array {
            Ok(Some(stack))
        } else {
            self.traveler.top = checkpoint;
            Ok(None)
        }
    }
    
    fn assignment(&mut self, left: Rc<Expression>) -> Result<Statement, Response> {
        self.traveler.next();

        let right = Rc::new(self.expression()?);

        Ok(Statement::Assignment(
            Assignment {
                left,
                right,
                position: self.traveler.current().position,
            }
        ))
    }

    fn definition(&mut self, name: String) -> Result<Definition, Response> {
        self.skip_whitespace_eol();
        
        self.traveler.expect_content(":")?;
        self.traveler.next();
        
        self.skip_whitespace();

        let t;

        if self.traveler.current_content() == "=" {
            t = None
        } else {
            t = Some(self.get_type()?);
            self.skip_whitespace();
        }

        if self.traveler.current_content() == "=" {
            self.traveler.next();

            let right = Some(Rc::new(self.expression()?));
            
            Ok(Definition {
                t, name, right, position: self.traveler.current().position
            })

        } else {
            Ok(Definition {
                t, name, right: None, position: self.traveler.current().position
            })
        }
    }

    fn function_match(&mut self) -> Result<FunctionMatch, Response> {
        self.traveler.next();
        self.skip_whitespace_eol();

        let name = self.traveler.current_content().clone();
        
        self.traveler.next();
        self.skip_whitespace_eol();

        let mut t = None;

        if self.traveler.current_content() == "->" {
            self.traveler.next();
            self.skip_whitespace_eol();
            
            t = Some(self.get_type()?);
            self.skip_whitespace_eol();
        }

        let arms = self.block_of(&Self::match_arm, ("{", "}"))?;

        Ok(FunctionMatch {
            t,
            name,
            arms,
        })
    }

    fn expression_(self: &mut Self) -> Result<Option<Expression>, Response> {
        match self.expression()? {
            Expression::EOF => Ok(None),
            c               => Ok(Some(c)),
        }
    }

    fn statement_(self: &mut Self) -> Result<Option<Statement>, Response> {
        match self.statement()? {
            Statement::Expression(e) => match *e {
                Expression::EOF => Ok(None),
                ref e           => Ok(Some(Statement::Expression(Rc::new(e.clone())))),
            },
            c => Ok(Some(c)),
        }
    }

    fn function(&mut self) -> Result<Function, Response> {
        self.traveler.next();
        self.skip_whitespace_eol();

        let name = self.traveler.current_content().clone();
        self.traveler.next();
        self.skip_whitespace_eol();

        let mut params = Vec::new();

        let mut t = None;

        loop {
            match self.traveler.current_content().as_str() {
                "->" => {
                    self.traveler.next();
                    self.skip_whitespace_eol();

                    t = Some(self.get_type()?);
                    self.skip_whitespace_eol();

                    break
                },

                "{" => break,

                _ => {
                    let a = self.traveler.current_content().clone();
                    self.traveler.next();
                    self.skip_whitespace_eol();

                    params.push(self.definition(a)?)
                },
            }
        }

        let body = self.block_of(&Self::statement_, ("{", "}"))?;

        Ok(Function {
            t,
            name,
            params,
            body,
        })
    }

    fn type_definition(self: &mut Self) -> Result<TypeDefinition, Response> {
        self.skip_whitespace_eol();
        let name = self.traveler.current_content().to_owned();
        self.traveler.next();

        self.skip_whitespace();

        self.traveler.expect_content(":")?;
        self.traveler.next();

        self.skip_whitespace();

        let t = self.get_type()?;

        Ok(TypeDefinition {
            name,
            t,
        })
    }

    fn type_definition_(self: &mut Self) -> Result<Option<TypeDefinition>, Response> {
        if self.traveler.remaining() > 2 {
            Ok(Some(self.type_definition()?))
        } else {
            Ok(None)
        }
    }

    fn structure(&mut self) -> Result<Struct, Response> {
        self.traveler.next();
        self.skip_whitespace_eol();
        
        let name = self.traveler.current_content().clone();
        self.traveler.next();
        self.skip_whitespace_eol();

        if self.traveler.current_content() == "{" {
            let body = self.block_of(&Self::type_definition_, ("{", "}"))?;
            
            Ok(Struct {
                name,
                body,
            })
        } else {
            panic!()
        }
    }
    
    fn if_pattern(&mut self) -> Result<If, Response> {
        self.traveler.next();

        self.skip_whitespace_eol();

        let condition = self.expression()?;

        self.skip_whitespace_eol();

        self.traveler.expect_content("{")?;

        let body = self.block_of(&Self::statement_, ("{", "}"))?;

        self.skip_whitespace_eol();

        if self.traveler.current_content() == "elif" || self.traveler.current_content() == "else" {
            let mut elses = Vec::new();
            
            let mut else_flag = false;

            loop {
                let current = self.traveler.current_content();

                if else_flag && (current == "elif" || current == "else") {

                    return Err(
                        Response::group(
                            vec![
                            Response::error(Some(ErrorLocation::new(self.traveler.current().position, current.len())), format!(r#"irrelevant "{}" following previous "else""#, current)),
                            Response::note(None, r#"all cases are already covered at this point"#.to_owned()),
                            ]
                        )
                    )
                }

                match current.as_str() {
                    "elif" => {

                        self.traveler.next();
                        self.skip_whitespace_eol();
                    
                        let condition = self.expression()?;
                        
                        self.skip_whitespace_eol();
                        
                        self.traveler.expect_content("{")?;

                        let body = self.block_of(&Self::statement_, ("{", "}"))?;

                        elses.push((Some(condition), body));

                        self.skip_whitespace_eol();
                    },

                    "else" => {
                        else_flag = true;
                        
                        self.traveler.next();
                        self.skip_whitespace_eol();
                        
                        self.traveler.expect_content("{")?;

                        let body = self.block_of(&Self::statement_, ("{", "}"))?;
                        
                        elses.push((None, body));
                        
                        self.skip_whitespace_eol();
                    },

                    _ => {
                        self.traveler.prev();
                        break
                    }
                }
            }

            Ok(If {
                condition,
                body,
                elses: Some(elses),
            })
            
        } else {
            Ok(If {
                condition,
                body,
                elses: None,
            })
        }
    }

    fn function_type_def_(self: &mut Self) -> Result<Option<TypeDefinition>, Response> {
        if self.traveler.remaining() > 2 {
            let d = self.type_definition()?;
            
            match d.t {
                Type::Fun(..) => (),
                ref c         => return Err(Response::error(Some(ErrorLocation::new(self.traveler.current().position, 5)), format!("invalid function definition: {:?}", c)))
            }
            
            Ok(Some(d))
        } else {
            Ok(None)
        }
    }
    
    fn interface(&mut self) -> Result<Interface, Response> {
        self.traveler.next();
        self.skip_whitespace();
        
        let name = self.traveler.expect(TokenType::Identifier)?;
        self.traveler.next();
        
        self.skip_whitespace();
        
        let types = self.block_of(&Self::function_type_def_, ("{", "}"))?;
        
        Ok(Interface {
            name,
            types,
        })
    }

    fn statement(&mut self) -> Result<Statement, Response> {
        self.skip_whitespace();

        match self.traveler.current().token_type {
            TokenType::Identifier => {
                let a = self.traveler.current_content().clone();
                self.traveler.next();

                self.skip_whitespace_eol();
                
                let position = self.traveler.current().position;

                let b = if self.traveler.current_content() == "=" {
                    self.assignment(Rc::new(Expression::Identifier(a, position)))?
                } else if self.traveler.current_content() == ":" {
                    let c = self.definition(a)?;

                    Statement::Definition(c)
                } else {
                    self.back_whitespace();
                    self.traveler.prev();

                    Statement::Expression(Rc::new(self.expression()?))
                };

                if self.traveler.remaining() > 1 {
                    self.traveler.expect_content("\n")?;
                    self.traveler.next();
                }

                Ok(b)
            },

            TokenType::Keyword => match self.traveler.current_content().as_str() {
                "mut" => {
                    self.traveler.next();

                    self.skip_whitespace_eol();

                    let a = self.traveler.current_content().clone();
                    self.traveler.next();

                    self.skip_whitespace();

                    let mut def = self.definition(a)?;

                    if def.t.is_some() {
                        def.t = Some(Type::Mut(Some(Rc::new(def.t.unwrap()))));
                    }
                    
                    Ok(Statement::Definition(def))
                }
                
                "function"  => Ok(Statement::FunctionMatch(self.function_match()?)),
                "fun"       => Ok(Statement::Function(self.function()?)),
                "struct"    => Ok(Statement::Struct(self.structure()?)),
                "if"        => Ok(Statement::If(self.if_pattern()?)),
                "unless"    => Ok(Statement::Unless(Unless { base: self.if_pattern()? } )),
                "match"     => Ok(Statement::MatchPattern(self.match_pattern()?)),
                "interface" => Ok(Statement::Interface(self.interface()?)),

                _ => Ok(Statement::Expression(Rc::new(self.expression()?))),
            },

            TokenType::EOL => {
                if self.traveler.remaining() > 1 {
                    self.traveler.next();
                    self.statement()
                } else {
                    Ok(Statement::Expression(Rc::new(Expression::EOF)))
                }
            }

            _ => Ok(Statement::Expression(Rc::new(self.expression()?))),
        }
    }
    
    fn operation(&mut self, expression: Expression) -> Result<Expression, Response> {

        let mut ex_stack = vec![expression];
        let mut op_stack: Vec<(Operand, u8)> = Vec::new();
        

        op_stack.push(Operand::from_str(&self.traveler.current_content()).unwrap());
        self.traveler.next();

        ex_stack.push(self.atom()?);

        let mut done = false;

        while ex_stack.len() > 1 {
            if !done {
                self.skip_whitespace();
                if self.traveler.current().token_type != TokenType::Operator {
                    done = true;
                    continue
                }

                let (op, precedence) = Operand::from_str(&self.traveler.current_content()).unwrap();
                self.traveler.next();

                if precedence >= op_stack.last().unwrap().1 {
                    let left  = ex_stack.pop().unwrap();
                    let right = ex_stack.pop().unwrap();

                    ex_stack.push(
                        Expression::BinaryOp(
                            BinaryOp {
                                right: Rc::new(left),
                                op:    op_stack.pop().unwrap().0,
                                left:  Rc::new(right),
                                position: self.traveler.current().position
                            }
                        )
                    );

                    let term = self.atom()?;

                    ex_stack.push(term);
                    op_stack.push((op, precedence));

                    continue
                }
                
                let term = self.atom()?;

                ex_stack.push(term);
                op_stack.push((op, precedence));
            }

            let left  = ex_stack.pop().unwrap();
            let right = ex_stack.pop().unwrap();

            ex_stack.push(
                Expression::BinaryOp(
                    BinaryOp {
                        right: Rc::new(left),
                        op:    op_stack.pop().unwrap().0,
                        left:  Rc::new(right),
                        position: self.traveler.current().position,
                    }
                )
            );
            
            if self.traveler.current_content() == "\n" {
                break
            }
        }

        Ok(ex_stack.pop().unwrap())
    }
}
