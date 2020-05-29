use crate::ast::*;
use crate::typechecker::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PCG {}

impl Tag for PCG {
    type TypeTag = SwindleType;
    type StatementTag = bool;
    type DeclareTag = SwindleType;
    type VariableID = usize;
    type StringID = usize;
    type BuiltinID = Builtin<PCG>;
}

struct PCGState {
    variable_map: HashMap<String, usize>,
    string_map: HashMap<String, usize>,
    variables: Vec<SwindleType>,
    strings: Vec<String>,
}

impl PCGState {
    fn new() -> Self {
        PCGState {
            variable_map: HashMap::new(),
            string_map: HashMap::new(),
            variables: Vec::new(),
            strings: Vec::new(),
        }
    }

    fn add_variable(&mut self, varname: String, typ: SwindleType) -> usize {
        let id = self.variables.len();
        self.variable_map.insert(varname, id);
        self.variables.push(typ);
        id
    }

    fn get_variable(&self, varname: String) -> usize {
        *self.variable_map.get(&varname).unwrap()
    }

    fn add_string(&mut self, string: String) -> usize {
        match self.string_map.get(&string) {
            Some(&id) => id,
            None => {
                let id = self.strings.len();
                self.string_map.insert(string.clone(), id);
                self.strings.push(string);
                id
            }
        }
    }
}

pub fn preprocess_program(
    program: Program<Typed>,
) -> (Program<PCG>, Vec<SwindleType>, Vec<String>) {
    let mut state = PCGState::new();
    let mut statements = Vec::new();
    for tagged_stmt in program.statements {
        statements.push(TaggedStatement {
            tag: match tagged_stmt.tag {
                SwindleType::List(_) | SwindleType::String => true,
                _ => false,
            },
            statement: preprocess_statement(&mut state, tagged_stmt.statement),
        })
    }
    (Program { statements }, state.variables, state.strings)
}

fn preprocess_statement(state: &mut PCGState, statement: Statement<Typed>) -> Statement<PCG> {
    match statement {
        Statement::Declare(typ, varname, expression) => Statement::Declare(
            typ.clone(),
            state.add_variable(varname, typ),
            preprocess_expression(state, *expression),
        ),
        Statement::Break => Statement::Break,
        Statement::Continue => Statement::Continue,
        Statement::Expression(expression) => {
            Statement::Expression(preprocess_expression(state, *expression))
        }
    }
}

fn preprocess_expression(
    state: &mut PCGState,
    expression: Expression<Typed>,
) -> Box<Expression<PCG>> {
    Box::new(match expression {
        Expression::Assign(typ, lvalue, expression) => Expression::Assign(
            typ,
            preprocess_lvalue(state, *lvalue),
            preprocess_expression(state, *expression),
        ),
        Expression::OrExp(orexp) => Expression::OrExp(preprocess_orexp(state, *orexp)),
    })
}

fn preprocess_lvalue(state: &mut PCGState, lvalue: LValue<Typed>) -> Box<LValue<PCG>> {
    Box::new(match lvalue {
        LValue::Variable(varname) => LValue::Variable(state.get_variable(varname)),
        LValue::Index(lvalue, index) => LValue::Index(
            preprocess_lvalue(state, *lvalue),
            preprocess_expression(state, *index),
        ),
    })
}

fn preprocess_orexp(state: &mut PCGState, orexp: OrExp<Typed>) -> Box<OrExp<PCG>> {
    Box::new(match orexp {
        OrExp::Or(andexp, orexp) => OrExp::Or(
            preprocess_andexp(state, *andexp),
            preprocess_orexp(state, *orexp),
        ),
        OrExp::AndExp(andexp) => OrExp::AndExp(preprocess_andexp(state, *andexp)),
    })
}

fn preprocess_andexp(state: &mut PCGState, andexp: AndExp<Typed>) -> Box<AndExp<PCG>> {
    Box::new(match andexp {
        AndExp::And(compexp, andexp) => AndExp::And(
            preprocess_compexp(state, *compexp),
            preprocess_andexp(state, *andexp),
        ),
        AndExp::CompExp(compexp) => AndExp::CompExp(preprocess_compexp(state, *compexp)),
    })
}

fn preprocess_compexp(state: &mut PCGState, compexp: CompExp<Typed>) -> Box<CompExp<PCG>> {
    Box::new(match compexp {
        CompExp::Comp(op, addexp1, addexp2) => {
            let op = match op {
                CompOp::Leq => CompOp::Leq,
                CompOp::Lt => CompOp::Lt,
                CompOp::Eq(t) => CompOp::Eq(t),
            };
            CompExp::Comp(
                op,
                preprocess_addexp(state, *addexp1),
                preprocess_addexp(state, *addexp2),
            )
        }
        CompExp::AddExp(addexp) => CompExp::AddExp(preprocess_addexp(state, *addexp)),
    })
}

