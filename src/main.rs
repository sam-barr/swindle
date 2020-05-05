use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::Read;
use swindle::bytecode::*;
use swindle::parser::parse_program;
use swindle::renamer::*;
use swindle::tokenizer::*;
use swindle::typechecker::*;

fn main() {
    let code = {
        let file_name = env::args().collect::<Vec<_>>().pop().unwrap();
        let mut file = File::open(&file_name).unwrap();
        let mut code = String::new();
        file.read_to_string(&mut code).unwrap();
        code
    };

    let result = tokenize(&code)
        .and_then(|tokens| parse_program(&tokens))
        .and_then(type_program);

    match result {
        Ok(program) => {
            let (program, _num_variables) = rename_program(program);
            let (bytecode, strings) = byte_program(program);
            run(&bytecode, strings);
        }
        Err(e) => println!("{}", e),
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum SwindleValue<'a> {
    Int(i32),
    String(&'a String),
    Bool(bool),
    Unit,
}

impl fmt::Display for SwindleValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwindleValue::Int(n) => write!(f, "{}", n),
            SwindleValue::String(s) => write!(f, "{}", s),
            SwindleValue::Bool(b) => write!(f, "{}", b),
            SwindleValue::Unit => write!(f, "()"),
        }
    }
}

impl<'a> SwindleValue<'a> {
    fn negate(self) -> Self {
        match self {
            SwindleValue::Int(n) => SwindleValue::Int(-n),
            _ => panic!(),
        }
    }

    fn not(self) -> Self {
        match self {
            SwindleValue::Bool(b) => SwindleValue::Bool(!b),
            _ => panic!(),
        }
    }

    fn int_biop_int<F>(self, other: Self, f: F) -> Self
    where
        F: FnOnce(i32, i32) -> i32,
    {
        match (self, other) {
            (SwindleValue::Int(a), SwindleValue::Int(b)) => SwindleValue::Int(f(a, b)),
            _ => panic!(),
        }
    }

    fn int_biop_bool<F>(self, other: Self, f: F) -> Self
    where
        F: FnOnce(i32, i32) -> bool,
    {
        match (self, other) {
            (SwindleValue::Int(a), SwindleValue::Int(b)) => SwindleValue::Bool(f(a, b)),
            _ => panic!(),
        }
    }

    fn bool_biop_bool<F>(self, other: Self, f: F) -> Self
    where
        F: FnOnce(bool, bool) -> bool,
    {
        match (self, other) {
            (SwindleValue::Bool(a), SwindleValue::Bool(b)) => SwindleValue::Bool(f(a, b)),
            _ => panic!(),
        }
    }
}

