#![allow(dead_code)]
use crate::ast::*;
use crate::typechecker::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct PCG {}

impl Tag for PCG {
    type TypeTag = SwindleType;
    type StatementTag = ();
    type DeclareTag = ();
    type VariableID = usize;
    type StringID = usize;
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
            tag: (),
            statement: preprocess_statement(&mut state, tagged_stmt.statement),
        })
    }
    (Program { statements }, state.variables, state.strings)
}

fn preprocess_statement(state: &mut PCGState, statement: Statement<Typed>) -> Statement<PCG> {
    match statement {
        Statement::Declare(typ, varname, expression) => Statement::Declare(
            (),
            state.add_variable(varname, typ),
            preprocess_expression(state, *expression),
        ),
        Statement::Write(typ, newline, expression) => {
            Statement::Write(typ, newline, preprocess_expression(state, *expression))
        }
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
        Expression::Assign(varname, expression) => Expression::Assign(
            state.get_variable(varname),
            preprocess_expression(state, *expression),
        ),
        Expression::OrExp(orexp) => Expression::OrExp(preprocess_orexp(state, *orexp)),
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
        CompExp::Comp(op, addexp1, addexp2) => CompExp::Comp(
            op,
            preprocess_addexp(state, *addexp1),
            preprocess_addexp(state, *addexp2),
        ),
        CompExp::AddExp(addexp) => CompExp::AddExp(preprocess_addexp(state, *addexp)),
    })
}

fn preprocess_addexp(state: &mut PCGState, addexp: AddExp<Typed>) -> Box<AddExp<PCG>> {
    Box::new(match addexp {
        AddExp::Add(op, mulexp, addexp) => AddExp::Add(
            op,
            preprocess_mulexp(state, *mulexp),
            preprocess_addexp(state, *addexp),
        ),
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
        Primary::WhileExp(whileexp) => Primary::WhileExp(preprocess_whileexp(state, whileexp)),
        Primary::Unit => Primary::Unit,
    }
}

fn preprocess_whileexp(state: &mut PCGState, whileexp: WhileExp<Typed>) -> WhileExp<PCG> {
    let tag = whileexp.tag;
    let cond = preprocess_expression(state, *whileexp.cond);
    let body = preprocess_body(state, whileexp.body);
    let els = preprocess_body(state, whileexp.els);
    WhileExp {
        tag,
        cond,
        body,
        els,
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
    for stmt in body.statements {
        statements.push(preprocess_statement(state, stmt));
    }

    Body { statements }
}
