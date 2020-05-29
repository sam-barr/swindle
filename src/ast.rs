use crate::error::*;
use std::boxed::Box;
use std::default::Default;

pub trait Tag {
    type TypeTag: core::fmt::Debug + Clone;
    type StatementTag: core::fmt::Debug + Clone;
    type DeclareTag: core::fmt::Debug + Clone;
    type VariableID: core::fmt::Debug + Clone;
    type StringID: core::fmt::Debug + Clone;
    type BuiltinID: core::fmt::Debug + Clone;
}

#[derive(Debug, Clone)]
pub struct Program<T>
where
    T: Tag,
{
    pub statements: Vec<TaggedStatement<T>>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum Statement<T>
where
    T: Tag,
{
    Declare(T::DeclareTag, T::VariableID, Box<Expression<T>>),
    Break,
    Continue,
    Expression(Box<Expression<T>>),
}

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    String,
    Bool,
    Unit,
    List(Box<Type>),
    Fn(Box<Type>, Vec<Type>),
}

// make this a tagged statement?
#[derive(Debug, Clone)]
pub struct Body<T>
where
    T: Tag,
{
    pub statements: Vec<TaggedStatement<T>>,
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

#[derive(Debug, Clone)]
pub enum Expression<T>
where
    T: Tag,
{
    Assign(T::TypeTag, Box<LValue<T>>, Box<Expression<T>>),
    OrExp(Box<OrExp<T>>),
}

#[derive(Debug, Clone)]
pub enum LValue<T>
where
    T: Tag,
{
    Variable(T::VariableID),
    Index(Box<LValue<T>>, Box<Expression<T>>),
}

pub fn primary_to_lvalue<T>(primary: Primary<T>) -> Option<LValue<T>>
where
    T: Tag,
{
    match primary {
        Primary::Variable(var) => Some(LValue::Variable(var)),
        Primary::Index(_, primary, expression) => {
            primary_to_lvalue(*primary).map(|lvalue| LValue::Index(Box::new(lvalue), expression))
        }
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum OrExp<T>
where
    T: Tag,
{
    Or(Box<AndExp<T>>, Box<OrExp<T>>),
    AndExp(Box<AndExp<T>>),
}

#[derive(Debug, Clone)]
pub enum AndExp<T>
where
    T: Tag,
{
    And(Box<CompExp<T>>, Box<AndExp<T>>),
    CompExp(Box<CompExp<T>>),
}

#[derive(Debug, Clone)]
pub enum CompExp<T>
where
    T: Tag,
{
    Comp(CompOp<T>, Box<AddExp<T>>, Box<AddExp<T>>),
    AddExp(Box<AddExp<T>>),
}

#[derive(Debug, Copy, Clone)]
pub enum CompOp<T>
where
    T: Tag,
{
    Leq,
    Lt,
    Eq(T::TypeTag),
}

#[derive(Debug, Clone)]
pub enum AddExp<T>
where
    T: Tag,
{
    Add(AddOp<T>, Box<MulExp<T>>, Box<AddExp<T>>),
    MulExp(Box<MulExp<T>>),
}

#[derive(Debug, Copy, Clone)]
pub enum AddOp<T>
where
    T: Tag,
{
    Sum(T::TypeTag),
    Difference,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum Unary<T>
where
    T: Tag,
{
    Negate(Box<Unary<T>>),
    Not(Box<Unary<T>>),
    Primary(Box<Primary<T>>),
}

#[derive(Debug, Clone)]
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
    ForExp(ForExp<T>),
    StatementExp(Body<T>),
    Index(T::TypeTag, Box<Primary<T>>, Box<Expression<T>>),
    Builtin(T::BuiltinID),
    List(T::DeclareTag, Vec<Expression<T>>),
    Function(Function<T>),
    FunCall(Box<Primary<T>>, Vec<Expression<T>>),
    Unit,
}

#[derive(Debug, Clone)]
pub struct Function<T>
where
    T: Tag,
{
    pub params: Vec<(T::DeclareTag, T::VariableID)>,
    pub body: Body<T>,
}

#[derive(Debug, Clone)]
pub struct ForExp<T>
where
    T: Tag,
{
    // once I add lists, this will build a list, hence the type tag
    pub tag: T::TypeTag,
    pub init: Box<Statement<T>>,
    pub cond: Box<Expression<T>>,
    pub update: Box<Expression<T>>,
    pub body: Body<T>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Elif<T>
where
    T: Tag,
{
    pub cond: Box<Expression<T>>,
    pub body: Body<T>,
}

#[derive(Debug, Clone)]
pub struct Parsed {}

impl Tag for Parsed {
    type TypeTag = ();
    type StatementTag = FilePosition;
    type DeclareTag = Type;
    type VariableID = String;
    type StringID = String;
    type BuiltinID = (String, Vec<Expression<Parsed>>);
}

/*
 * AST traits and implementations
 */

pub trait ToExpression {
    type _Tag: Tag;
    fn to_expression(self) -> Expression<Self::_Tag>;
}

impl<T> ToExpression for Expression<T>
where
    T: Tag,
{
    type _Tag = T;
    fn to_expression(self) -> Expression<T> {
        self
    }
}

impl<T> ToExpression for OrExp<T>
where
    T: Tag,
{
    type _Tag = T;
    fn to_expression(self) -> Expression<T> {
        Expression::OrExp(Box::new(self))
    }
}

impl<T> ToExpression for AndExp<T>
where
    T: Tag,
{
    type _Tag = T;
    fn to_expression(self) -> Expression<T> {
        OrExp::AndExp(Box::new(self)).to_expression()
    }
}

impl<T> ToExpression for CompExp<T>
where
    T: Tag,
{
    type _Tag = T;
    fn to_expression(self) -> Expression<T> {
        AndExp::CompExp(Box::new(self)).to_expression()
    }
}

impl<T> ToExpression for AddExp<T>
where
    T: Tag,
{
    type _Tag = T;
    fn to_expression(self) -> Expression<T> {
        CompExp::AddExp(Box::new(self)).to_expression()
    }
}

impl<T> ToExpression for MulExp<T>
where
    T: Tag,
{
    type _Tag = T;
    fn to_expression(self) -> Expression<T> {
        AddExp::MulExp(Box::new(self)).to_expression()
    }
}

impl<T> ToExpression for Unary<T>
where
    T: Tag,
{
    type _Tag = T;
    fn to_expression(self) -> Expression<T> {
        MulExp::Unary(Box::new(self)).to_expression()
    }
}

impl<T> ToExpression for Primary<T>
where
    T: Tag,
{
    type _Tag = T;
    fn to_expression(self) -> Expression<T> {
        Unary::Primary(Box::new(self)).to_expression()
    }
}

pub trait ToStatement: ToExpression {
    fn to_statement(self) -> Statement<Self::_Tag>
    where
        Self: std::marker::Sized,
    {
        Statement::Expression(Box::new(self.to_expression()))
    }
}

impl<T> ToStatement for Expression<T> where T: Tag {}
impl<T> ToStatement for OrExp<T> where T: Tag {}
impl<T> ToStatement for AndExp<T> where T: Tag {}
impl<T> ToStatement for CompExp<T> where T: Tag {}
impl<T> ToStatement for AddExp<T> where T: Tag {}
impl<T> ToStatement for MulExp<T> where T: Tag {}
impl<T> ToStatement for Unary<T> where T: Tag {}
impl<T> ToStatement for Primary<T> where T: Tag {}
