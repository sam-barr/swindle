#![allow(dead_code)]
use crate::ast::*;
use crate::renamer::UID;
use crate::typechecker::Typed;
use std::collections::HashMap;

// TODO: idea: clear stack once a statement terminates

#[derive(Debug, Copy, Clone)]
pub enum ByteCodeOp {
    // Primary
    UID(usize),      // this is an argument to other operatios
    IntConst(i64),   // push
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
    Eq,  // 2pop, push (NOTE there is no Neq)
    Gt,  // 2pop, push
    Geq, // 2pop, push

    // Bool
    And, // 2pop, push
    Or,  // 2pop, push

    Stringify, // pop, push
    Append,    // 2pop, push

    Assign,  // arg: UID, pop, push
    Declare, // arg: UID, pop, push
    Write,   // pop, push
    Writeln, // pop, push

    Label(usize), // basically a NOP
    JumpIfFalse,  // arg: UID, pop
    Jump,         // pop
}

struct CodeGenState {
    string_ids: HashMap<String, UID>,
    next_string_id: UID,
    next_label: UID,
    break_label: UID,
    continue_label: UID,
}

impl CodeGenState {
    fn new() -> Self {
        CodeGenState {
            string_ids: HashMap::new(),
            next_string_id: UID::new(),
            next_label: UID::new(),
            break_label: Default::default(),
            continue_label: Default::default(),
        }
    }

    fn lookup_string(&mut self, string: String) -> UID {
        match self.string_ids.get(&string) {
            Some(uid) => *uid,
            None => {
                self.string_ids.insert(string, self.next_string_id);
                let new_id = self.next_string_id;
                self.next_string_id.inc();
                new_id
            }
        }
    }

    fn get_label(&mut self) -> UID {
        let label = self.next_label;
        self.next_label.inc();
        label
    }
}

pub fn byte_program(
    program: Program<Typed, UID>,
) -> (Vec<ByteCodeOp>, HashMap<String, UID>, usize, usize) {
    let mut state = CodeGenState::new();
    let mut bytecode = Vec::new();
    for tagged_stmt in program.statements {
        bytecode.append(&mut byte_statement(&mut state, tagged_stmt.statement));

        // all statements push a value onto the stack
        // so we pop them off
        bytecode.push(ByteCodeOp::Pop);
    }

    (
        bytecode,
        state.string_ids,
        state.next_label.get_value(),
        state.next_string_id.get_value(),
    )
}

fn byte_statement(state: &mut CodeGenState, statement: Statement<Typed, UID>) -> Vec<ByteCodeOp> {
    match statement {
        Statement::Declare(_, uid, expression) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_expression(state, *expression));
            bc.push(ByteCodeOp::Declare);
            bc.push(ByteCodeOp::UID(uid.get_value()));
            bc.push(ByteCodeOp::Unit); // All the non-expression statements return unit
            bc
        }
        Statement::Write(_, expression) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_expression(state, *expression));
            bc.push(ByteCodeOp::Write);
            bc.push(ByteCodeOp::Unit);
            bc
        }
        Statement::Writeln(_, expression) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_expression(state, *expression));
            bc.push(ByteCodeOp::Writeln);
            bc.push(ByteCodeOp::Unit);
            bc
        }
        Statement::Break => vec![
            ByteCodeOp::Jump,
            ByteCodeOp::UID(state.break_label.get_value()),
        ],
        Statement::Continue => vec![
            ByteCodeOp::Jump,
            ByteCodeOp::UID(state.continue_label.get_value()),
        ],
        Statement::Expression(expression) => byte_expression(state, *expression),
    }
}

fn byte_expression(
    state: &mut CodeGenState,
    expression: Expression<Typed, UID>,
) -> Vec<ByteCodeOp> {
    match expression {
        Expression::Assign(uid, expression) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_expression(state, *expression));
            bc.push(ByteCodeOp::Assign);
            bc.push(ByteCodeOp::UID(uid.get_value()));
            bc
        }
        Expression::OrExp(orexp) => byte_orexp(state, *orexp),
    }
}

