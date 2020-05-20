use crate::ast::*;
use crate::error::*;
use std::collections::HashMap;
use std::default::Default;

// TODO: Dissallow statements of type "string" which have no side affects

#[derive(Debug)]
pub struct Typed {}

impl Tag for Typed {
    type TypeTag = SwindleType;
    type StatementTag = SwindleType;
    type DeclareTag = SwindleType;
    type VariableID = String;
    type StringID = String;
}

// copy and clone might not work in the future
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SwindleType {
    Int,
    String,
    Bool,
    Unit,
}

type TyperResult<A> = Result<A, SwindleError>;

#[derive(Debug, Clone)]
struct TyperState {
    types: HashMap<String, SwindleType>,
    file_posn: FilePosition,
    in_loop: bool,
}

impl TyperState {
    fn new() -> Self {
        TyperState {
            types: HashMap::new(),
            file_posn: Default::default(),
            in_loop: false,
        }
    }

    fn get(&self, varname: &str) -> Option<SwindleType> {
        self.types.get(varname).cloned()
    }

    fn insert(&mut self, varname: String, typ: SwindleType) {
        self.types.insert(varname, typ);
    }
}

fn throw_error<A>(message: String, file_posn: FilePosition) -> TyperResult<A> {
    Err(SwindleError {
        message,
        file_posn,
        error_type: ErrorType::Typechecker,
    })
}

pub fn type_program(program: Program<Parsed>) -> TyperResult<Program<Typed>> {
    let mut state = TyperState::new();
    let mut statements = Vec::new();
    for tagged_stmt in program.statements {
        state.file_posn = tagged_stmt.tag;
        match type_statement(&mut state, tagged_stmt.statement) {
            Ok((stmt, t)) => statements.push(TaggedStatement::new(t, stmt)),
            Err(e) => return Err(e),
        }
    }

    Ok(Program { statements })
}

fn type_statement(
    state: &mut TyperState,
    statement: Statement<Parsed>,
) -> TyperResult<(Statement<Typed>, SwindleType)> {
    match statement {
        Statement::Declare(typ, varname, expression) => {
            if state.get(&varname).is_some() {
                throw_error(
                    "cannot declare a variable twice".to_string(),
                    state.file_posn,
                )
            } else {
                type_expression(state, *expression).and_then(|(e, t)| {
                    if type_matches_swindle_type(&typ, &t) {
                        state.insert(varname.to_string(), t.clone());
                        Ok((Statement::Declare(t, varname, e), SwindleType::Unit))
                    } else {
                        throw_error("bad types for declare".to_string(), state.file_posn)
                    }
                })
            }
        }
        Statement::Write((), newline, expression) => type_expression(state, *expression)
            .map(|(e, t)| (Statement::Write(t, newline, e), SwindleType::Unit)),
        Statement::Break => {
            if state.in_loop {
                Ok((Statement::Break, SwindleType::Unit))
            } else {
                throw_error(
                    "can only use a break statement in a loop".to_string(),
                    state.file_posn,
                )
            }
        }
        Statement::Continue => {
            if state.in_loop {
                Ok((Statement::Continue, SwindleType::Unit))
            } else {
                throw_error(
                    "can only use a continue statement in a loop".to_string(),
                    state.file_posn,
                )
            }
        }
        Statement::Expression(expression) => {
            type_expression(state, *expression).map(|(e, t)| (Statement::Expression(e), t))
        }
    }
}

fn type_matches_swindle_type(typ: &Type, swindle: &SwindleType) -> bool {
    match (typ, swindle) {
        (Type::Int, SwindleType::Int) => true,
        (Type::String, SwindleType::String) => true,
        (Type::Bool, SwindleType::Bool) => true,
        (Type::Unit, SwindleType::Unit) => true,
        _ => false,
    }
}

