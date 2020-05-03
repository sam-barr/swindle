#![allow(dead_code)]
use std::boxed::Box;

#[derive(Debug)]
pub struct Program<ID> {
    pub statements: Vec<Box<Statement<ID>>>,
}

#[derive(Debug)]
pub enum Statement<ID> {
    Declare(Type, ID, Box<Expression<ID>>),
    Write(Box<Expression<ID>>),
    Writeln(Box<Expression<ID>>),
    Expression(Box<Expression<ID>>),
}

#[derive(Debug, Copy, Clone)]
pub enum Type {
    Int(),
    String(),
    Bool(),
    Unit(),
}

#[derive(Debug)]
pub enum Expression<ID> {
    Assign(ID, Box<Expression<ID>>), // TODO: eventually have a LValue enum
    OrExp(Box<OrExp<ID>>),
    // TODO: Control Structures (yes they are expressions)
}

#[derive(Debug)]
pub enum OrExp<ID> {
    Or(Box<AndExp<ID>>, Box<OrExp<ID>>),
    AndExp(Box<AndExp<ID>>),
}

#[derive(Debug)]
pub enum AndExp<ID> {
    And(Box<CompExp<ID>>, Box<AndExp<ID>>),
    CompExp(Box<CompExp<ID>>),
}

#[derive(Debug)]
pub enum CompExp<ID> {
    Leq(Box<AddExp<ID>>, Box<AddExp<ID>>),
    Lt(Box<AddExp<ID>>, Box<AddExp<ID>>),
    Eq(Box<AddExp<ID>>, Box<AddExp<ID>>),
    Neq(Box<AddExp<ID>>, Box<AddExp<ID>>),
    Gt(Box<AddExp<ID>>, Box<AddExp<ID>>),
    Geq(Box<AddExp<ID>>, Box<AddExp<ID>>),
    AddExp(Box<AddExp<ID>>),
}

#[derive(Debug)]
pub enum AddExp<ID> {
    Sum(Box<MulExp<ID>>, Box<AddExp<ID>>),
    Difference(Box<MulExp<ID>>, Box<AddExp<ID>>),
    MulExp(Box<MulExp<ID>>),
}

#[derive(Debug)]
pub enum MulExp<ID> {
    Product(Box<Unary<ID>>, Box<MulExp<ID>>),
    Quotient(Box<Unary<ID>>, Box<MulExp<ID>>),
    Unary(Box<Unary<ID>>),
}

#[derive(Debug)]
pub enum Unary<ID> {
    Negate(Box<Unary<ID>>),
    Not(Box<Unary<ID>>),
    Primary(Box<Primary<ID>>),
    //Append(Vec<Box<Primary<ID>>>),
}

#[derive(Debug)]
pub enum Primary<ID> {
    Paren(Box<Expression<ID>>),
    IntLit(u32), // I only parse positive integer btw
    StringLit(String),
    BoolLit(bool),
    Variable(ID),
    Unit(),
    //Deref(Expression<Tag, ID>),
    //Tuple(Vec<Expression<Tag, ID>>),
}