fn run(bytecode: &[ByteCodeOp], strings: HashMap<UID, String>) {
    let labels = {
        let mut labels = HashMap::new();
        for (idx, op) in bytecode.iter().enumerate() {
            if let ByteCodeOp::Label(uid) = op {
                labels.insert(uid, idx);
            }
        }
        labels
    };
    let mut variables = HashMap::<UID, SwindleValue>::new();
    let mut stack = Vec::new();
    let mut idx = 0;

    while idx < bytecode.len() {
        //println!("{:?}: {:?}", bytecode[idx], stack);

        match bytecode[idx] {
            ByteCodeOp::IntConst(n) => stack.push(SwindleValue::Int(n)),
            ByteCodeOp::StringConst => {
                idx += 1;
                if let ByteCodeOp::UID(uid) = bytecode[idx] {
                    stack.push(SwindleValue::String(strings.get(&uid).unwrap()));
                }
            }
            ByteCodeOp::BoolConst(b) => stack.push(SwindleValue::Bool(b)),
            ByteCodeOp::Variable => {
                idx += 1;
                if let ByteCodeOp::UID(uid) = bytecode[idx] {
                    stack.push(*variables.get(&uid).unwrap());
                }
            }
            ByteCodeOp::Unit => stack.push(SwindleValue::Unit),
            ByteCodeOp::Pop => {
                stack.pop().unwrap();
            }

            ByteCodeOp::Negate => {
                let b = stack.pop().unwrap();
                stack.push(b.not());
            }
            ByteCodeOp::Not => {
                let b = stack.pop().unwrap();
                stack.push(b.negate());
            }

            ByteCodeOp::Product => {
                let n1 = stack.pop().unwrap();
                let n2 = stack.pop().unwrap();
                stack.push(n1.int_biop_int(n2, |a, b| a * b));
            }
            ByteCodeOp::Quotient => {
                let n1 = stack.pop().unwrap();
                let n2 = stack.pop().unwrap();
                stack.push(n1.int_biop_int(n2, |a, b| a / b));
            }
            ByteCodeOp::Remainder => {
                let n1 = stack.pop().unwrap();
                let n2 = stack.pop().unwrap();
                stack.push(n1.int_biop_int(n2, |a, b| a % b));
            }

            ByteCodeOp::Sum => {
                let n1 = stack.pop().unwrap();
                let n2 = stack.pop().unwrap();
                stack.push(n1.int_biop_int(n2, |a, b| a + b));
            }
            ByteCodeOp::Difference => {
                let n1 = stack.pop().unwrap();
                let n2 = stack.pop().unwrap();
                stack.push(n1.int_biop_int(n2, |a, b| a - b));
            }

            ByteCodeOp::Leq => {
                let n1 = stack.pop().unwrap();
                let n2 = stack.pop().unwrap();
                stack.push(n1.int_biop_bool(n2, |a, b| a <= b));
            }
            ByteCodeOp::Lt => {
                let n1 = stack.pop().unwrap();
                let n2 = stack.pop().unwrap();
                stack.push(n1.int_biop_bool(n2, |a, b| a < b));
            }
            ByteCodeOp::Eq => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(SwindleValue::Bool(a == b));
            }
            ByteCodeOp::Neq => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(SwindleValue::Bool(a != b));
            }
            ByteCodeOp::Gt => {
                let n1 = stack.pop().unwrap();
                let n2 = stack.pop().unwrap();
                stack.push(n1.int_biop_bool(n2, |a, b| a > b));
            }
            ByteCodeOp::Geq => {
                let n1 = stack.pop().unwrap();
                let n2 = stack.pop().unwrap();
                stack.push(n1.int_biop_bool(n2, |a, b| a >= b));
            }

            ByteCodeOp::And => {
                let b1 = stack.pop().unwrap();
                let b2 = stack.pop().unwrap();
                stack.push(b1.bool_biop_bool(b2, |a, b| a && b));
            }
            ByteCodeOp::Or => {
                let b1 = stack.pop().unwrap();
                let b2 = stack.pop().unwrap();
                stack.push(b1.bool_biop_bool(b2, |a, b| a || b));
            }

            ByteCodeOp::Declare => {
                idx += 1;
                if let ByteCodeOp::UID(uid) = bytecode[idx] {
                    let value = stack.pop().unwrap();
                    variables.insert(uid, value);
                }
            }
            ByteCodeOp::Assign => {
                idx += 1;
                if let ByteCodeOp::UID(uid) = bytecode[idx] {
                    let value = stack.pop().unwrap();
                    variables.insert(uid, value);
                    stack.push(value); // assignment returns the assigned value
                }
            }
            ByteCodeOp::Write => print!("{}", stack.pop().unwrap()),
            ByteCodeOp::Writeln => println!("{}", stack.pop().unwrap()),

            ByteCodeOp::Label(_) => {}
            ByteCodeOp::JumpIfFalse => {
                idx += 1;
                if let ByteCodeOp::UID(uid) = bytecode[idx] {
                    if let SwindleValue::Bool(false) = stack.pop().unwrap() {
                        idx = *labels.get(&uid).unwrap();
                    }
                }
            }
            ByteCodeOp::Jump => {
                idx += 1;
                if let ByteCodeOp::UID(uid) = bytecode[idx] {
                    idx = *labels.get(&uid).unwrap();
                }
            }
            _ => panic!("wut"),
        }

        idx += 1;
    }
}
