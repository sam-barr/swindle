#![allow(dead_code)]
use crate::ast::*;
use crate::renamer::UID;
use crate::typechecker::Typed;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
pub enum ByteCodeOp {
    // Primary
    UID(UID),
    IntConst(i32),
    StringConst, // args: UID
    BoolConst(bool),
    Variable, // args: UID
    Unit,

    // Unary
    Negate, // pop
    Not,    // pop

    // Mul
    Product,  // 2pop
    Quotient, // 2pop

    // Add
    Sum,        // 2pop
    Difference, // 2pop

    // Comp
    Leq, // 2pop
    Lt,  // 2pop
    Eq,  // 2pop
    Neq, // 2pop
    Gt,  // 2pop
    Geq, // 2pop

    // Bool
    And, // 2pop
    Or,  // 2pop

    Assign,  // arg: UID, pop
    Declare, // arg: UID, pop
    Write,   // pop
    Writeln, // pop
}

struct StringTable {
    ids: HashMap<String, UID>,
    next_id: UID,
}

impl StringTable {
    fn new() -> Self {
        StringTable {
            ids: HashMap::new(),
            next_id: UID::new(),
        }
    }

    fn get(&mut self, string: String) -> UID {
        match self.ids.get(&string) {
            Some(uid) => *uid,
            None => {
                self.ids.insert(string, self.next_id);
                let new_id = self.next_id;
                self.next_id.inc();
                new_id
            }
        }
    }
}

pub fn byte_program(program: Program<Typed, UID>) -> (Vec<ByteCodeOp>, HashMap<UID, String>) {
    let mut strings = StringTable::new();
    let mut bytecode = Vec::new();
    for ((), stmt) in program.statements {
        bytecode.append(&mut byte_statement(&mut strings, *stmt));
    }

    let mut string_map = HashMap::new();
    for (string, uid) in strings.ids {
        string_map.insert(uid, string);
    }

    (bytecode, string_map)
}

fn byte_statement(strings: &mut StringTable, statement: Statement<Typed, UID>) -> Vec<ByteCodeOp> {
    match statement {
        Statement::Declare(_, uid, expression) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_expression(strings, *expression));
            bc.push(ByteCodeOp::Declare);
            bc.push(ByteCodeOp::UID(uid));
            bc
        }
        Statement::Write(_, expression) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_expression(strings, *expression));
            bc.push(ByteCodeOp::Write);
            bc
        }
        Statement::Writeln(_, expression) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_expression(strings, *expression));
            bc.push(ByteCodeOp::Writeln);
            bc
        }
        Statement::Expression(expression) => byte_expression(strings, *expression),
    }
}

fn byte_expression(
    strings: &mut StringTable,
    expression: Expression<Typed, UID>,
) -> Vec<ByteCodeOp> {
    match expression {
        Expression::Assign(uid, expression) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_expression(strings, *expression));
            bc.push(ByteCodeOp::Assign);
            bc.push(ByteCodeOp::UID(uid));
            bc
        }
        Expression::OrExp(orexp) => byte_orexp(strings, *orexp),
    }
}

fn byte_orexp(strings: &mut StringTable, orexp: OrExp<Typed, UID>) -> Vec<ByteCodeOp> {
    match orexp {
        OrExp::Or(andexp, orexp) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_andexp(strings, *andexp));
            bc.append(&mut byte_orexp(strings, *orexp));
            bc.push(ByteCodeOp::Or);
            bc
        }
        OrExp::AndExp(andexp) => byte_andexp(strings, *andexp),
    }
}

fn byte_andexp(strings: &mut StringTable, andexp: AndExp<Typed, UID>) -> Vec<ByteCodeOp> {
    match andexp {
        AndExp::And(compexp, andexp) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_compexp(strings, *compexp));
            bc.append(&mut byte_andexp(strings, *andexp));
            bc.push(ByteCodeOp::And);
            bc
        }
        AndExp::CompExp(compexp) => byte_compexp(strings, *compexp),
    }
}

fn byte_compexp(strings: &mut StringTable, compexp: CompExp<Typed, UID>) -> Vec<ByteCodeOp> {
    match compexp {
        CompExp::Comp(compop, addexp1, addexp2) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_addexp(strings, *addexp1));
            bc.append(&mut byte_addexp(strings, *addexp2));
            bc.push(match compop {
                CompOp::Leq => ByteCodeOp::Leq,
                CompOp::Lt => ByteCodeOp::Lt,
                CompOp::Eq => ByteCodeOp::Eq,
                CompOp::Neq => ByteCodeOp::Neq,
                CompOp::Gt => ByteCodeOp::Gt,
                CompOp::Geq => ByteCodeOp::Geq,
            });
            bc
        }
        CompExp::AddExp(addexp) => byte_addexp(strings, *addexp),
    }
}

fn byte_addexp(strings: &mut StringTable, addexp: AddExp<Typed, UID>) -> Vec<ByteCodeOp> {
    match addexp {
        AddExp::Add(addop, mulexp, addexp) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_mulexp(strings, *mulexp));
            bc.append(&mut byte_addexp(strings, *addexp));
            bc.push(match addop {
                AddOp::Sum => ByteCodeOp::Sum,
                AddOp::Difference => ByteCodeOp::Difference,
            });
            bc
        }
        AddExp::MulExp(mulexp) => byte_mulexp(strings, *mulexp),
    }
}

fn byte_mulexp(strings: &mut StringTable, mulexp: MulExp<Typed, UID>) -> Vec<ByteCodeOp> {
    match mulexp {
        MulExp::Mul(mulop, unary, mulexp) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_unary(strings, *unary));
            bc.append(&mut byte_mulexp(strings, *mulexp));
            bc.push(match mulop {
                MulOp::Product => ByteCodeOp::Product,
                MulOp::Quotient => ByteCodeOp::Quotient,
            });
            bc
        }
        MulExp::Unary(unary) => byte_unary(strings, *unary),
    }
}

fn byte_unary(strings: &mut StringTable, unary: Unary<Typed, UID>) -> Vec<ByteCodeOp> {
    match unary {
        Unary::Negate(unary) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_unary(strings, *unary));
            bc.push(ByteCodeOp::Negate);
            bc
        }
        Unary::Not(unary) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_unary(strings, *unary));
            bc.push(ByteCodeOp::Not);
            bc
        }
        Unary::Primary(primary) => byte_primary(strings, *primary),
    }
}

fn byte_primary(strings: &mut StringTable, primary: Primary<Typed, UID>) -> Vec<ByteCodeOp> {
    match primary {
        Primary::Paren(expression) => byte_expression(strings, *expression),
        Primary::IntLit(n) => vec![ByteCodeOp::IntConst(n)],
        Primary::StringLit(s) => vec![ByteCodeOp::StringConst, ByteCodeOp::UID(strings.get(s))],
        Primary::BoolLit(b) => vec![ByteCodeOp::BoolConst(b)],
        Primary::Variable(_, uid) => vec![ByteCodeOp::Variable, ByteCodeOp::UID(uid)],
        Primary::Unit() => vec![ByteCodeOp::Unit],
    }
}
