#![allow(dead_code)]
use std::collections::HashMap;
use std::env;
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
            let vm = VM::new(bytecode, strings);
            vm.run(false);
        }
        Err(e) => println!("{}", e),
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum SwindleValue {
    Int(i64),
    ConstString(UID),
    HeapString(usize),
    Bool(bool),
    Unit,
}

impl SwindleValue {
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
        F: FnOnce(i64, i64) -> i64,
    {
        match (self, other) {
            (SwindleValue::Int(a), SwindleValue::Int(b)) => SwindleValue::Int(f(a, b)),
            _ => panic!(),
        }
    }

    fn int_biop_bool<F>(self, other: Self, f: F) -> Self
    where
        F: FnOnce(i64, i64) -> bool,
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

//struct Heap {
//    heap: HashMap<UID, String>
//}

struct VM {
    bytecode: Vec<ByteCodeOp>,
    idx: usize,
    strings: HashMap<UID, String>,
    variables: HashMap<UID, SwindleValue>,
    stack: Vec<SwindleValue>,
    labels: HashMap<UID, usize>,
    heap: Vec<String>,
}

impl VM {
    fn new(bytecode: Vec<ByteCodeOp>, strings: HashMap<UID, String>) -> Self {
        let mut labels = HashMap::new();
        for (idx, op) in bytecode.iter().enumerate() {
            if let ByteCodeOp::Label(uid) = op {
                labels.insert(*uid, idx);
            }
        }

        VM {
            bytecode,
            idx: 0,
            strings,
            variables: HashMap::new(),
            stack: Vec::new(),
            labels,
            heap: Vec::new(),
        }
    }

    fn debug(&self) {
        println!("~~~~~~~~VM~~~~~~~~");
        println!("Current op: {:?}", self.bytecode[self.idx]);
        println!("Variables: {:?}", self.variables);
        println!("Stack: {:?}", self.stack);
        println!("Heap: {:?}", self.heap);
        println!("~~~~~~~~~~~~~~~~~~");
    }

    fn display(&self, value: SwindleValue) -> String {
        match value {
            SwindleValue::ConstString(uid) => self.strings.get(&uid).unwrap().to_string(),
            SwindleValue::HeapString(idx) => self.heap[idx].to_string(),
            SwindleValue::Unit => "()".to_string(),
            SwindleValue::Int(n) => n.to_string(),
            SwindleValue::Bool(b) => b.to_string(),
        }
    }

    fn push(&mut self, value: SwindleValue) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> SwindleValue {
        self.stack.pop().unwrap()
    }

