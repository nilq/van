use std::rc::Rc;

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Block(Vec<Statement>),
    Number(f64),
    Bool(bool),
    Str(String),
    Char(char),
    Identifier(String, TokenPosition),
    BinaryOp(BinaryOp),
    MatchPattern(MatchPattern),
    Call(Call),
    Index(Index),
    Array(Vec<Expression>),
    If(Rc<If>),
    Unless(Rc<Unless>),
    Struct(Vec<TypeDefinition>),
    Initialization(Rc<Initialization>),
    FunctionMatch(Rc<FunctionMatch>),
    Fun(Rc<Fun>),
    Extern(Rc<Expression>),
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOp {
    pub left:     Rc<Expression>,
    pub op:       Operand,
    pub right:    Rc<Expression>,
    pub position: TokenPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOp {
    pub op:       Operand,
    pub expr:     Rc<Expression>,
    pub position: TokenPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchPattern {
    pub matching: Rc<Expression>,
    pub arms:     Vec<MatchArm>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub param: Rc<Expression>,
    pub body:  Rc<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub callee:   Rc<Expression>,
    pub args:     Vec<Rc<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Index {
    pub id:       Rc<Expression>,
    pub index:    Rc<Expression>,
    pub position: TokenPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDefinition {
    pub name: String,
    pub t: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Initialization {
    pub id:     Option<Expression>,
    pub values: Vec<Assignment>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Rc<Expression>),
    Definition(Definition),
    Assignment(Assignment),
    FunctionMatch(FunctionMatch),
    Fun(Fun),
    Struct(Struct),
    If(If),
    Unless(Unless),
    MatchPattern(MatchPattern),
    Interface(Interface),
    Implementation(Implementation),
    Return(Option<Expression>),
    Import(Import),
    Extern(Rc<Statement>),
    While(While),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    Fun(Fun),
    Match(FunctionMatch)
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionMatch {
    pub t:    Option<Type>,
    pub name: Option<Expression>,
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fun {
    pub t:      Option<Type>,
    pub name:   Option<Expression>,
    pub params: Vec<TypeDefinition>,
    pub body:   Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    pub t:     Option<Type>,
    pub name:  String,
    pub right: Option<Rc<Expression>>,
    pub position: TokenPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: String,
    pub body: Vec<TypeDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct If {
    pub condition: Expression,
    pub body:      Vec<Statement>,
    pub elses:     Option<Vec<(Option<Expression>, Vec<Statement>)>>, // vec<(condition, body)?>
}

#[derive(Debug, Clone, PartialEq)]
pub struct While {
    pub condition: Expression,
    pub body:      Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interface {
    pub name:  String,
    pub types: Vec<TypeDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unless {
    pub base: If,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Implementation {
    pub structure: String,
    pub interface: Option<String>,
    pub body:      Vec<Function>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expose {
    Specifically(Vec<String>),
    Everything,
    Nothing,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub from:   Expression,
    pub expose: Expose,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub left:  Rc<Expression>,
    pub right: Rc<Expression>,
    pub position: TokenPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Pow,
    Mul, Div, Mod,
    Add, Sub,
    Equal, NEqual,
    Lt, Gt, LtEqual, GtEqual,
    Concat,
    PipeLeft, PipeRight,
    XOR,
}

impl Operand {
    pub fn from_str(v: &str) -> Option<(Operand, u8)> {
        match v {
            "^^"  => Some((Operand::Pow, 0)),
            "*"   => Some((Operand::Mul, 1)),
            "/"   => Some((Operand::Div, 1)),
            "%"   => Some((Operand::Mod, 1)),
            "+"   => Some((Operand::Add, 2)),
            "-"   => Some((Operand::Sub, 2)),
            "=="  => Some((Operand::Equal, 3)),
            "~="  => Some((Operand::NEqual, 3)),
            "<"   => Some((Operand::Lt, 4)),
            ">"   => Some((Operand::Gt, 4)),
            "<="  => Some((Operand::LtEqual, 4)),
            ">="  => Some((Operand::GtEqual, 4)),
            "^"   => Some((Operand::XOR, 4)),
            "++"  => Some((Operand::Concat, 5)),
            "<|"  => Some((Operand::PipeLeft, 5)),
            "|>"  => Some((Operand::PipeRight, 5)),
            _     => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Mut(Option<Rc<Type>>),
    Array(Rc<Type>, Option<Expression>),
    Fun(Vec<Type>, Option<Rc<Type>>),
    Identifier(String),
    Undefined,
}

impl Type {
    pub fn equals(&self, other: &Type) -> bool {
        if let &Type::Mut(ref other) = other {
            if let &Type::Mut(_) = self {
                ()
            } else {
                return self.equals(&**other.as_ref().unwrap())
            }
        }

        match *other {
            Type::Array(ref other_t, ref other_len) => match *self {
                Type::Array(ref t, ref len) => {
                    if !other_len.is_some() {
                        self == &Type::Array(other_t.clone(), len.clone())
                    } else if !len.is_some() {
                        Type::Array(t.clone(), other_len.clone()) == Type::Array(other_t.clone(), other_len.clone())
                    } else {
                        self == other
                    }
                },

                _ => self == other
            },
            
            _ => self == other
        }
    }

    pub fn unmut(&self) -> Option<Rc<Type>> {
        if let &Type::Mut(ref unmut) = self {
            unmut.clone()
        } else {
            Some(Rc::new(self.clone()))
        }
    }
}
