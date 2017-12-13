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

    fn get_type(&mut self) -> Type {
        let a = Type::Identifier(Rc::new(self.traveler.current_content()));
        self.traveler.next();
        a
    }
    
    fn match_arm(self: &mut Self) -> Result<Option<MatchArm>, Response> {
        self.skip_whitespace();
        
        if self.traveler.current_content() == "|" {
            self.traveler.next();
            self.skip_whitespace();
            
            let param = Rc::new(self.expression()?);

            self.skip_whitespace();

            if self.traveler.current_content() == "->" {
                self.traveler.next();
            }
            
            self.skip_whitespace();
            
            let body = Rc::new(self.expression()?);
            
            self.skip_whitespace();
            
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

        self.skip_whitespace();

        let matching = Rc::new(self.expression()?);

        self.skip_whitespace();

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

        if self.traveler.remaining() > 1 {
            if self.traveler.current().token_type == TokenType::Operator {
                return self.operation(expr)
            }
        }

        Ok(expr)
    }

    fn call(self: &mut Self, callee: Rc<Expression>) -> Result<Call, Response> {
        let mut args = Vec::new();

        while self.traveler.remaining() > 1 {
            args.push(Rc::new(self.expression()?));
            self.skip_whitespace()
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

            TokenType::Operator => {
                let (op, _) = Operand::from_str(&self.traveler.current_content()).unwrap();
                
                self.traveler.next();
                
                Ok(Expression::UnaryOp(UnaryOp {
                    op,
                    expr: Rc::new(self.expression()?),
                    position: self.traveler.current().position,
                }))
            },

            TokenType::Keyword => match self.traveler.current_content().as_str() {
                "match" => {
                    self.traveler.next();

                    Ok(Expression::MatchPattern(self.match_pattern()?))
                },

                ref c => Err(
                    Response::group(
                        vec![
                            Response::error(Some(ErrorLocation::new(self.traveler.current().position, c.len())), format!("bad keyword: `{}`", c)),
                            Response::note(None, "try a real a good keyword".to_owned())
                        ],
                    )
                ),
            }

            TokenType::Symbol => match self.traveler.current_content().as_str() {
                "(" => Ok(self.block_of(&Self::expression_, ("(", ")"))?.get(0).unwrap().clone()),
                _   => panic!("symbol: {}", self.traveler.current_content()),
            },

            _ => panic!("{:#?}: {}", self.traveler.current().token_type, self.traveler.current_content()),
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
        self.skip_whitespace();
        
        self.traveler.expect_content(":");
        self.traveler.next();

        self.skip_whitespace();

        let t;

        if self.traveler.current_content() == "=" {
            self.traveler.next();
            t = None
        } else {
            t = Some(self.get_type());
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
        self.skip_whitespace();

        let name = self.traveler.current_content().clone();
        
        self.traveler.next();
        self.skip_whitespace();

        let mut t = None;

        if self.traveler.current_content() == "->" {
            self.traveler.next();
            self.skip_whitespace();
            
            t = Some(self.get_type());
            self.skip_whitespace();
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
        self.skip_whitespace();

        let name = self.traveler.current_content().clone();
        self.traveler.next();
        self.skip_whitespace();

        let mut params = Vec::new();

        let mut t = None;

        loop {
            match self.traveler.current_content().as_str() {
                "->" => {
                    self.traveler.next();
                    self.skip_whitespace();

                    t = Some(self.get_type());
                    self.skip_whitespace();

                    break
                },

                "{" => break,

                _ => {
                    let a = self.traveler.current_content().clone();
                    self.traveler.next();
                    self.skip_whitespace();

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
        self.skip_whitespace();
        let name = self.traveler.current_content().to_owned();
        self.traveler.next();

        self.skip_whitespace();

        self.traveler.expect_content(":");
        self.traveler.next();

        self.skip_whitespace();

        let t = self.get_type();

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
        self.skip_whitespace();
        
        let name = self.traveler.current_content().clone();
        self.traveler.next();
        self.skip_whitespace();

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

    fn statement(&mut self) -> Result<Statement, Response> {
        self.skip_whitespace();

        match self.traveler.current().token_type {
            TokenType::Identifier => {
                let a = self.traveler.current_content().clone();
                self.traveler.next();

                self.skip_whitespace();
                
                let position = self.traveler.current().position;

                if self.traveler.current_content() == "=" {
                    self.assignment(Rc::new(Expression::Identifier(a, position)))
                } else if self.traveler.current_content() == ":" {
                    Ok(Statement::Definition(self.definition(a)?))
                } else {
                    self.back_whitespace();
                    self.traveler.prev();

                    Ok(Statement::Expression(Rc::new(self.expression()?)))
                }
            },

            TokenType::Keyword => match self.traveler.current_content().as_str() {
                "mut" => {
                    self.traveler.next();

                    self.skip_whitespace();

                    let a = self.traveler.current_content().clone();
                    self.traveler.next();
                    
                    self.skip_whitespace();

                    let mut def = self.definition(a)?;

                    if def.t.is_some() {
                        def.t = Some(Type::Mut(Some(Rc::new(def.t.unwrap()))));
                    }

                    Ok(Statement::Definition(def))
                }
                
                "function" => Ok(Statement::FunctionMatch(self.function_match()?)),
                "fun"      => Ok(Statement::Function(self.function()?)),
                "struct"   => Ok(Statement::Struct(self.structure()?)),
                
                _ => Ok(Statement::Expression(Rc::new(self.expression()?))),
            },

            _ => Ok(Statement::Expression(Rc::new(self.expression()?))),
        }
    }
    
    fn operation(&mut self, expression: Expression) -> Result<Expression, Response> {
        let mut ex_stack = vec![expression];
        let mut op_stack: Vec<(Operand, u8)> = Vec::new();

        op_stack.push(Operand::from_str(&self.traveler.current_content()).unwrap());
        self.traveler.next();

        if self.traveler.current_content() == "\n" {
            self.traveler.next();
        }
        
        let term = self.atom()?;
        ex_stack.push(term);

        let mut done = false;

        while ex_stack.len() > 1 {
            if !done {
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
        }

        Ok(ex_stack.pop().unwrap())
    }
}
