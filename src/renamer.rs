#![allow(dead_code)]
use crate::ast::*;
use crate::typechecker::{SwindleType, Typed};
use std::boxed::Box;
use std::collections::HashMap;
use std::default::Default;

// TODO: this could be a lot smarter

#[derive(Clone)]
struct NameTable {
    num_registers: usize,
    next_id: UID,
    table: HashMap<String, UID>,
}

impl NameTable {
    fn new() -> Self {
        NameTable {
            num_registers: 0,
            next_id: Default::default(),
            table: HashMap::new(),
        }
    }

    fn insert(&mut self, name: String) -> UID {
        self.num_registers += 1;
        self.table.insert(name, self.next_id);
        let id = self.next_id;
        self.next_id.inc();
        id
    }

    fn get(&self, name: &str) -> UID {
        *self.table.get(name).unwrap()
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct UID(usize);

impl UID {
    pub fn new() -> Self {
        UID(0)
    }

    pub fn inc(&mut self) {
        self.0 += 1;
    }

    pub fn get_value(&self) -> usize {
        self.0
    }
}

impl Default for UID {
    fn default() -> Self {
        UID::new()
    }
}

pub fn rename_program(program: Program<Typed, String>) -> (Program<Typed, UID>, usize) {
    let mut name_table = NameTable::new();
    let mut statements = Vec::new();

    for tagged_stmt in program.statements {
        statements.push(TaggedStatement::new(
            (),
            rename_statement(&mut name_table, tagged_stmt.statement),
        ));
    }

    (Program { statements }, name_table.num_registers)
}

fn rename_statement(
    name_table: &mut NameTable,
    statement: Statement<Typed, String>,
) -> Statement<Typed, UID> {
    match statement {
        Statement::Declare(typ, varname, expression) => {
            let expression = rename_expression(name_table, *expression);
            let new_name = name_table.insert(varname);
            Statement::Declare(typ, new_name, expression)
        }
        Statement::Write(tag, newline, expression) => {
            Statement::Write(tag, newline, rename_expression(name_table, *expression))
        }
        Statement::Break => Statement::Break,
        Statement::Continue => Statement::Continue,
        Statement::Expression(expression) => {
            Statement::Expression(rename_expression(name_table, *expression))
        }
    }
}

fn rename_expression(
    name_table: &mut NameTable,
    expression: Expression<Typed, String>,
) -> Box<Expression<Typed, UID>> {
    Box::new(match expression {
        Expression::Assign(varname, expression) => Expression::Assign(
            name_table.get(&varname),
            rename_expression(name_table, *expression),
        ),
        Expression::OrExp(orexp) => Expression::OrExp(rename_orexp(name_table, *orexp)),
    })
}

fn rename_whileexp(
    name_table: &mut NameTable,
    whileexp: WhileExp<Typed, String>,
) -> Box<WhileExp<Typed, UID>> {
    Box::new(WhileExp {
        cond: rename_expression(name_table, *whileexp.cond),
        body: rename_body(name_table, whileexp.body),
    })
}

fn rename_ifexp(name_table: &mut NameTable, ifexp: IfExp<Typed, String>) -> Box<IfExp<Typed, UID>> {
    Box::new(IfExp {
        cond: rename_expression(name_table, *ifexp.cond),
        body: rename_body(name_table, ifexp.body),
        elifs: {
            let mut elifs = Vec::new();
            for elif in ifexp.elifs {
                elifs.push(rename_elif(name_table, elif));
            }
            elifs
        },
        els: rename_body(name_table, ifexp.els),
    })
}

fn rename_elif(name_table: &mut NameTable, elif: Elif<Typed, String>) -> Elif<Typed, UID> {
    Elif {
        cond: rename_expression(name_table, *elif.cond),
        body: rename_body(name_table, elif.body),
    }
}

fn rename_body(name_table: &mut NameTable, body: Body<Typed, String>) -> Body<Typed, UID> {
    let mut name_table_clone = name_table.clone();
    let mut statements = Vec::new();
    for stmt in body.statements {
        statements.push(rename_statement(&mut name_table_clone, stmt));
    }
    name_table.num_registers = name_table_clone.num_registers;

    Body { statements }
}

fn rename_orexp(name_table: &mut NameTable, orexp: OrExp<Typed, String>) -> Box<OrExp<Typed, UID>> {
    Box::new(match orexp {
        OrExp::Or(andexp, orexp) => OrExp::Or(
            rename_andexp(name_table, *andexp),
            rename_orexp(name_table, *orexp),
        ),
        OrExp::AndExp(andexp) => OrExp::AndExp(rename_andexp(name_table, *andexp)),
    })
}

fn rename_andexp(
    name_table: &mut NameTable,
    andexp: AndExp<Typed, String>,
) -> Box<AndExp<Typed, UID>> {
    Box::new(match andexp {
        AndExp::And(compexp, andexp) => AndExp::And(
            rename_compexp(name_table, *compexp),
            rename_andexp(name_table, *andexp),
        ),
        AndExp::CompExp(compexp) => AndExp::CompExp(rename_compexp(name_table, *compexp)),
    })
}

fn rename_compexp(
    name_table: &mut NameTable,
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

fn rename_addexp(
    name_table: &mut NameTable,
    addexp: AddExp<Typed, String>,
) -> Box<AddExp<Typed, UID>> {
    Box::new(match addexp {
        AddExp::Add(addop, mulexp, addexp) => AddExp::Add(
            addop,
            rename_mulexp(name_table, *mulexp),
            rename_addexp(name_table, *addexp),
        ),
        AddExp::MulExp(mulexp) => AddExp::MulExp(rename_mulexp(name_table, *mulexp)),
    })
}

fn rename_mulexp(
    name_table: &mut NameTable,
    mulexp: MulExp<Typed, String>,
) -> Box<MulExp<Typed, UID>> {
    Box::new(match mulexp {
        MulExp::Mul(mulop, unary, mulexp) => MulExp::Mul(
            mulop,
            rename_unary(name_table, *unary),
            rename_mulexp(name_table, *mulexp),
        ),
        MulExp::Unary(unary) => MulExp::Unary(rename_unary(name_table, *unary)),
    })
}

fn rename_unary(name_table: &mut NameTable, unary: Unary<Typed, String>) -> Box<Unary<Typed, UID>> {
    Box::new(match unary {
        Unary::Negate(unary) => Unary::Negate(rename_unary(name_table, *unary)),
        Unary::Not(unary) => Unary::Not(rename_unary(name_table, *unary)),
        Unary::Primary(primary) => Unary::Primary(rename_primary(name_table, *primary)),
        Unary::Stringify(primaries) => {
            let mut typed_primaries = Vec::new();
            for primary in primaries {
                typed_primaries.push(*rename_primary(name_table, primary));
            }
            Unary::Stringify(typed_primaries)
        }
    })
}

fn rename_primary(
    name_table: &mut NameTable,
    primary: Primary<Typed, String>,
) -> Box<Primary<Typed, UID>> {
    Box::new(match primary {
        Primary::Paren(expression) => Primary::Paren(rename_expression(name_table, *expression)),
        Primary::IntLit(n) => Primary::IntLit(n),
        Primary::StringLit(s) => Primary::StringLit(s),
        Primary::BoolLit(b) => Primary::BoolLit(b),
        Primary::Variable(t, varname) => Primary::Variable(t, name_table.get(&varname)),
        Primary::Unit => Primary::Unit,
        Primary::IfExp(ifexp) => Primary::IfExp(rename_ifexp(name_table, *ifexp)),
        Primary::WhileExp(whileexp) => Primary::WhileExp(rename_whileexp(name_table, *whileexp)),
    })
}