fn byte_whileexp(state: &mut CodeGenState, whileexp: WhileExp<Typed, UID>) -> Vec<ByteCodeOp> {
    let old_continue_label = state.continue_label;
    let old_break_label = state.break_label;
    let start_label = state.get_label();
    let end_label = state.get_label();
    state.continue_label = start_label;
    state.break_label = end_label;

    let mut bc = Vec::new();
    bc.push(ByteCodeOp::Label(start_label.get_value()));
    bc.append(&mut byte_expression(state, *whileexp.cond));
    bc.push(ByteCodeOp::JumpIfFalse);
    bc.push(ByteCodeOp::UID(end_label.get_value()));
    bc.append(&mut byte_body(state, whileexp.body));
    bc.push(ByteCodeOp::Pop);
    bc.push(ByteCodeOp::Jump);
    bc.push(ByteCodeOp::UID(start_label.get_value()));
    bc.push(ByteCodeOp::Label(end_label.get_value()));
    bc.push(ByteCodeOp::Unit);

    state.continue_label = old_continue_label;
    state.break_label = old_break_label;

    bc
}

fn byte_ifexp(state: &mut CodeGenState, ifexp: IfExp<Typed, UID>) -> Vec<ByteCodeOp> {
    let end_label = state.get_label();
    let label = state.get_label();

    let mut bc = Vec::new();
    bc.append(&mut byte_expression(state, *ifexp.cond));
    bc.push(ByteCodeOp::JumpIfFalse);
    bc.push(ByteCodeOp::UID(label.get_value()));
    bc.append(&mut byte_body(state, ifexp.body));
    bc.push(ByteCodeOp::Jump);
    bc.push(ByteCodeOp::UID(end_label.get_value()));
    bc.push(ByteCodeOp::Label(label.get_value()));

    for elif in ifexp.elifs {
        bc.append(&mut byte_elif(end_label, state, elif));
    }

    bc.append(&mut byte_body(state, ifexp.els));
    bc.push(ByteCodeOp::Label(end_label.get_value()));

    bc
}

fn byte_elif(end_label: UID, state: &mut CodeGenState, elif: Elif<Typed, UID>) -> Vec<ByteCodeOp> {
    let label = state.get_label();
    let mut bc = Vec::new();
    bc.append(&mut byte_expression(state, *elif.cond));
    bc.push(ByteCodeOp::JumpIfFalse);
    bc.push(ByteCodeOp::UID(label.get_value()));
    bc.append(&mut byte_body(state, elif.body));
    bc.push(ByteCodeOp::Jump);
    bc.push(ByteCodeOp::UID(end_label.get_value()));
    bc.push(ByteCodeOp::Label(label.get_value()));

    bc
}

fn byte_body(state: &mut CodeGenState, body: Body<Typed, UID>) -> Vec<ByteCodeOp> {
    let mut bc = Vec::new();
    if !body.statements.is_empty() {
        for stmt in body.statements {
            bc.append(&mut byte_statement(state, stmt));
            bc.push(ByteCodeOp::Pop);
        }
        // remove the last pop, becaue the last statement is the value we return
        bc.pop().unwrap();
    } else {
        bc.push(ByteCodeOp::Unit);
    }
    bc
}

fn byte_orexp(state: &mut CodeGenState, orexp: OrExp<Typed, UID>) -> Vec<ByteCodeOp> {
    match orexp {
        OrExp::Or(andexp, orexp) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_orexp(state, *orexp));
            bc.append(&mut byte_andexp(state, *andexp));
            bc.push(ByteCodeOp::Or);
            bc
        }
        OrExp::AndExp(andexp) => byte_andexp(state, *andexp),
    }
}

fn byte_andexp(state: &mut CodeGenState, andexp: AndExp<Typed, UID>) -> Vec<ByteCodeOp> {
    match andexp {
        AndExp::And(compexp, andexp) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_andexp(state, *andexp));
            bc.append(&mut byte_compexp(state, *compexp));
            bc.push(ByteCodeOp::And);
            bc
        }
        AndExp::CompExp(compexp) => byte_compexp(state, *compexp),
    }
}