fn type_expression(
    state: &mut TyperState,
    expression: Expression<Parsed>,
) -> TyperResult<(Box<Expression<Typed>>, SwindleType)> {
    match expression {
        Expression::Assign((), varname, expression) => match state.get(&varname) {
            Some(tv) => type_expression(state, *expression).and_then(|(e, te)| {
                if te == tv {
                    Ok((Box::new(Expression::Assign(te.clone(), varname, e)), te))
                } else {
                    throw_error("bad types for assign".to_string(), state.file_posn)
                }
            }),
            None => throw_error(format!("undeclared variable {}", varname), state.file_posn),
        },
        Expression::OrExp(orexp) => {
            type_orexp(state, *orexp).map(|(o, t)| (Box::new(Expression::OrExp(o)), t))
        }
    }
}

fn type_whileexp(
    state: &mut TyperState,
    whileexp: WhileExp<Parsed>,
) -> TyperResult<(WhileExp<Typed>, SwindleType)> {
    let cond = match type_expression(state, *whileexp.cond) {
        Ok((cond, SwindleType::Bool)) => cond,
        Err(e) => return Err(e),
        _ => {
            return throw_error(
                "while condition must be a bool".to_string(),
                state.file_posn,
            )
        }
    };

    let (body, body_ty) = match type_body(state, whileexp.body) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    Ok((
        WhileExp {
            tag: body_ty,
            cond,
            body,
        },
        SwindleType::Unit,
    ))
}

fn type_ifexp(
    state: &mut TyperState,
    ifexp: IfExp<Parsed>,
) -> TyperResult<(IfExp<Typed>, SwindleType)> {
    let cond = match type_expression(state, *ifexp.cond) {
        Ok((cond, SwindleType::Bool)) => cond,
        Err(e) => return Err(e),
        _ => return throw_error("if condition must be bool".to_string(), state.file_posn),
    };

    let (body, iftype) = match type_body(state, ifexp.body) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let mut elifs = Vec::new();
    for elif in ifexp.elifs {
        elifs.push(match type_elif(state, elif) {
            Ok((elif, t)) => {
                if t == iftype {
                    elif
                } else {
                    return throw_error(
                        "type of elif body doesn't match if body".to_string(),
                        state.file_posn,
                    );
                }
            }
            Err(e) => return Err(e),
        })
    }

    let els = match type_body(state, ifexp.els) {
        Ok((els, t)) => {
            if t == iftype {
                els
            } else {
                return throw_error(
                    "type of else body doesn't match if body".to_string(),
                    state.file_posn,
                );
            }
        }
        Err(e) => return Err(e),
    };

    Ok((
        IfExp {
            tag: iftype.clone(),
            cond,
            body,
            elifs,
            els,
        },
        iftype,
    ))
}

fn type_elif(
    state: &mut TyperState,
    elif: Elif<Parsed>,
) -> TyperResult<(Elif<Typed>, SwindleType)> {
    let cond = match type_expression(state, *elif.cond) {
        Ok((cond, SwindleType::Bool)) => cond,
        Err(e) => return Err(e),
        _ => return throw_error("if condition must be bool".to_string(), state.file_posn),
    };

    let (body, typ) = match type_body(state, elif.body) {
        Ok((body, if_type)) => (body, if_type),
        Err(e) => return Err(e),
    };

    Ok((Elif { cond, body }, typ))
}

fn type_body(
    state: &mut TyperState,
    body: Body<Parsed>,
) -> TyperResult<(Body<Typed>, SwindleType)> {
    let mut state = state.clone();
    let mut body_type = SwindleType::Unit;
    let mut statements = Vec::new();
    let mut have_jumped = false; // keep track of whether we've seen 'break' or 'continue'

    for tagged_stmt in body.statements {
        state.file_posn = tagged_stmt.tag;
        match type_statement(&mut state, tagged_stmt.statement) {
            Ok((stmt, t)) => {
                if have_jumped {
                    return throw_error("unreachable statement".to_string(), state.file_posn);
                }
                if let Statement::Break | Statement::Continue = stmt {
                    have_jumped = true;
                }
                body_type = t.clone();
                statements.push(TaggedStatement::new(t, stmt));
            }
            Err(e) => return Err(e),
        }
    }

    Ok((Body { statements }, body_type))
}

