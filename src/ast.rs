#![allow(dead_code)]
use std::boxed::Box;
use std::default::Default;

pub trait Tag {
    type VariableTag: core::fmt::Debug;
    type WriteTag: core::fmt::Debug;
    type StatementTag: core::fmt::Debug;
}

#[derive(Debug)]
pub struct Program<T, ID>
where
    T: Tag,
{
    //pub statements: Vec<(T::StatementTag, Box<Statement<T, ID>>)>,
    pub statements: Vec<TaggedStatement<T, ID>>,
}

#[derive(Debug)]
pub struct TaggedStatement<T, ID>
where
    T: Tag,
{
    pub tag: T::StatementTag,
    pub statement: Statement<T, ID>,
}

impl<T, ID> TaggedStatement<T, ID>
where
    T: Tag,
{
    pub fn new(tag: T::StatementTag, statement: Statement<T, ID>) -> Self {
        TaggedStatement { tag, statement }
    }
}

#[derive(Debug)]
pub enum Statement<T, ID>
where
    T: Tag,
{
    Declare(Type, ID, Box<Expression<T, ID>>),
    Write(T::WriteTag, Box<Expression<T, ID>>),
    Writeln(T::WriteTag, Box<Expression<T, ID>>),
    Expression(Box<Expression<T, ID>>),
}

#[derive(Debug, Copy, Clone)]
pub enum Type {
    Int(),
    String(),
    Bool(),
    Unit(),
}

// make this a tagged statement?
#[derive(Debug)]
pub struct Body<T, ID>
where
    T: Tag,
{
    pub statements: Vec<Statement<T, ID>>,
}

impl<T, ID> Default for Body<T, ID>
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
pub enum Expression<T, ID>
where
    T: Tag,
{
    Assign(ID, Box<Expression<T, ID>>), // TODO: eventually have a LValue enum
    IfExp(Box<IfExp<T, ID>>),
    OrExp(Box<OrExp<T, ID>>),
    // TODO: Control Structures (yes they are expressions)
}

#[derive(Debug)]
pub struct IfExp<T, ID>
where
    T: Tag,
{
    pub cond: Box<Expression<T, ID>>,
    pub body: Body<T, ID>,
    pub elifs: Vec<Elif<T, ID>>,
    pub els: Body<T, ID>, // if its empty there's no else
}

#[derive(Debug)]
pub struct Elif<T, ID>
where
    T: Tag,
{
    pub cond: Box<Expression<T, ID>>,
    pub body: Body<T, ID>,
}

#[derive(Debug)]
pub enum OrExp<T, ID>
where
    T: Tag,
{
    Or(Box<AndExp<T, ID>>, Box<OrExp<T, ID>>),
    AndExp(Box<AndExp<T, ID>>),
}

#[derive(Debug)]
pub enum AndExp<T, ID>
where
    T: Tag,
{
    And(Box<CompExp<T, ID>>, Box<AndExp<T, ID>>),
    CompExp(Box<CompExp<T, ID>>),
}

#[derive(Debug)]
pub enum CompExp<T, ID>
where
    T: Tag,
{
    Comp(CompOp, Box<AddExp<T, ID>>, Box<AddExp<T, ID>>),
    AddExp(Box<AddExp<T, ID>>),
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
pub enum AddExp<T, ID>
where
    T: Tag,
{
    Add(AddOp, Box<MulExp<T, ID>>, Box<AddExp<T, ID>>),
    MulExp(Box<MulExp<T, ID>>),
}

#[derive(Debug, Copy, Clone)]
pub enum AddOp {
    Sum,
    Difference,
}

#[derive(Debug)]
pub enum MulExp<T, ID>
where
    T: Tag,
{
    Mul(MulOp, Box<Unary<T, ID>>, Box<MulExp<T, ID>>),
    Unary(Box<Unary<T, ID>>),
}

#[derive(Debug, Copy, Clone)]
pub enum MulOp {
    Product,
    Quotient,
    Remainder,
}

#[derive(Debug)]
pub enum Unary<T, ID>
where
    T: Tag,
{
    Negate(Box<Unary<T, ID>>),
    Not(Box<Unary<T, ID>>),
    Primary(Box<Primary<T, ID>>),
    //Append(Vec<Box<Primary<T, ID>>>),
}

#[derive(Debug)]
pub enum Primary<T, ID>
where
    T: Tag,
{
    Paren(Box<Expression<T, ID>>),
    IntLit(i32), // I only parse positive integer btw
    StringLit(String),
    BoolLit(bool),
    Variable(T::VariableTag, ID),
    Unit(),
    //Deref(Expression<Tag, ID>),
    //Tuple(Vec<Expression<Tag, ID>>),
}
