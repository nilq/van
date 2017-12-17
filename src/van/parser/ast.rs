use std::rc::Rc;

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Number(f64),
    Bool(bool),
    Str(String),
    Char(char),
    Identifier(String, TokenPosition),
    BinaryOp(BinaryOp),
    MatchPattern(MatchPattern),
    Call(Call),
    Array(Vec<Expression>),
    If(Rc<If>),
    Unless(Rc<Unless>),
    Struct(Vec<TypeDefinition>),
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
    pub id:    Rc<Expression>,
    pub index: Rc<Expression>,
    pub position: TokenPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDefinition {
    pub name: String,
    pub t: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arm {
    pub params:   Vec<Rc<Expression>>,
    pub body:     Rc<Statement>,
    pub position: TokenPosition,
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    Fun(Fun),
    Match(FunctionMatch)
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionMatch {
    pub t:    Option<Type>,
    pub name: String,
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fun {
    pub t:      Option<Type>,
    pub name:   String,
    pub params: Vec<Definition>,
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
    pub interface: String,
    pub body:      Vec<Function>,
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
}
