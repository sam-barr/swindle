#![allow(dead_code)]
use crate::error::*;
use std::boxed::Box;
use std::default::Default;

pub trait Tag {
    type TypeTag: core::fmt::Debug;
    type StatementTag: core::fmt::Debug;
    type DeclareTag: core::fmt::Debug;
    type VariableID: core::fmt::Debug;
    type StringID: core::fmt::Debug;
}

#[derive(Debug)]
pub struct Program<T>
where
    T: Tag,
{
    pub statements: Vec<TaggedStatement<T>>,
}

#[derive(Debug)]
pub struct TaggedStatement<T>
where
    T: Tag,
{
    pub tag: T::StatementTag,
    pub statement: Statement<T>,
}

impl<T> TaggedStatement<T>
where
    T: Tag,
{
    pub fn new(tag: T::StatementTag, statement: Statement<T>) -> Self {
        TaggedStatement { tag, statement }
    }
}

#[derive(Debug)]
pub enum Statement<T>
where
    T: Tag,
{
    Declare(T::DeclareTag, T::VariableID, Box<Expression<T>>),
    Write(T::TypeTag, bool, Box<Expression<T>>),
    Break,
    Continue,
    Expression(Box<Expression<T>>),
}

#[derive(Debug, Copy, Clone)]
pub enum Type {
    Int,
    String,
    Bool,
    Unit,
}

// make this a tagged statement?
#[derive(Debug)]
pub struct Body<T>
where
    T: Tag,
{
    pub statements: Vec<Statement<T>>,
}

impl<T> Default for Body<T>
where
    T: Tag,
{
    fn default() -> Self {
        Body {
            statements: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum Expression<T>
where
    T: Tag,
{
    Assign(T::TypeTag, T::VariableID, Box<Expression<T>>), // TODO: eventually have a LValue enum
    OrExp(Box<OrExp<T>>),
}

#[derive(Debug)]
pub enum OrExp<T>
where
    T: Tag,
{
    Or(Box<AndExp<T>>, Box<OrExp<T>>),
    AndExp(Box<AndExp<T>>),
}

#[derive(Debug)]
pub enum AndExp<T>
where
    T: Tag,
{
    And(Box<CompExp<T>>, Box<AndExp<T>>),
    CompExp(Box<CompExp<T>>),
}

#[derive(Debug)]
pub enum CompExp<T>
where
    T: Tag,
{
    Comp(CompOp, Box<AddExp<T>>, Box<AddExp<T>>),
    AddExp(Box<AddExp<T>>),
}

#[derive(Debug, Copy, Clone)]
pub enum CompOp {
    Leq,
    Lt,
    Eq,
    Neq,
    Gt,
    Geq,
}

#[derive(Debug)]
pub enum AddExp<T>
where
    T: Tag,
{
    Add(AddOp, Box<MulExp<T>>, Box<AddExp<T>>),
    MulExp(Box<MulExp<T>>),
}

#[derive(Debug, Copy, Clone)]
pub enum AddOp {
    Sum,
    Difference,
}

#[derive(Debug)]
pub enum MulExp<T>
where
    T: Tag,
{
    Mul(MulOp, Box<Unary<T>>, Box<MulExp<T>>),
    Unary(Box<Unary<T>>),
}

#[derive(Debug, Copy, Clone)]
pub enum MulOp {
    Product,
    Quotient,
    Remainder,
}

#[derive(Debug)]
pub enum Unary<T>
where
    T: Tag,
{
    Negate(Box<Unary<T>>),
    Not(Box<Unary<T>>),
    Primary(Box<Primary<T>>),
}

#[derive(Debug)]
pub enum Primary<T>
where
    T: Tag,
{
    Paren(Box<Expression<T>>),
    IntLit(u64), // I only parse positive integer btw
    StringLit(T::StringID),
    BoolLit(bool),
    Variable(T::VariableID),
    IfExp(IfExp<T>),
    WhileExp(WhileExp<T>),
    StatementExp(Body<T>),
    Unit,
}

#[derive(Debug)]
pub struct WhileExp<T>
where
    T: Tag,
{
    pub tag: T::TypeTag,
    pub cond: Box<Expression<T>>,
    pub body: Body<T>,
    pub els: Body<T>,
}

#[derive(Debug)]
pub struct IfExp<T>
where
    T: Tag,
{
    pub tag: T::TypeTag,
    pub cond: Box<Expression<T>>,
    pub body: Body<T>,
    pub elifs: Vec<Elif<T>>,
    pub els: Body<T>, // if its empty there's no else
}

#[derive(Debug)]
pub struct Elif<T>
where
    T: Tag,
{
    pub cond: Box<Expression<T>>,
    pub body: Body<T>,
}

#[derive(Debug)]
pub struct Parsed {}

impl Tag for Parsed {
    type TypeTag = ();
    type StatementTag = FilePosition;
    type DeclareTag = Type;
    type VariableID = String;
    type StringID = String;
}
