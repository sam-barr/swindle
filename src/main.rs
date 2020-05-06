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
            let mut vm = VM::new(bytecode, strings);
            let debug = false;
            vm.run(debug);
            if debug {
                vm.debug();
            }
        }
        Err(e) => println!("{}", e),
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum SwindleValue {
    Int(i64),
    ConstString(UID),
    HeapString(UID),
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

#[derive(Debug)]
struct Heap {
    heap: HashMap<UID, (String, u32)>,
    free_uids: Vec<UID>,
    next_uid: UID,
}

impl Heap {
    fn new() -> Self {
        Heap {
            heap: HashMap::new(),
            free_uids: Vec::new(),
            next_uid: UID::new(),
        }
    }

    fn alloc(&mut self, string: String) -> UID {
        let uid = match self.free_uids.pop() {
            Some(uid) => uid,
            None => {
                let uid = self.next_uid;
                self.next_uid.inc();
                uid
            }
        };
        self.heap.insert(uid, (string, 1));
        uid
    }

    fn drop(&mut self, uid: UID) {
        if let Some(pair) = self.heap.get_mut(&uid) {
            pair.1 -= 1;
            if pair.1 == 0 {
                self.heap.remove(&uid).unwrap();
                self.free_uids.push(uid);
            }
        }
    }

    fn access(&mut self, uid: UID) {
        if let Some(pair) = self.heap.get_mut(&uid) {
            pair.1 += 1;
        }
    }

    fn get(&self, uid: UID) -> &String {
        &self.heap.get(&uid).unwrap().0
    }
}

struct VM {
    bytecode: Vec<ByteCodeOp>,
    idx: usize,
    strings: HashMap<UID, String>,
    variables: HashMap<UID, SwindleValue>,
    stack: Vec<SwindleValue>,
    labels: HashMap<UID, usize>,
    heap: Heap,
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
            heap: Heap::new(),
        }
    }

    fn debug(&self) {
        println!("~~~~~~~~VM~~~~~~~~");
        if self.idx < self.bytecode.len() {
            print!("Current op: {:?}", self.bytecode[self.idx]);
            match self.bytecode[self.idx] {
                ByteCodeOp::StringConst
                | ByteCodeOp::Variable
                | ByteCodeOp::Assign
                | ByteCodeOp::Declare
                | ByteCodeOp::JumpIfFalse => println!("({:?})", self.bytecode[self.idx + 1]),
                _ => println!(),
            }
        }
        println!("Variables: {:?}", self.variables);
        println!("Stack: {:?}", self.stack);
        println!("Heap: {:?}", self.heap);
        println!("~~~~~~~~~~~~~~~~~~");
    }

    fn display(&self, value: SwindleValue) -> String {
        match value {
            SwindleValue::ConstString(uid) => self.strings.get(&uid).unwrap().to_string(),
            SwindleValue::HeapString(uid) => self.heap.get(uid).to_string(),
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

    fn drop(&mut self, value: SwindleValue) {
        if let SwindleValue::HeapString(uid) = value {
            self.heap.drop(uid);
        }
    }

    fn access(&mut self, value: SwindleValue) {
        if let SwindleValue::HeapString(uid) = value {
            self.heap.access(uid);
        }
    }

    fn run(&mut self, debug: bool) {
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
                        let value = *self.variables.get(&uid).unwrap();
                        self.push(value);
                        self.access(value); // value in variable and on stack
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
                    if let Some(s1) = match a {
                        SwindleValue::HeapString(uid) => Some(self.heap.get(uid)),
                        SwindleValue::ConstString(uid) => Some(self.strings.get(&uid).unwrap()),
                        _ => {
                            self.push(SwindleValue::Bool(a == b));
                            None
                        }
                    } {
                        let s2 = match b {
                            SwindleValue::HeapString(uid) => self.heap.get(uid),
                            SwindleValue::ConstString(uid) => self.strings.get(&uid).unwrap(),
                            _ => panic!(),
                        };
                        let eq = s1 == s2;
                        self.push(SwindleValue::Bool(eq));
                        self.drop(a);
                        self.drop(b);
                    }
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

                ByteCodeOp::Stringify => {
                    let value = self.pop();
                    match value {
                        SwindleValue::Int(n) => {
                            let s = n.to_string();
                            let uid = self.heap.alloc(s);
                            self.push(SwindleValue::HeapString(uid));
                        }
                        SwindleValue::Bool(n) => {
                            let s = n.to_string();
                            let uid = self.heap.alloc(s);
                            self.push(SwindleValue::HeapString(uid));
                        }
                        SwindleValue::Unit => {
                            let s = "".to_string();
                            let uid = self.heap.alloc(s);
                            self.push(SwindleValue::HeapString(uid));
                        }
                        SwindleValue::ConstString(_) | SwindleValue::HeapString(_) => {
                            // just putt the value back on the stack, already a string
                            self.push(value)
                        }
                    }
                }
                ByteCodeOp::Append => {
                    let v1 = self.pop();
                    let v2 = self.pop();
                    let mut s1 = match v1 {
                        SwindleValue::ConstString(uid) => self.strings.get(&uid).unwrap(),
                        SwindleValue::HeapString(uid) => self.heap.get(uid),
                        _ => panic!(),
                    }
                    .to_string();
                    let s2 = match v2 {
                        SwindleValue::ConstString(uid) => self.strings.get(&uid).unwrap(),
                        SwindleValue::HeapString(uid) => self.heap.get(uid),
                        _ => panic!(),
                    };
                    s1.push_str(s2);
                    self.drop(v1);
                    self.drop(v2);
                    let uid = self.heap.alloc(s1);
                    self.push(SwindleValue::HeapString(uid));
                }

                ByteCodeOp::Declare => {
                    self.idx += 1;
                    if let ByteCodeOp::UID(uid) = self.bytecode[self.idx] {
                        let value = self.pop();

                        if let Some(old_value) = self.variables.insert(uid, value) {
                            self.drop(old_value);
                        }
                    }
                }
                ByteCodeOp::Assign => {
                    self.idx += 1;
                    if let ByteCodeOp::UID(uid) = self.bytecode[self.idx] {
                        let value = self.pop();

                        if let Some(old_value) = self.variables.insert(uid, value) {
                            self.drop(old_value);
                        }
                        self.push(value); // assignment returns the assigned value
                        self.access(value); // value is now in variable and on the stack
                    }
                }
                ByteCodeOp::Write => {
                    let v = self.pop();
                    print!("{}", self.display(v));
                    self.drop(v);
                }
                ByteCodeOp::Writeln => {
                    let v = self.pop();
                    println!("{}", self.display(v));
                    self.drop(v);
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
