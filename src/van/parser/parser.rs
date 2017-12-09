use std::rc::Rc;

use super::*;

pub struct Parser {
    traveler: Traveler,
}

impl Parser {
    pub fn new(traveler: Traveler) -> Parser {
        Parser {
            traveler,
        }
    }

    pub fn parse(&mut self) -> Vec<Statement> {
        let mut stack = Vec::new();

        while self.traveler.remaining() > 1 {
            stack.push(self.statement())
        }

        stack
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.traveler.current().token_type {
                TokenType::Whitespace => { self.traveler.next(); }
                _ => break,
            }

            if self.traveler.remaining() < 2 {
                break
            }
        }
    }

    fn get_type(&mut self) -> Type {
        let a = Type::Identifier(Rc::new(self.traveler.current_content()));
        self.traveler.next();
        a
    }
    
    fn expression(&mut self) -> Expression {
        let expr = self.atom();

        if expr == Expression::EOF {
            return expr
        }

        if self.traveler.remaining() > 1 {
            if self.traveler.current().token_type == TokenType::Operator {
                return self.operation(expr)
            }
        }

        expr
    }
    
    fn atom(&mut self) -> Expression {
        self.skip_whitespace();

        if self.traveler.remaining() == 1 {
            return Expression::EOF
        }
        
        match self.traveler.current().token_type {
            TokenType::Int => {
                let a = Expression::Number(self.traveler.current_content().parse::<f64>().unwrap());
                self.traveler.next();
                a
            }

            TokenType::Bool => {
                let a = Expression::Bool(self.traveler.current_content() == "true");
                self.traveler.next();
                a
            }

            TokenType::Str => {
                let a = Expression::Str(Rc::new(self.traveler.current_content().clone()));
                self.traveler.next();
                a
            }

            TokenType::Char => {
                let a = Expression::Char(self.traveler.current_content().clone().remove(0));
                self.traveler.next();
                a
            }

            TokenType::Identifier => {
                let a = Expression::Identifier(Rc::new(self.traveler.current_content().clone()), self.traveler.current().position);
                self.traveler.next();
                a
            },

            TokenType::Operator => {
                let (op, _) = Operand::from_str(&self.traveler.current_content()).unwrap();
                
                self.traveler.next();
                
                Expression::Unary(Unary {
                    op,
                    expr: Rc::new(self.expression()),
                    position: self.traveler.current().position,
                })
            }

            _ => panic!("{:#?}: {}", self.traveler.current().token_type, self.traveler.current_content()),
        }
    }
    
    fn assignment(&mut self, left: Rc<Expression>) -> Statement {
        self.traveler.next();

        let right = Rc::new(self.expression());

        Statement::Assignment(
            Assignment {
                left,
                right,
                position: self.traveler.current().position,
            }
        )
    }
    
    fn definition(&mut self, left: Rc<Expression>) -> Definition {
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

            let right = Some(Rc::new(self.expression()));

            Definition {
                t, left, right, position: self.traveler.current().position
            }

        } else {
            Definition {
                t, left, right: None, position: self.traveler.current().position
            }
        }
    }
    
    fn statement(&mut self) -> Statement {
        self.skip_whitespace();

        match self.traveler.current().token_type {
            TokenType::Identifier => {
                let a = Expression::Identifier(Rc::new(self.traveler.current_content().clone()), self.traveler.current().position);
                self.traveler.next();
                
                self.skip_whitespace();

                if self.traveler.current_content() == "=" {
                    self.assignment(Rc::new(a))
                } else if self.traveler.current_content() == ":" {
                    Statement::Definition(self.definition(Rc::new(a)))
                } else {
                    self.traveler.prev();
                    Statement::Expression(Rc::new(self.expression()))
                }
            },
            
            TokenType::Keyword => match self.traveler.current_content().as_str() {
                "mut" => {
                    self.traveler.next();
                    
                    self.skip_whitespace();

                    let a = Expression::Identifier(Rc::new(self.traveler.current_content().clone()), self.traveler.current().position);
                    self.traveler.next();
                    
                    self.skip_whitespace();

                    let mut def = self.definition(Rc::new(a));

                    if def.t.is_some() {
                        def.t = Some(Type::Mut(Some(Rc::new(def.t.unwrap()))));
                    }
                    
                    Statement::Definition(def)
                }
                _ => Statement::Expression(Rc::new(self.expression())),
            },
            
            _ => Statement::Expression(Rc::new(self.expression())),
        }
    }
    
    fn operation(&mut self, expression: Expression) -> Expression {
        let mut ex_stack = vec![expression];
        let mut op_stack: Vec<(Operand, u8)> = Vec::new();

        op_stack.push(Operand::from_str(&self.traveler.current_content()).unwrap());
        self.traveler.next();

        if self.traveler.current_content() == "\n" {
            self.traveler.next();
        }

        let atom = self.atom();

        ex_stack.push(atom);

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
                        Expression::Operation(
                            Operation {
                                right: Rc::new(left),
                                op:    op_stack.pop().unwrap().0,
                                left:  Rc::new(right),
                                position: self.traveler.current().position
                            }
                        )
                    );

                    let term = self.atom();

                    ex_stack.push(term);
                    op_stack.push((op, precedence));

                    continue
                }

                let term = self.atom();

                ex_stack.push(term);
                op_stack.push((op, precedence));
            }

            let left  = ex_stack.pop().unwrap();
            let right = ex_stack.pop().unwrap();

            ex_stack.push(
                Expression::Operation(
                    Operation {
                        right: Rc::new(left),
                        op:    op_stack.pop().unwrap().0,
                        left:  Rc::new(right),
                        position: self.traveler.current().position,
                    }
                )
            );
        }

        ex_stack.pop().unwrap()
    }
}
