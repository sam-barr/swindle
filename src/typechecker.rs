#![allow(dead_code)]
use crate::ast::*;
use crate::error::*;
use crate::parser::Parsed;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Typed {}

impl Tag for Typed {
    type VariableTag = SwindleType;
    type WriteTag = SwindleType;
    type StatementTag = ();
}

// copy and clone might not work in the future
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SwindleType {
    Int(),
    String(),
    Bool(),
    Unit(),
}

type TypeMap = HashMap<String, SwindleType>;
type TyperResult<A> = Result<A, SwindleError>;

fn throw_error<A>(message: String, file_posn: FilePosition) -> TyperResult<A> {
    Err(SwindleError {
        message,
        file_posn,
        error_type: ErrorType::Parser,
    })
}

pub fn type_program(program: Program<Parsed, String>) -> TyperResult<Program<Typed, String>> {
    let mut types = HashMap::new();
    let mut statements = Vec::new();
    for (file_posn, stmt) in program.statements {
        match type_statement(file_posn, &mut types, *stmt) {
            Ok((stmt, _)) => statements.push(((), stmt)),
            Err(e) => return Err(e),
        }
    }

    Ok(Program { statements })
}

fn type_statement(
    file_posn: FilePosition,
    types: &mut TypeMap,
    statement: Statement<Parsed, String>,
) -> TyperResult<(Box<Statement<Typed, String>>, SwindleType)> {
    match statement {
        Statement::Declare(typ, varname, expression) => {
            if types.contains_key(&varname) {
                throw_error("cannot declare a variable twice".to_string(), file_posn)
            } else {
                type_expression(file_posn, types, *expression).and_then(|(e, t)| {
                    if type_matches_swindle_type(typ, t) {
                        types.insert(varname.to_string(), t);
                        Ok((Box::new(Statement::Declare(typ, varname, e)), t))
                    } else {
                        throw_error("bad types for declare".to_string(), file_posn)
                    }
                })
            }
        }
        Statement::Write((), expression) => type_expression(file_posn, types, *expression)
            .map(|(e, t)| (Box::new(Statement::Write(t, e)), t)),
        Statement::Writeln((), expression) => type_expression(file_posn, types, *expression)
            .map(|(e, t)| (Box::new(Statement::Writeln(t, e)), t)),
        Statement::Expression(expression) => type_expression(file_posn, types, *expression)
            .map(|(e, t)| (Box::new(Statement::Expression(e)), t)),
    }
}

fn type_matches_swindle_type(typ: Type, swindle: SwindleType) -> bool {
    match (typ, swindle) {
        (Type::Int(), SwindleType::Int()) => true,
        (Type::String(), SwindleType::String()) => true,
        (Type::Bool(), SwindleType::Bool()) => true,
        (Type::Unit(), SwindleType::Unit()) => true,
        _ => false,
    }
}

fn type_expression(
    file_posn: FilePosition,
    types: &TypeMap,
    expression: Expression<Parsed, String>,
) -> TyperResult<(Box<Expression<Typed, String>>, SwindleType)> {
    match expression {
        Expression::Assign(varname, expression) => match types.get(&varname) {
            Some(tv) => type_expression(file_posn, types, *expression).and_then(|(e, te)| {
                if te == *tv {
                    Ok((Box::new(Expression::Assign(varname, e)), te))
                } else {
                    throw_error("bad types for assign".to_string(), file_posn)
                }
            }),
            None => throw_error(format!("undeclared variable {}", varname), file_posn),
        },
        Expression::OrExp(orexp) => {
            type_orexp(file_posn, types, *orexp).map(|(o, t)| (Box::new(Expression::OrExp(o)), t))
        }
    }
}

fn type_orexp(
    file_posn: FilePosition,
    types: &TypeMap,
    orexp: OrExp<Parsed, String>,
) -> TyperResult<(Box<OrExp<Typed, String>>, SwindleType)> {
    match orexp {
        OrExp::Or(andexp, orexp) => type_andexp(file_posn, types, *andexp).and_then(|(a, ta)| {
            type_orexp(file_posn, types, *orexp).and_then(|(o, to)| match (ta, to) {
                (SwindleType::Bool(), SwindleType::Bool()) => {
                    Ok((Box::new(OrExp::Or(a, o)), SwindleType::Bool()))
                }
                _ => throw_error("bad types for or".to_string(), file_posn),
            })
        }),
        OrExp::AndExp(andexp) => {
            type_andexp(file_posn, types, *andexp).map(|(a, t)| (Box::new(OrExp::AndExp(a)), t))
        }
    }
}

fn type_andexp(
    file_posn: FilePosition,
    types: &TypeMap,
    andexp: AndExp<Parsed, String>,
) -> TyperResult<(Box<AndExp<Typed, String>>, SwindleType)> {
    match andexp {
        AndExp::And(compexp, andexp) => {
            type_compexp(file_posn, types, *compexp).and_then(|(c, tc)| {
                type_andexp(file_posn, types, *andexp).and_then(|(a, ta)| match (tc, ta) {
                    (SwindleType::Bool(), SwindleType::Bool()) => {
                        Ok((Box::new(AndExp::And(c, a)), SwindleType::Bool()))
                    }
                    _ => throw_error("bad types for and".to_string(), file_posn),
                })
            })
        }
        AndExp::CompExp(compexp) => {
            type_compexp(file_posn, types, *compexp).map(|(c, t)| (Box::new(AndExp::CompExp(c)), t))
        }
    }
}

