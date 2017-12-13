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
    
    fn match_arm(self: &mut Self) -> Option<MatchArm> {
        self.skip_whitespace();
        
        if self.traveler.current_content() == "|" {
            self.traveler.next();
            self.skip_whitespace();
            
            let param = Rc::new(self.expression());
            

            self.skip_whitespace();

            if self.traveler.current_content() == "->" {
                self.traveler.next();
            }
            
            self.skip_whitespace();
            
            let body = Rc::new(self.expression());
            
            self.skip_whitespace();
            
            Some(MatchArm {
                param,
                body,
            })
        } else {
            None
        }
    }

    fn match_pattern(&mut self) -> MatchPattern {
        self.traveler.next();

        self.skip_whitespace();

        let matching = Rc::new(self.expression());

        self.skip_whitespace();

        if self.traveler.current_content() == "{" {
            let arms = self.block_of(&Self::match_arm, ("{", "}"));

            MatchPattern {
                matching,
                arms,
            }

        } else {
            panic!("pattern: {:?} : {:#?}", self.traveler.current().token_type, self.traveler.current_content())
        }
    }

    fn block_of<B>(&mut self, match_with: &Fn(&mut Self) -> Option<B>, delimeters: (&str, &str)) -> Vec<B> {
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
        
        while let Some(n) = match_with(&mut parser) {
            stack_b.push(n)
        }

        self.inside = backup_inside;

        stack_b
    }

    fn expression(&mut self) -> Expression {
        let expr = self.atom();

        if expr == Expression::EOF {
            return expr
        }
        
        self.skip_whitespace();

        if self.traveler.remaining() > 1 {
            if self.traveler.current().token_type == TokenType::Operator {
                return self.operation(expr)
            }
        }

        expr
    }

    fn call(self: &mut Self, callee: Rc<Expression>) -> Call {
        let mut args = Vec::new();

        while self.traveler.remaining() > 1 {
            args.push(Rc::new(self.expression()));
            self.skip_whitespace()
        }

        Call {
            callee,
            args,
        }
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
                let a = Expression::Str(self.traveler.current_content().clone());
                self.traveler.next();
                a
            }

            TokenType::Char => {
                let a = Expression::Char(self.traveler.current_content().clone().remove(0));
                self.traveler.next();
                a
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
                                return a
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

                        Expression::Call(Self::call(&mut parser, Rc::new(a)))
                    }
                    _ => a
                }
            },

            TokenType::Operator => {
                let (op, _) = Operand::from_str(&self.traveler.current_content()).unwrap();
                
                self.traveler.next();
                
                Expression::UnaryOp(UnaryOp {
                    op,
                    expr: Rc::new(self.expression()),
                    position: self.traveler.current().position,
                })
            },
            
            TokenType::Keyword => match self.traveler.current_content().as_str() {
                "match" => {
                    self.traveler.next();
                    
                    Expression::MatchPattern(self.match_pattern())
                },

                _ => panic!("bad keyword: {}", self.traveler.current_content()),
            }

            TokenType::Symbol => match self.traveler.current_content().as_str() {
                "(" => self.block_of(&Self::expression_, ("(", ")")).get(0).unwrap().clone(),
                _   => panic!("symbol: {}", self.traveler.current_content()),
            },

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

    fn definition(&mut self, name: String) -> Definition {
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

            let right = Some(Rc::new(self.expression()));

            Definition {
                t, name, right, position: self.traveler.current().position
            }

        } else {
            Definition {
                t, name, right: None, position: self.traveler.current().position
            }
        }
    }

    fn function_match(&mut self) -> FunctionMatch {
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

        let arms = self.block_of(&Self::match_arm, ("{", "}"));

        FunctionMatch {
            t,
            name,
            arms,
        }
    }

    fn expression_(self: &mut Self) -> Option<Expression> {
        match self.expression() {
            Expression::EOF => None,
            c               => Some(c),
        }
    }

    fn statement_(self: &mut Self) -> Option<Statement> {
        match self.statement() {
            Statement::Expression(e) => match *e {
                Expression::EOF => None,
                ref e           => Some(Statement::Expression(Rc::new(e.clone()))),
            },
            c => Some(c),
        }
    }

    fn function(&mut self) -> Function {
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

                    params.push(self.definition(a))
                },
            }
        }

        let body = self.block_of(&Self::statement_, ("{", "}"));

        Function {
            t,
            name,
            params,
            body,
        }
    }
    
    fn type_definition(self: &mut Self) -> TypeDefinition {
        self.skip_whitespace();
        let name = self.traveler.current_content().to_owned();
        self.traveler.next();
        
        self.skip_whitespace();
        
        self.traveler.expect_content(":");
        self.traveler.next();
        
        self.skip_whitespace();
        
        let t = self.get_type();
        
        TypeDefinition {
            name,
            t,
        }
    }
    
    fn type_definition_(self: &mut Self) -> Option<TypeDefinition> {
        if self.traveler.remaining() > 2 {
            Some(self.type_definition())
        } else {
            None
        }
    }

    fn structure(&mut self) -> Struct {
        self.traveler.next();
        self.skip_whitespace();
        
        let name = self.traveler.current_content().clone();
        self.traveler.next();
        self.skip_whitespace();

        if self.traveler.current_content() == "{" {
            let body = self.block_of(&Self::type_definition_, ("{", "}"));
            
            Struct {
                name,
                body,
            }
        } else {
            panic!()
        }
    }
    
    fn statement(&mut self) -> Statement {
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
                    Statement::Definition(self.definition(a))
                } else {
                    self.back_whitespace();
                    self.traveler.prev();

                    Statement::Expression(Rc::new(self.expression()))
                }
            },

            TokenType::Keyword => match self.traveler.current_content().as_str() {
                "mut" => {
                    self.traveler.next();

                    self.skip_whitespace();

                    let a = self.traveler.current_content().clone();
                    self.traveler.next();
                    
                    self.skip_whitespace();

                    let mut def = self.definition(a);

                    if def.t.is_some() {
                        def.t = Some(Type::Mut(Some(Rc::new(def.t.unwrap()))));
                    }
                    
                    Statement::Definition(def)
                }
                
                "function" => Statement::FunctionMatch(self.function_match()),
                "fun"      => Statement::Function(self.function()),
                "struct"   => Statement::Struct(self.structure()),
                
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
                        Expression::BinaryOp(
                            BinaryOp {
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

        ex_stack.pop().unwrap()
    }
}