    fn run(mut self, debug: bool) {
        while self.idx < self.bytecode.len() {
            if debug {
                self.debug();
                let mut buffer = String::new();
                std::io::stdin().read_line(&mut buffer).unwrap();
            }

            match self.bytecode[self.idx] {
                ByteCodeOp::IntConst(n) => self.push(SwindleValue::Int(n)),
                ByteCodeOp::StringConst => {
                    self.idx += 1;
                    if let ByteCodeOp::UID(uid) = self.bytecode[self.idx] {
                        self.push(SwindleValue::ConstString(uid));
                    }
                }
                ByteCodeOp::BoolConst(b) => self.push(SwindleValue::Bool(b)),
                ByteCodeOp::Variable => {
                    self.idx += 1;
                    if let ByteCodeOp::UID(uid) = self.bytecode[self.idx] {
                        self.push(*self.variables.get(&uid).unwrap());
                    }
                }
                ByteCodeOp::Unit => self.push(SwindleValue::Unit),
                ByteCodeOp::Pop => {
                    self.pop();
                }

                ByteCodeOp::Negate => {
                    let b = self.pop();
                    self.push(b.not());
                }
                ByteCodeOp::Not => {
                    let b = self.pop();
                    self.push(b.negate());
                }

                ByteCodeOp::Product => {
                    let n1 = self.pop();
                    let n2 = self.pop();
                    self.push(n1.int_biop_int(n2, |a, b| a * b));
                }
                ByteCodeOp::Quotient => {
                    let n1 = self.pop();
                    let n2 = self.pop();
                    self.push(n1.int_biop_int(n2, |a, b| a / b));
                }
                ByteCodeOp::Remainder => {
                    let n1 = self.pop();
                    let n2 = self.pop();
                    self.push(n1.int_biop_int(n2, |a, b| a % b));
                }

                ByteCodeOp::Sum => {
                    let n1 = self.pop();
                    let n2 = self.pop();
                    self.push(n1.int_biop_int(n2, |a, b| a + b));
                }
                ByteCodeOp::Difference => {
                    let n1 = self.pop();
                    let n2 = self.pop();
                    self.push(n1.int_biop_int(n2, |a, b| a - b));
                }

                ByteCodeOp::Leq => {
                    let n1 = self.pop();
                    let n2 = self.pop();
                    self.push(n1.int_biop_bool(n2, |a, b| a <= b));
                }
                ByteCodeOp::Lt => {
                    let n1 = self.pop();
                    let n2 = self.pop();
                    self.push(n1.int_biop_bool(n2, |a, b| a < b));
                }
                ByteCodeOp::Eq => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(SwindleValue::Bool(a == b));
                }
                ByteCodeOp::Neq => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(SwindleValue::Bool(a != b));
                }
                ByteCodeOp::Gt => {
                    let n1 = self.pop();
                    let n2 = self.pop();
                    self.push(n1.int_biop_bool(n2, |a, b| a > b));
                }
                ByteCodeOp::Geq => {
                    let n1 = self.pop();
                    let n2 = self.pop();
                    self.push(n1.int_biop_bool(n2, |a, b| a >= b));
                }

                ByteCodeOp::And => {
                    let b1 = self.pop();
                    let b2 = self.pop();
                    self.push(b1.bool_biop_bool(b2, |a, b| a && b));
                }
                ByteCodeOp::Or => {
                    let b1 = self.pop();
                    let b2 = self.pop();
                    self.push(b1.bool_biop_bool(b2, |a, b| a || b));
                }

                ByteCodeOp::Append => {
                    let s1 = self.pop();
                    let s2 = self.pop();
                    let i = self.heap.len();
                    let s = format!("{}{}", self.display(s1), self.display(s2));
                    self.heap.push(s);
                    self.push(SwindleValue::HeapString(i));
                }

                ByteCodeOp::Declare => {
                    self.idx += 1;
                    if let ByteCodeOp::UID(uid) = self.bytecode[self.idx] {
                        let value = self.pop();
                        self.variables.insert(uid, value);
                    }
                }
                ByteCodeOp::Assign => {
                    self.idx += 1;
                    if let ByteCodeOp::UID(uid) = self.bytecode[self.idx] {
                        let value = self.pop();
                        self.variables.insert(uid, value);
                        self.push(value); // assignment returns the assigned value
                    }
                }
                ByteCodeOp::Write => {
                    let v = self.pop();
                    print!("{}", self.display(v));
                }
                ByteCodeOp::Writeln => {
                    let v = self.pop();
                    println!("{}", self.display(v));
                }

                ByteCodeOp::Label(_) => {}
                ByteCodeOp::JumpIfFalse => {
                    self.idx += 1;
                    if let ByteCodeOp::UID(uid) = self.bytecode[self.idx] {
                        if let SwindleValue::Bool(false) = self.pop() {
                            self.idx = *self.labels.get(&uid).unwrap();
                        }
                    }
                }
                ByteCodeOp::Jump => {
                    self.idx += 1;
                    if let ByteCodeOp::UID(uid) = self.bytecode[self.idx] {
                        self.idx = *self.labels.get(&uid).unwrap();
                    }
                }
                _ => panic!("wut"),
            }

            self.idx += 1;
        }
    }
}
