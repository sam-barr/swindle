#![allow(dead_code)]
use crate::ast::*;
use std::collections::HashMap;

// copy and clone might not work in the future
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum SwindleType {
    Int(),
    String(),
    Bool(),
    Unit(),
}

type TypeMap = HashMap<String, SwindleType>;

pub fn type_program(program: Program<String>) -> Option<Program<String>> {
    let mut types = HashMap::new();
    let mut statements = Vec::new();
    for stmt in program.statements {
        if let Some((stmt, _)) = type_statement(&mut types, *stmt) {
            statements.push(stmt);
        } else {
            return None;
        }
    }

    Some(Program { statements })
}

fn type_statement(
    types: &mut TypeMap,
    statement: Statement<String>,
) -> Option<(Box<Statement<String>>, SwindleType)> {
    match statement {
        Statement::Declare(typ, varname, expression) => {
            if types.contains_key(&varname) {
                None
            } else {
                type_expression(types, *expression).and_then(|(e, t)| {
                    if type_matches_swindle_type(typ, t) {
                        types.insert(varname.to_string(), t);
                        Some((Box::new(Statement::Declare(typ, varname, e)), t))
                    } else {
                        None
                    }
                })
            }
        }
        Statement::Write(expression) => {
            type_expression(types, *expression).map(|(e, t)| (Box::new(Statement::Write(e)), t))
        }
        Statement::Writeln(expression) => {
            type_expression(types, *expression).map(|(e, t)| (Box::new(Statement::Writeln(e)), t))
        }
        Statement::Expression(expression) => type_expression(types, *expression)
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
    types: &TypeMap,
    expression: Expression<String>,
) -> Option<(Box<Expression<String>>, SwindleType)> {
    match expression {
        Expression::Assign(varname, expression) => types.get(&varname).and_then(|tv| {
            type_expression(types, *expression).and_then(|(e, te)| {
                if te == *tv {
                    Some((Box::new(Expression::Assign(varname, e)), te))
                } else {
                    None
                }
            })
        }),
        Expression::OrExp(orexp) => {
            type_orexp(types, *orexp).map(|(o, t)| (Box::new(Expression::OrExp(o)), t))
        }
    }
}

fn type_orexp(types: &TypeMap, orexp: OrExp<String>) -> Option<(Box<OrExp<String>>, SwindleType)> {
    match orexp {
        OrExp::Or(andexp, orexp) => type_andexp(types, *andexp).and_then(|(a, ta)| {
            type_orexp(types, *orexp).and_then(|(o, to)| match (ta, to) {
                (SwindleType::Bool(), SwindleType::Bool()) => {
                    Some((Box::new(OrExp::Or(a, o)), SwindleType::Bool()))
                }
                _ => None,
            })
        }),
        OrExp::AndExp(andexp) => {
            type_andexp(types, *andexp).map(|(a, t)| (Box::new(OrExp::AndExp(a)), t))
        }
    }
}

fn type_andexp(
    types: &TypeMap,
    andexp: AndExp<String>,
) -> Option<(Box<AndExp<String>>, SwindleType)> {
    match andexp {
        AndExp::And(compexp, andexp) => type_compexp(types, *compexp).and_then(|(c, tc)| {
            type_andexp(types, *andexp).and_then(|(a, ta)| match (tc, ta) {
                (SwindleType::Bool(), SwindleType::Bool()) => {
                    Some((Box::new(AndExp::And(c, a)), SwindleType::Bool()))
                }
                _ => None,
            })
        }),
        AndExp::CompExp(compexp) => {
            type_compexp(types, *compexp).map(|(c, t)| (Box::new(AndExp::CompExp(c)), t))
        }
    }
}

fn type_compexp(
    types: &TypeMap,
    compexp: CompExp<String>,
) -> Option<(Box<CompExp<String>>, SwindleType)> {
    match compexp {
        CompExp::Leq(addexp1, addexp2) => type_addexp(types, *addexp1).and_then(|(a1, t1)| {
            type_addexp(types, *addexp2).and_then(|(a2, t2)| match (t1, t2) {
                (SwindleType::Int(), SwindleType::Int()) => {
                    Some((Box::new(CompExp::Leq(a1, a2)), SwindleType::Int()))
                }
                _ => None,
            })
        }),
        CompExp::Lt(addexp1, addexp2) => type_addexp(types, *addexp1).and_then(|(a1, t1)| {
            type_addexp(types, *addexp2).and_then(|(a2, t2)| match (t1, t2) {
                (SwindleType::Int(), SwindleType::Int()) => {
                    Some((Box::new(CompExp::Lt(a1, a2)), SwindleType::Int()))
                }
                _ => None,
            })
        }),
        CompExp::Eq(addexp1, addexp2) => type_addexp(types, *addexp1).and_then(|(a1, t1)| {
            type_addexp(types, *addexp2).and_then(|(a2, t2)| {
                if t1 == t2 {
                    Some((Box::new(CompExp::Eq(a1, a2)), SwindleType::Int()))
                } else {
                    None
                }
            })
        }),
        CompExp::Neq(addexp1, addexp2) => type_addexp(types, *addexp1).and_then(|(a1, t1)| {
            type_addexp(types, *addexp2).and_then(|(a2, t2)| {
                if t1 == t2 {
                    Some((Box::new(CompExp::Neq(a1, a2)), SwindleType::Int()))
                } else {
                    None
                }
            })
        }),
        CompExp::Gt(addexp1, addexp2) => type_addexp(types, *addexp1).and_then(|(a1, t1)| {
            type_addexp(types, *addexp2).and_then(|(a2, t2)| match (t1, t2) {
                (SwindleType::Int(), SwindleType::Int()) => {
                    Some((Box::new(CompExp::Gt(a1, a2)), SwindleType::Int()))
                }
                _ => None,
            })
        }),
        CompExp::Geq(addexp1, addexp2) => type_addexp(types, *addexp1).and_then(|(a1, t1)| {
            type_addexp(types, *addexp2).and_then(|(a2, t2)| match (t1, t2) {
                (SwindleType::Int(), SwindleType::Int()) => {
                    Some((Box::new(CompExp::Geq(a1, a2)), SwindleType::Int()))
                }
                _ => None,
            })
        }),
        CompExp::AddExp(addexp) => {
            type_addexp(types, *addexp).map(|(a, t)| (Box::new(CompExp::AddExp(a)), t))
        }
    }
}

fn type_addexp(
    types: &TypeMap,
    addexp: AddExp<String>,
) -> Option<(Box<AddExp<String>>, SwindleType)> {
    match addexp {
        AddExp::Sum(mulexp, addexp) => type_mulexp(types, *mulexp).and_then(|(m, tm)| {
            type_addexp(types, *addexp).and_then(|(a, ta)| match (tm, ta) {
                (SwindleType::Int(), SwindleType::Int()) => {
                    Some((Box::new(AddExp::Sum(m, a)), SwindleType::Int()))
                }
                _ => None,
            })
        }),
        AddExp::Difference(mulexp, addexp) => type_mulexp(types, *mulexp).and_then(|(m, tm)| {
            type_addexp(types, *addexp).and_then(|(a, ta)| match (tm, ta) {
                (SwindleType::Int(), SwindleType::Int()) => {
                    Some((Box::new(AddExp::Difference(m, a)), SwindleType::Int()))
                }
                _ => None,
            })
        }),
        AddExp::MulExp(mulexp) => {
            type_mulexp(types, *mulexp).map(|(m, t)| (Box::new(AddExp::MulExp(m)), t))
        }
    }
}

fn type_mulexp(
    types: &TypeMap,
    mulexp: MulExp<String>,
) -> Option<(Box<MulExp<String>>, SwindleType)> {
    match mulexp {
        MulExp::Product(unary, mulexp) => type_unary(types, *unary).and_then(|(u, tu)| {
            type_mulexp(types, *mulexp).and_then(|(m, tm)| match (tu, tm) {
                (SwindleType::Int(), SwindleType::Int()) => {
                    Some((Box::new(MulExp::Product(u, m)), SwindleType::Int()))
                }
                _ => None,
            })
        }),
        MulExp::Quotient(unary, mulexp) => type_unary(types, *unary).and_then(|(u, tu)| {
            type_mulexp(types, *mulexp).and_then(|(m, tm)| match (tu, tm) {
                (SwindleType::Int(), SwindleType::Int()) => {
                    Some((Box::new(MulExp::Quotient(u, m)), SwindleType::Int()))
                }
                _ => None,
            })
        }),
        MulExp::Unary(unary) => {
            type_unary(types, *unary).map(|(u, t)| (Box::new(MulExp::Unary(u)), t))
        }
    }
}

fn type_unary(types: &TypeMap, unary: Unary<String>) -> Option<(Box<Unary<String>>, SwindleType)> {
    match unary {
        Unary::Negate(unary) => type_unary(types, *unary).and_then(|(u, t)| match t {
            SwindleType::Int() => Some((Box::new(Unary::Negate(u)), t)),
            _ => None,
        }),
        Unary::Not(unary) => type_unary(types, *unary).and_then(|(u, t)| match t {
            SwindleType::Bool() => Some((Box::new(Unary::Negate(u)), t)),
            _ => None,
        }),
        Unary::Primary(primary) => {
            type_primary(types, *primary).map(|(p, t)| (Box::new(Unary::Primary(p)), t))
        }
    }
}

fn type_primary(
    types: &TypeMap,
    primary: Primary<String>,
) -> Option<(Box<Primary<String>>, SwindleType)> {
    match primary {
        Primary::Paren(expression) => {
            type_expression(types, *expression).map(|(e, t)| (Box::new(Primary::Paren(e)), t))
        }
        Primary::IntLit(n) => Some((Box::new(Primary::IntLit(n)), SwindleType::Int())),
        Primary::StringLit(s) => Some((Box::new(Primary::StringLit(s)), SwindleType::String())),
        Primary::BoolLit(b) => Some((Box::new(Primary::BoolLit(b)), SwindleType::Bool())),
        Primary::Variable(varname) => types
            .get(&varname)
            .map(|&t| (Box::new(Primary::Variable(varname)), t)),
        Primary::Unit() => Some((Box::new(Primary::Unit()), SwindleType::Unit())),
    }
}