fn type_orexp(
    state: &mut TyperState,
    orexp: OrExp<Parsed>,
) -> TyperResult<(Box<OrExp<Typed>>, SwindleType)> {
    match orexp {
        OrExp::Or(andexp, orexp) => type_andexp(state, *andexp).and_then(|(a, ta)| {
            type_orexp(state, *orexp).and_then(|(o, to)| match (ta, to) {
                (SwindleType::Bool, SwindleType::Bool) => {
                    Ok((Box::new(OrExp::Or(a, o)), SwindleType::Bool))
                }
                _ => throw_error("bad types for or".to_string(), state.file_posn),
            })
        }),
        OrExp::AndExp(andexp) => {
            type_andexp(state, *andexp).map(|(a, t)| (Box::new(OrExp::AndExp(a)), t))
        }
    }
}

fn type_andexp(
    state: &mut TyperState,
    andexp: AndExp<Parsed>,
) -> TyperResult<(Box<AndExp<Typed>>, SwindleType)> {
    match andexp {
        AndExp::And(compexp, andexp) => type_compexp(state, *compexp).and_then(|(c, tc)| {
            type_andexp(state, *andexp).and_then(|(a, ta)| match (tc, ta) {
                (SwindleType::Bool, SwindleType::Bool) => {
                    Ok((Box::new(AndExp::And(c, a)), SwindleType::Bool))
                }
                _ => throw_error("bad types for and".to_string(), state.file_posn),
            })
        }),
        AndExp::CompExp(compexp) => {
            type_compexp(state, *compexp).map(|(c, t)| (Box::new(AndExp::CompExp(c)), t))
        }
    }
}

fn type_compexp(
    state: &mut TyperState,
    compexp: CompExp<Parsed>,
) -> TyperResult<(Box<CompExp<Typed>>, SwindleType)> {
    match compexp {
        CompExp::Comp(compop, addexp1, addexp2) => {
            type_addexp(state, *addexp1).and_then(|(a1, t1)| {
                type_addexp(state, *addexp2).and_then(|(a2, t2)| {
                    let op = match compop {
                        CompOp::Leq => CompOp::Leq,
                        CompOp::Lt => CompOp::Lt,
                        CompOp::Eq(()) => CompOp::Eq(t1.clone()),
                    };

                    let result = (Box::new(CompExp::Comp(op, a1, a2)), SwindleType::Bool);
                    match compop {
                        CompOp::Eq(_) => {
                            if t1 == t2 {
                                Ok(result)
                            } else {
                                throw_error(
                                    "can't check equality for non-matching types".to_string(),
                                    state.file_posn,
                                )
                            }
                        }
                        _ => match (t1, t2) {
                            (SwindleType::Int, SwindleType::Int) => Ok(result),
                            _ => throw_error(
                                "can only compare integers".to_string(),
                                state.file_posn,
                            ),
                        },
                    }
                })
            })
        }
        CompExp::AddExp(addexp) => {
            type_addexp(state, *addexp).map(|(a, t)| (Box::new(CompExp::AddExp(a)), t))
        }
    }
}

fn type_addexp(
    state: &mut TyperState,
    addexp: AddExp<Parsed>,
) -> TyperResult<(Box<AddExp<Typed>>, SwindleType)> {
    match addexp {
        AddExp::Add(addop, mulexp, addexp) => type_mulexp(state, *mulexp).and_then(|(m, tm)| {
            type_addexp(state, *addexp).and_then(|(a, ta)| match (addop, tm, ta) {
                (AddOp::Sum(()), SwindleType::String, SwindleType::String) => Ok((
                    Box::new(AddExp::Add(AddOp::Sum(SwindleType::String), m, a)),
                    SwindleType::String,
                )),
                (AddOp::Sum(()), SwindleType::Int, SwindleType::Int) => Ok((
                    Box::new(AddExp::Add(AddOp::Sum(SwindleType::Int), m, a)),
                    SwindleType::Int,
                )),
                (AddOp::Difference, SwindleType::Int, SwindleType::Int) => Ok((
                    Box::new(AddExp::Add(AddOp::Difference, m, a)),
                    SwindleType::Int,
                )),
                _ => throw_error("bad types for addition".to_string(), state.file_posn),
            })
        }),
        AddExp::MulExp(mulexp) => {
            type_mulexp(state, *mulexp).map(|(m, t)| (Box::new(AddExp::MulExp(m)), t))
        }
    }
}