fn byte_compexp(state: &mut CodeGenState, compexp: CompExp<Typed, UID>) -> Vec<ByteCodeOp> {
    match compexp {
        CompExp::Comp(compop, addexp1, addexp2) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_addexp(state, *addexp2));
            bc.append(&mut byte_addexp(state, *addexp1));
            match compop {
                CompOp::Leq => bc.push(ByteCodeOp::Leq),
                CompOp::Lt => bc.push(ByteCodeOp::Lt),
                CompOp::Eq => bc.push(ByteCodeOp::Eq),
                CompOp::Neq => {
                    bc.push(ByteCodeOp::Eq);
                    bc.push(ByteCodeOp::Not);
                }
                CompOp::Gt => bc.push(ByteCodeOp::Gt),
                CompOp::Geq => bc.push(ByteCodeOp::Geq),
            }
            bc
        }
        CompExp::AddExp(addexp) => byte_addexp(state, *addexp),
    }
}

fn byte_addexp(state: &mut CodeGenState, addexp: AddExp<Typed, UID>) -> Vec<ByteCodeOp> {
    match addexp {
        AddExp::Add(addop, mulexp, addexp) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_addexp(state, *addexp));
            bc.append(&mut byte_mulexp(state, *mulexp));
            bc.push(match addop {
                AddOp::Sum => ByteCodeOp::Sum,
                AddOp::Difference => ByteCodeOp::Difference,
            });
            bc
        }
        AddExp::MulExp(mulexp) => byte_mulexp(state, *mulexp),
    }
}

fn byte_mulexp(state: &mut CodeGenState, mulexp: MulExp<Typed, UID>) -> Vec<ByteCodeOp> {
    match mulexp {
        MulExp::Mul(mulop, unary, mulexp) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_mulexp(state, *mulexp));
            bc.append(&mut byte_unary(state, *unary));
            bc.push(match mulop {
                MulOp::Product => ByteCodeOp::Product,
                MulOp::Quotient => ByteCodeOp::Quotient,
                MulOp::Remainder => ByteCodeOp::Remainder,
            });
            bc
        }
        MulExp::Unary(unary) => byte_unary(state, *unary),
    }
}

fn byte_unary(state: &mut CodeGenState, unary: Unary<Typed, UID>) -> Vec<ByteCodeOp> {
    match unary {
        Unary::Negate(unary) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_unary(state, *unary));
            bc.push(ByteCodeOp::Negate);
            bc
        }
        Unary::Not(unary) => {
            let mut bc = Vec::new();
            bc.append(&mut byte_unary(state, *unary));
            bc.push(ByteCodeOp::Not);
            bc
        }
        Unary::Primary(primary) => byte_primary(state, *primary),
        Unary::Stringify(mut primaries) => {
            let mut bc = byte_primary(state, primaries.pop().unwrap());
            bc.push(ByteCodeOp::Stringify);
            while let Some(primary) = primaries.pop() {
                bc.append(&mut byte_primary(state, primary));
                bc.push(ByteCodeOp::Stringify);
                bc.push(ByteCodeOp::Append);
            }
            bc
        }
    }
}

fn byte_primary(state: &mut CodeGenState, primary: Primary<Typed, UID>) -> Vec<ByteCodeOp> {
    match primary {
        Primary::Paren(expression) => byte_expression(state, *expression),
        Primary::IntLit(n) => vec![ByteCodeOp::IntConst(n)],
        Primary::StringLit(s) => vec![
            ByteCodeOp::StringConst,
            ByteCodeOp::UID(state.lookup_string(s).get_value()),
        ],
        Primary::BoolLit(b) => vec![ByteCodeOp::BoolConst(b)],
        Primary::Variable(_, uid) => vec![ByteCodeOp::Variable, ByteCodeOp::UID(uid.get_value())],
        Primary::Unit => vec![ByteCodeOp::Unit],
        Primary::IfExp(ifexp) => byte_ifexp(state, *ifexp),
        Primary::WhileExp(whileexp) => byte_whileexp(state, *whileexp),
    }
}