fn type_compexp(
    file_posn: FilePosition,
    types: &TypeMap,
    compexp: CompExp<Parsed, String>,
) -> TyperResult<(Box<CompExp<Typed, String>>, SwindleType)> {
    match compexp {
        CompExp::Comp(compop, addexp1, addexp2) => type_addexp(file_posn, types, *addexp1)
            .and_then(|(a1, t1)| {
                type_addexp(file_posn, types, *addexp2).and_then(|(a2, t2)| {
                    let result = (Box::new(CompExp::Comp(compop, a1, a2)), SwindleType::Bool());
                    match compop {
                        CompOp::Eq | CompOp::Neq => {
                            if t1 == t2 {
                                Ok(result)
                            } else {
                                throw_error(
                                    "can't check equality for non-matching types".to_string(),
                                    file_posn,
                                )
                            }
                        }
                        _ => match (t1, t2) {
                            (SwindleType::Int(), SwindleType::Int()) => Ok(result),
                            _ => throw_error("can only compare integers".to_string(), file_posn),
                        },
                    }
                })
            }),
        CompExp::AddExp(addexp) => {
            type_addexp(file_posn, types, *addexp).map(|(a, t)| (Box::new(CompExp::AddExp(a)), t))
        }
    }
}

fn type_addexp(
    file_posn: FilePosition,
    types: &TypeMap,
    addexp: AddExp<Parsed, String>,
) -> TyperResult<(Box<AddExp<Typed, String>>, SwindleType)> {
    match addexp {
        AddExp::Add(addop, mulexp, addexp) => {
            type_mulexp(file_posn, types, *mulexp).and_then(|(m, tm)| {
                type_addexp(file_posn, types, *addexp).and_then(|(a, ta)| match (tm, ta) {
                    (SwindleType::Int(), SwindleType::Int()) => {
                        Ok((Box::new(AddExp::Add(addop, m, a)), SwindleType::Int()))
                    }
                    _ => throw_error("bad types for addition".to_string(), file_posn),
                })
            })
        }
        AddExp::MulExp(mulexp) => {
            type_mulexp(file_posn, types, *mulexp).map(|(m, t)| (Box::new(AddExp::MulExp(m)), t))
        }
    }
}

fn type_mulexp(
    file_posn: FilePosition,
    types: &TypeMap,
    mulexp: MulExp<Parsed, String>,
) -> TyperResult<(Box<MulExp<Typed, String>>, SwindleType)> {
    match mulexp {
        MulExp::Mul(mulop, unary, mulexp) => {
            type_unary(file_posn, types, *unary).and_then(|(u, tu)| {
                type_mulexp(file_posn, types, *mulexp).and_then(|(m, tm)| match (tu, tm) {
                    (SwindleType::Int(), SwindleType::Int()) => {
                        Ok((Box::new(MulExp::Mul(mulop, u, m)), SwindleType::Int()))
                    }
                    _ => throw_error("bad types for multiplication".to_string(), file_posn),
                })
            })
        }
        MulExp::Unary(unary) => {
            type_unary(file_posn, types, *unary).map(|(u, t)| (Box::new(MulExp::Unary(u)), t))
        }
    }
}

fn type_unary(
    file_posn: FilePosition,
    types: &TypeMap,
    unary: Unary<Parsed, String>,
) -> TyperResult<(Box<Unary<Typed, String>>, SwindleType)> {
    match unary {
        Unary::Negate(unary) => type_unary(file_posn, types, *unary).and_then(|(u, t)| match t {
            SwindleType::Int() => Ok((Box::new(Unary::Negate(u)), t)),
            _ => throw_error("can only negate integers".to_string(), file_posn),
        }),
        Unary::Not(unary) => type_unary(file_posn, types, *unary).and_then(|(u, t)| match t {
            SwindleType::Bool() => Ok((Box::new(Unary::Negate(u)), t)),
            _ => throw_error("can only not a boolean".to_string(), file_posn),
        }),
        Unary::Primary(primary) => {
            type_primary(file_posn, types, *primary).map(|(p, t)| (Box::new(Unary::Primary(p)), t))
        }
    }
}

fn type_primary(
    file_posn: FilePosition,
    types: &TypeMap,
    primary: Primary<Parsed, String>,
) -> TyperResult<(Box<Primary<Typed, String>>, SwindleType)> {
    match primary {
        Primary::Paren(expression) => type_expression(file_posn, types, *expression)
            .map(|(e, t)| (Box::new(Primary::Paren(e)), t)),
        Primary::IntLit(n) => Ok((Box::new(Primary::IntLit(n)), SwindleType::Int())),
        Primary::StringLit(s) => Ok((Box::new(Primary::StringLit(s)), SwindleType::String())),
        Primary::BoolLit(b) => Ok((Box::new(Primary::BoolLit(b)), SwindleType::Bool())),
        Primary::Variable((), varname) => match types.get(&varname) {
            Some(typ) => Ok((Box::new(Primary::Variable(*typ, varname)), *typ)),
            None => throw_error(format!("undeclared variable: {}", varname), file_posn),
        },
        Primary::Unit() => Ok((Box::new(Primary::Unit()), SwindleType::Unit())),
    }
}
