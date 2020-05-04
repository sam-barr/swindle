#![allow(dead_code)]
use crate::ast::*;
use crate::typechecker::Typed;
use std::boxed::Box;
use std::collections::HashMap;

// TODO: this could be a lot smarter

type NameTable = HashMap<String, UID>;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct UID(u32);

impl UID {
    pub fn new() -> Self {
        UID(0)
    }

    pub fn inc(&mut self) {
        self.0 += 1;
    }
}

pub fn rename_program(program: Program<Typed, String>) -> (Program<Typed, UID>, u32) {
    let mut next_id = UID::new();
    let mut name_table = HashMap::new();
    let mut statements = Vec::new();

    for ((), stmt) in program.statements {
        statements.push(((), rename_statement(&mut next_id, &mut name_table, *stmt)));
    }

    (Program { statements }, next_id.0)
}

fn rename_statement(
    next_id: &mut UID,
    name_table: &mut NameTable,
    statement: Statement<Typed, String>,
) -> Box<Statement<Typed, UID>> {
    Box::new(match statement {
        Statement::Declare(typ, varname, expression) => {
            let expression = rename_expression(name_table, *expression);
            name_table.insert(varname, *next_id);
            let new_name = *next_id;
            next_id.inc();
            Statement::Declare(typ, new_name, expression)
        }
        Statement::Write(tag, expression) => {
            Statement::Write(tag, rename_expression(name_table, *expression))
        }
        Statement::Writeln(tag, expression) => {
            Statement::Writeln(tag, rename_expression(name_table, *expression))
        }
        Statement::Expression(expression) => {
            Statement::Expression(rename_expression(name_table, *expression))
        }
    })
}

fn rename_expression(
    name_table: &NameTable,
    expression: Expression<Typed, String>,
) -> Box<Expression<Typed, UID>> {
    Box::new(match expression {
        Expression::Assign(varname, expression) => Expression::Assign(
            *name_table.get(&varname).unwrap(),
            rename_expression(name_table, *expression),
        ),
        Expression::OrExp(orexp) => Expression::OrExp(rename_orexp(name_table, *orexp)),
    })
}

fn rename_orexp(name_table: &NameTable, orexp: OrExp<Typed, String>) -> Box<OrExp<Typed, UID>> {
    Box::new(match orexp {
        OrExp::Or(andexp, orexp) => OrExp::Or(
            rename_andexp(name_table, *andexp),
            rename_orexp(name_table, *orexp),
        ),
        OrExp::AndExp(andexp) => OrExp::AndExp(rename_andexp(name_table, *andexp)),
    })
}

fn rename_andexp(name_table: &NameTable, andexp: AndExp<Typed, String>) -> Box<AndExp<Typed, UID>> {
    Box::new(match andexp {
        AndExp::And(compexp, andexp) => AndExp::And(
            rename_compexp(name_table, *compexp),
            rename_andexp(name_table, *andexp),
        ),
        AndExp::CompExp(compexp) => AndExp::CompExp(rename_compexp(name_table, *compexp)),
    })
}

fn rename_compexp(
    name_table: &NameTable,
    compexp: CompExp<Typed, String>,
) -> Box<CompExp<Typed, UID>> {
    Box::new(match compexp {
        CompExp::Comp(compop, addexp1, addexp2) => CompExp::Comp(
            compop,
            rename_addexp(name_table, *addexp1),
            rename_addexp(name_table, *addexp2),
        ),
        CompExp::AddExp(addexp) => CompExp::AddExp(rename_addexp(name_table, *addexp)),
    })
}

fn rename_addexp(name_table: &NameTable, addexp: AddExp<Typed, String>) -> Box<AddExp<Typed, UID>> {
    Box::new(match addexp {
        AddExp::Add(addop, mulexp, addexp) => AddExp::Add(
            addop,
            rename_mulexp(name_table, *mulexp),
            rename_addexp(name_table, *addexp),
        ),
        AddExp::MulExp(mulexp) => AddExp::MulExp(rename_mulexp(name_table, *mulexp)),
    })
}

fn rename_mulexp(name_table: &NameTable, mulexp: MulExp<Typed, String>) -> Box<MulExp<Typed, UID>> {
    Box::new(match mulexp {
        MulExp::Mul(mulop, unary, mulexp) => MulExp::Mul(
            mulop,
            rename_unary(name_table, *unary),
            rename_mulexp(name_table, *mulexp),
        ),
        MulExp::Unary(unary) => MulExp::Unary(rename_unary(name_table, *unary)),
    })
}

fn rename_unary(name_table: &NameTable, unary: Unary<Typed, String>) -> Box<Unary<Typed, UID>> {
    Box::new(match unary {
        Unary::Negate(unary) => Unary::Negate(rename_unary(name_table, *unary)),
        Unary::Not(unary) => Unary::Not(rename_unary(name_table, *unary)),
        Unary::Primary(primary) => Unary::Primary(rename_primary(name_table, *primary)),
    })
}

fn rename_primary(
    name_table: &NameTable,
    primary: Primary<Typed, String>,
) -> Box<Primary<Typed, UID>> {
    Box::new(match primary {
        Primary::Paren(expression) => Primary::Paren(rename_expression(name_table, *expression)),
        Primary::IntLit(n) => Primary::IntLit(n),
        Primary::StringLit(s) => Primary::StringLit(s),
        Primary::BoolLit(b) => Primary::BoolLit(b),
        Primary::Variable(t, varname) => Primary::Variable(t, *name_table.get(&varname).unwrap()),
        Primary::Unit() => Primary::Unit(),
    })
}