fn preprocess_addexp(state: &mut PCGState, addexp: AddExp<Typed>) -> Box<AddExp<PCG>> {
    Box::new(match addexp {
        AddExp::Add(op, mulexp, addexp) => {
            let op = match op {
                AddOp::Sum(t) => AddOp::Sum(t),
                AddOp::Difference => AddOp::Difference,
            };
            AddExp::Add(
                op,
                preprocess_mulexp(state, *mulexp),
                preprocess_addexp(state, *addexp),
            )
        }
        AddExp::MulExp(mulexp) => AddExp::MulExp(preprocess_mulexp(state, *mulexp)),
    })
}

fn preprocess_mulexp(state: &mut PCGState, mulexp: MulExp<Typed>) -> Box<MulExp<PCG>> {
    Box::new(match mulexp {
        MulExp::Mul(op, unary, mulexp) => MulExp::Mul(
            op,
            preprocess_unary(state, *unary),
            preprocess_mulexp(state, *mulexp),
        ),
        MulExp::Unary(unary) => MulExp::Unary(preprocess_unary(state, *unary)),
    })
}

fn preprocess_unary(state: &mut PCGState, unary: Unary<Typed>) -> Box<Unary<PCG>> {
    Box::new(match unary {
        Unary::Negate(unary) => Unary::Negate(preprocess_unary(state, *unary)),
        Unary::Not(unary) => Unary::Not(preprocess_unary(state, *unary)),
        Unary::Primary(primary) => Unary::Primary(Box::new(preprocess_primary(state, *primary))),
    })
}

fn preprocess_primary(state: &mut PCGState, primary: Primary<Typed>) -> Primary<PCG> {
    match primary {
        Primary::Paren(e) => Primary::Paren(preprocess_expression(state, *e)),
        Primary::IntLit(n) => Primary::IntLit(n),
        Primary::StringLit(s) => Primary::StringLit(state.add_string(s)),
        Primary::BoolLit(b) => Primary::BoolLit(b),
        Primary::Variable(v) => Primary::Variable(state.get_variable(v)),
        Primary::IfExp(ifexp) => Primary::IfExp(preprocess_ifexp(state, ifexp)),
        Primary::ForExp(forexp) => Primary::ForExp(preprocess_forexp(state, forexp)),
        Primary::StatementExp(body) => Primary::StatementExp(preprocess_body(state, body)),
        Primary::Index(typ, list, index) => Primary::Index(
            typ,
            Box::new(preprocess_primary(state, *list)),
            preprocess_expression(state, *index),
        ),
        Primary::Builtin(builtin) => Primary::Builtin(preprocess_builtin(state, builtin)),
        Primary::List(typ, items) => {
            let mut new_items = Vec::new();
            for item in items {
                new_items.push(*preprocess_expression(state, item));
            }
            Primary::List(typ, new_items)
        }
        Primary::Function(_) => unimplemented!(),
        Primary::FunCall(_, _) => unimplemented!(),
        Primary::Unit => Primary::Unit,
    }
}

fn preprocess_builtin(state: &mut PCGState, builtin: Builtin<Typed>) -> Builtin<PCG> {
    match builtin {
        Builtin::Length(typ, e) => Builtin::Length(typ, preprocess_expression(state, *e)),
        Builtin::Write(newline, args) => {
            let mut new_args = Vec::new();
            for (arg, typ) in args {
                new_args.push((*preprocess_expression(state, arg), typ));
            }
            Builtin::Write(newline, new_args)
        }
    }
}

fn preprocess_forexp(state: &mut PCGState, forexp: ForExp<Typed>) -> ForExp<PCG> {
    let tag = forexp.tag;
    let init = Box::new(preprocess_statement(state, *forexp.init));
    let cond = preprocess_expression(state, *forexp.cond);
    let update = preprocess_expression(state, *forexp.update);
    let body = preprocess_body(state, forexp.body);
    ForExp {
        tag,
        init,
        cond,
        update,
        body,
    }
}

fn preprocess_ifexp(state: &mut PCGState, ifexp: IfExp<Typed>) -> IfExp<PCG> {
    let tag = ifexp.tag;
    let cond = preprocess_expression(state, *ifexp.cond);
    let body = preprocess_body(state, ifexp.body);
    let mut elifs = Vec::new();
    for elif in ifexp.elifs {
        elifs.push(preprocess_elif(state, elif));
    }
    let els = preprocess_body(state, ifexp.els);
    IfExp {
        tag,
        cond,
        body,
        elifs,
        els,
    }
}

fn preprocess_elif(state: &mut PCGState, elif: Elif<Typed>) -> Elif<PCG> {
    Elif {
        cond: preprocess_expression(state, *elif.cond),
        body: preprocess_body(state, elif.body),
    }
}

fn preprocess_body(state: &mut PCGState, body: Body<Typed>) -> Body<PCG> {
    let mut statements = Vec::new();
    for tagged_stmt in body.statements {
        statements.push(TaggedStatement::new(
            tagged_stmt.tag == SwindleType::String,
            preprocess_statement(state, tagged_stmt.statement),
        ));
    }

    if let Some(tagged_stmt) = statements.last_mut() {
        tagged_stmt.tag &= true;
    }

    Body { statements }
}
