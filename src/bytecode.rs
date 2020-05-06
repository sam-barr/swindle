#![allow(dead_code)]
use crate::ast::*;
use crate::renamer::UID;
use crate::typechecker::Typed;
use std::collections::HashMap;

// TODO: idea: clear stack once a statement terminates

#[derive(Debug, Copy, Clone)]
pub enum ByteCodeOp {
    // Primary
    UID(UID),        // this is an argument to other operatios
    IntConst(i32),   // push
    StringConst,     // args: UID
    BoolConst(bool), // push
    Variable,        // args: UID
    Unit,            // push
    Pop,             // pop

    // Unary
    Negate, // pop, push
    Not,    // pop, push

    // Mul
    Product,   // 2pop, push
    Quotient,  // 2pop, push
    Remainder, // 2pop, push

    // Add
    Sum,        // 2pop, push
    Difference, // 2pop, push

    // Comp
    Leq, // 2pop, push
    Lt,  // 2pop, push
    Eq,  // 2pop, push
    Neq, // 2pop, push
    Gt,  // 2pop, push
    Geq, // 2pop, push

    // Bool
    And, // 2pop, push
    Or,  // 2pop, push

    Append, // usize pop, push

    Assign,  // arg: UID, pop, push
    Declare, // arg: UID, pop, push
    Write,   // pop, push
    Writeln, // pop, push

    Label(UID),  // basically a NOP
    JumpIfFalse, // arg: UID, pop
    Jump,        // pop
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
    let mut label = UID::new();
    for tagged_stmt in program.statements {
        bytecode.append(&mut byte_statement(
            &mut label,
            &mut strings,
            tagged_stmt.statement,
        ));

        // all statements push a value onto the stack
        // so we pop them off
        bytecode.push(ByteCodeOp::Pop);
    }

    let mut string_map = HashMap::new();
    for (string, uid) in strings.ids {
        string_map.insert(uid, string);
    }

    (bytecode, string_map)
}

fn byte_statement(
    label: &mut UID,
    strings: &mut StringTable,
    statement: Statement<Typed, UID>,
) -> Vec<ByteCodeOp> {
    match statement {
        Statement::Declare(_, uid, expression) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_expression(label, strings, *expression));
            bc.push(ByteCodeOp::Declare);
            bc.push(ByteCodeOp::UID(uid));
            bc.push(ByteCodeOp::Unit); // All the non-expression statements return unit
            bc
        }
        Statement::Write(_, expression) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_expression(label, strings, *expression));
            bc.push(ByteCodeOp::Write);
            bc.push(ByteCodeOp::Unit);
            bc
        }
        Statement::Writeln(_, expression) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_expression(label, strings, *expression));
            bc.push(ByteCodeOp::Writeln);
            bc.push(ByteCodeOp::Unit);
            bc
        }
        Statement::Expression(expression) => byte_expression(label, strings, *expression),
    }
}

fn byte_expression(
    label: &mut UID,
    strings: &mut StringTable,
    expression: Expression<Typed, UID>,
) -> Vec<ByteCodeOp> {
    match expression {
        Expression::Assign(uid, expression) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_expression(label, strings, *expression));
            bc.push(ByteCodeOp::Assign);
            bc.push(ByteCodeOp::UID(uid));
            bc
        }
        Expression::WhileExp(whileexp) => byte_whileexp(label, strings, *whileexp),
        Expression::IfExp(ifexp) => byte_ifexp(label, strings, *ifexp),
        Expression::OrExp(orexp) => byte_orexp(label, strings, *orexp),
    }
}

fn byte_whileexp(
    label: &mut UID,
    strings: &mut StringTable,
    whileexp: WhileExp<Typed, UID>,
) -> Vec<ByteCodeOp> {
    let start_label = *label;
    label.inc();
    let end_label = *label;
    label.inc();

    let mut bc = Vec::new();
    bc.push(ByteCodeOp::Label(start_label));
    bc.append(&mut byte_expression(label, strings, *whileexp.cond));
    bc.push(ByteCodeOp::JumpIfFalse);
    bc.push(ByteCodeOp::UID(end_label));
    bc.append(&mut byte_body(label, strings, whileexp.body));
    bc.push(ByteCodeOp::Pop);
    bc.push(ByteCodeOp::Jump);
    bc.push(ByteCodeOp::UID(start_label));
    bc.push(ByteCodeOp::Label(end_label));
    bc.push(ByteCodeOp::Unit);

    bc
}

fn byte_ifexp(
    label: &mut UID,
    strings: &mut StringTable,
    ifexp: IfExp<Typed, UID>,
) -> Vec<ByteCodeOp> {
    let end_label = *label;
    label.inc();

    let mut bc = Vec::new();
    bc.append(&mut byte_expression(label, strings, *ifexp.cond));
    bc.push(ByteCodeOp::JumpIfFalse);
    bc.push(ByteCodeOp::UID(*label));
    bc.append(&mut byte_body(label, strings, ifexp.body));
    bc.push(ByteCodeOp::Jump);
    bc.push(ByteCodeOp::UID(end_label));
    bc.push(ByteCodeOp::Label(*label));
    label.inc();

    for elif in ifexp.elifs {
        bc.append(&mut byte_elif(end_label, label, strings, elif));
    }

    bc.append(&mut byte_body(label, strings, ifexp.els));
    bc.push(ByteCodeOp::Label(end_label));

    bc
}