fn type_mulexp(
    state: &mut TyperState,
    mulexp: MulExp<Parsed>,
) -> TyperResult<(Box<MulExp<Typed>>, SwindleType)> {
    match mulexp {
        MulExp::Mul(mulop, unary, mulexp) => type_unary(state, *unary).and_then(|(u, tu)| {
            type_mulexp(state, *mulexp).and_then(|(m, tm)| match (tu, tm) {
                (SwindleType::Int, SwindleType::Int) => {
                    Ok((Box::new(MulExp::Mul(mulop, u, m)), SwindleType::Int))
                }
                _ => throw_error("bad types for multiplication".to_string(), state.file_posn),
            })
        }),
        MulExp::Unary(unary) => {
            type_unary(state, *unary).map(|(u, t)| (Box::new(MulExp::Unary(u)), t))
        }
    }
}

fn type_unary(
    state: &mut TyperState,
    unary: Unary<Parsed>,
) -> TyperResult<(Box<Unary<Typed>>, SwindleType)> {
    match unary {
        Unary::Negate(unary) => type_unary(state, *unary).and_then(|(u, t)| match t {
            SwindleType::Int => Ok((Box::new(Unary::Negate(u)), t)),
            _ => throw_error("can only negate integers".to_string(), state.file_posn),
        }),
        Unary::Not(unary) => type_unary(state, *unary).and_then(|(u, t)| match t {
            SwindleType::Bool => Ok((Box::new(Unary::Negate(u)), t)),
            _ => throw_error("can only not a boolean".to_string(), state.file_posn),
        }),
        Unary::Primary(primary) => {
            type_primary(state, *primary).map(|(p, t)| (Box::new(Unary::Primary(p)), t))
        }
    }
}

fn type_primary(
    state: &mut TyperState,
    primary: Primary<Parsed>,
) -> TyperResult<(Box<Primary<Typed>>, SwindleType)> {
    match primary {
        Primary::Paren(expression) => {
            type_expression(state, *expression).map(|(e, t)| (Box::new(Primary::Paren(e)), t))
        }
        Primary::IntLit(n) => Ok((Box::new(Primary::IntLit(n)), SwindleType::Int)),
        Primary::StringLit(s) => Ok((Box::new(Primary::StringLit(s)), SwindleType::String)),
        Primary::BoolLit(b) => Ok((Box::new(Primary::BoolLit(b)), SwindleType::Bool)),
        Primary::Variable(varname) => match state.get(&varname) {
            Some(typ) => Ok((Box::new(Primary::Variable(varname)), typ)),
            None => throw_error(format!("undeclared variable: {}", varname), state.file_posn),
        },
        Primary::Unit => Ok((Box::new(Primary::Unit), SwindleType::Unit)),
        Primary::IfExp(ifexp) => {
            type_ifexp(state, ifexp).map(|(i, t)| (Box::new(Primary::IfExp(i)), t))
        }
        Primary::WhileExp(whileexp) => {
            let was_in_loop = state.in_loop;
            state.in_loop = true;
            let result =
                type_whileexp(state, whileexp).map(|(i, t)| (Box::new(Primary::WhileExp(i)), t));
            state.in_loop = was_in_loop;
            result
        }
        Primary::StatementExp(body) => {
            type_body(state, body).map(|(body, ty)| (Box::new(Primary::StatementExp(body)), ty))
        }
        Primary::Index((), list, index) => {
            let (list, list_type, result_type) = match type_primary(state, *list) {
                Ok((list, SwindleType::String)) => (list, SwindleType::String, SwindleType::String),
                Err(e) => return Err(e),
                _ => return throw_error("bad type for list".to_string(), state.file_posn),
            };

            let index = match type_expression(state, *index) {
                Ok((index, SwindleType::Int)) => index,
                Err(e) => return Err(e),
                _ => return throw_error("bad type for list index".to_string(), state.file_posn),
            };

            Ok((
                Box::new(Primary::Index(list_type, list, index)),
                result_type,
            ))
        }
    }
}