fn byte_elif(
    end_label: UID,
    label: &mut UID,
    strings: &mut StringTable,
    elif: Elif<Typed, UID>,
) -> Vec<ByteCodeOp> {
    let mut bc = Vec::new();
    bc.append(&mut byte_expression(label, strings, *elif.cond));
    bc.push(ByteCodeOp::JumpIfFalse);
    bc.push(ByteCodeOp::UID(*label));
    bc.append(&mut byte_body(label, strings, elif.body));
    bc.push(ByteCodeOp::Jump);
    bc.push(ByteCodeOp::UID(end_label));
    bc.push(ByteCodeOp::Label(*label));
    label.inc();

    bc
}

fn byte_body(
    label: &mut UID,
    strings: &mut StringTable,
    body: Body<Typed, UID>,
) -> Vec<ByteCodeOp> {
    let mut bc = Vec::new();
    if !body.statements.is_empty() {
        for stmt in body.statements {
            bc.append(&mut byte_statement(label, strings, stmt));
            bc.push(ByteCodeOp::Pop);
        }
        // remove the last pop, becaue the last statement is the value we return
        bc.pop().unwrap();
    } else {
        bc.push(ByteCodeOp::Unit);
    }
    bc
}

fn byte_orexp(
    label: &mut UID,
    strings: &mut StringTable,
    orexp: OrExp<Typed, UID>,
) -> Vec<ByteCodeOp> {
    match orexp {
        OrExp::Or(andexp, orexp) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_orexp(label, strings, *orexp));
            bc.append(&mut byte_andexp(label, strings, *andexp));
            bc.push(ByteCodeOp::Or);
            bc
        }
        OrExp::AndExp(andexp) => byte_andexp(label, strings, *andexp),
    }
}

fn byte_andexp(
    label: &mut UID,
    strings: &mut StringTable,
    andexp: AndExp<Typed, UID>,
) -> Vec<ByteCodeOp> {
    match andexp {
        AndExp::And(compexp, andexp) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_andexp(label, strings, *andexp));
            bc.append(&mut byte_compexp(label, strings, *compexp));
            bc.push(ByteCodeOp::And);
            bc
        }
        AndExp::CompExp(compexp) => byte_compexp(label, strings, *compexp),
    }
}

fn byte_compexp(
    label: &mut UID,
    strings: &mut StringTable,
    compexp: CompExp<Typed, UID>,
) -> Vec<ByteCodeOp> {
    match compexp {
        CompExp::Comp(compop, addexp1, addexp2) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_addexp(label, strings, *addexp2));
            bc.append(&mut byte_addexp(label, strings, *addexp1));
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
        CompExp::AddExp(addexp) => byte_addexp(label, strings, *addexp),
    }
}

fn byte_addexp(
    label: &mut UID,
    strings: &mut StringTable,
    addexp: AddExp<Typed, UID>,
) -> Vec<ByteCodeOp> {
    match addexp {
        AddExp::Add(addop, mulexp, addexp) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_addexp(label, strings, *addexp));
            bc.append(&mut byte_mulexp(label, strings, *mulexp));
            bc.push(match addop {
                AddOp::Sum => ByteCodeOp::Sum,
                AddOp::Difference => ByteCodeOp::Difference,
            });
            bc
        }
        AddExp::MulExp(mulexp) => byte_mulexp(label, strings, *mulexp),
    }
}

fn byte_mulexp(
    label: &mut UID,
    strings: &mut StringTable,
    mulexp: MulExp<Typed, UID>,
) -> Vec<ByteCodeOp> {
    match mulexp {
        MulExp::Mul(mulop, unary, mulexp) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_mulexp(label, strings, *mulexp));
            bc.append(&mut byte_unary(label, strings, *unary));
            bc.push(match mulop {
                MulOp::Product => ByteCodeOp::Product,
                MulOp::Quotient => ByteCodeOp::Quotient,
                MulOp::Remainder => ByteCodeOp::Remainder,
            });
            bc
        }
        MulExp::Unary(unary) => byte_unary(label, strings, *unary),
    }
}

fn byte_unary(
    label: &mut UID,
    strings: &mut StringTable,
    unary: Unary<Typed, UID>,
) -> Vec<ByteCodeOp> {
    match unary {
        Unary::Negate(unary) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_unary(label, strings, *unary));
            bc.push(ByteCodeOp::Negate);
            bc
        }
        Unary::Not(unary) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_unary(label, strings, *unary));
            bc.push(ByteCodeOp::Not);
            bc
        }
        Unary::Primary(primary) => byte_primary(label, strings, *primary),
        Unary::Append(mut append) => {
            let num = append.len();
            let mut bc = Vec::new();
            while let Some(primary) = append.pop() {
                bc.append(&mut byte_primary(label, strings, primary));
            }
            for _ in 0..(num - 1) {
                bc.push(ByteCodeOp::Append);
            }
            bc
        }
    }
}

fn byte_primary(
    label: &mut UID,
    strings: &mut StringTable,
    primary: Primary<Typed, UID>,
) -> Vec<ByteCodeOp> {
    match primary {
        Primary::Paren(expression) => byte_expression(label, strings, *expression),
        Primary::IntLit(n) => vec![ByteCodeOp::IntConst(n)],
        Primary::StringLit(s) => vec![ByteCodeOp::StringConst, ByteCodeOp::UID(strings.get(s))],
        Primary::BoolLit(b) => vec![ByteCodeOp::BoolConst(b)],
        Primary::Variable(_, uid) => vec![ByteCodeOp::Variable, ByteCodeOp::UID(uid)],
        Primary::Unit() => vec![ByteCodeOp::Unit],
    }
}
