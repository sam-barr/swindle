use crate::bytecode::*;
use crate::renamer::*;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum SwindleValue {
    Int(i64),
    ConstString(usize),
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

pub struct VM {
    bytecode: Vec<ByteCodeOp>,
    idx: usize,
    strings: Vec<String>,
    variables: Vec<SwindleValue>,
    stack: Vec<SwindleValue>,
    labels: Vec<usize>,
    heap: Heap,
}

impl VM {
    pub fn new(
        bytecode: Vec<ByteCodeOp>,
        string_ids: HashMap<String, UID>,
        num_variables: usize,
        num_labels: usize,
        num_strings: usize,
    ) -> Self {
        let mut labels = vec![Default::default(); num_labels];
        for (idx, op) in bytecode.iter().enumerate() {
            if let ByteCodeOp::Label(uid) = op {
                labels[*uid] = idx;
            }
        }

        let mut strings = vec![Default::default(); num_strings];
        for (string, uid) in string_ids.iter() {
            strings[uid.get_value()] = string.to_string();
        }

        VM {
            bytecode,
            idx: 0,
            strings,
            variables: vec![SwindleValue::Unit; num_variables],
            stack: Vec::new(),
            labels,
            heap: Heap::new(),
        }
    }

    pub fn debug(&self) {
        println!("~~~~~~~~VM~~~~~~~~");
        if self.idx < self.bytecode.len() {
            print!("Current op: {:?}", self.bytecode[self.idx]);
        }
        println!("Variables: {:?}", self.variables);
        println!("Stack: {:?}", self.stack);
        println!("Heap: {:?}", self.heap);
        println!("~~~~~~~~~~~~~~~~~~");
    }

    fn get_variable(&self, uid: usize) -> SwindleValue {
        self.variables[uid]
    }

    fn set_variable(&mut self, uid: usize, value: SwindleValue) -> SwindleValue {
        let old_value = self.variables[uid];
        self.variables[uid] = value;
        old_value
    }

    fn get_label(&self, uid: usize) -> usize {
        self.labels[uid]
    }

    fn get_conststring(&self, uid: usize) -> &String {
        &self.strings[uid]
    }

    fn display(&self, value: SwindleValue) -> String {
        match value {
            SwindleValue::ConstString(uid) => self.get_conststring(uid).to_string(),
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

    pub fn run(&mut self, debug: bool) {
        while let Some(&bc_op) = self.bytecode.get(self.idx) {
            if debug {
                self.debug();
                let mut buffer = String::new();
                std::io::stdin().read_line(&mut buffer).unwrap();
            }

            match bc_op {
                ByteCodeOp::IntConst(n) => self.push(SwindleValue::Int(n)),
                ByteCodeOp::StringConst(uid) => self.push(SwindleValue::ConstString(uid)),
                ByteCodeOp::BoolConst(b) => self.push(SwindleValue::Bool(b)),
                ByteCodeOp::Variable(uid) => {
                    let value = self.get_variable(uid);
                    self.push(value);
                    self.access(value); // value in variable and on stack
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
                        SwindleValue::ConstString(uid) => Some(self.get_conststring(uid)),
                        _ => {
                            self.push(SwindleValue::Bool(a == b));
                            None
                        }
                    } {
                        let s2 = match b {
                            SwindleValue::HeapString(uid) => self.heap.get(uid),
                            SwindleValue::ConstString(uid) => self.get_conststring(uid),
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
                        SwindleValue::ConstString(uid) => self.get_conststring(uid),
                        SwindleValue::HeapString(uid) => self.heap.get(uid),
                        _ => panic!(),
                    }
                    .to_string();
                    let s2 = match v2 {
                        SwindleValue::ConstString(uid) => self.get_conststring(uid),
                        SwindleValue::HeapString(uid) => self.heap.get(uid),
                        _ => panic!(),
                    };
                    s1.push_str(s2);
                    self.drop(v1);
                    self.drop(v2);
                    let uid = self.heap.alloc(s1);
                    self.push(SwindleValue::HeapString(uid));
                }

                ByteCodeOp::Declare(uid) => {
                    let value = self.pop();

                    let old_value = self.set_variable(uid, value);
                    self.drop(old_value);
                }
                ByteCodeOp::Assign(uid) => {
                    let value = self.pop();

                    let old_value = self.set_variable(uid, value);
                    self.drop(old_value);
                    self.push(value); // assignment returns the assigned value
                    self.access(value); // value is now in variable and on the stack
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
                ByteCodeOp::JumpIfFalse(uid) => {
                    if let SwindleValue::Bool(false) = self.pop() {
                        self.idx = self.get_label(uid);
                    }
                }
                ByteCodeOp::Jump(uid) => self.idx = self.get_label(uid),
            }

            self.idx += 1;
        }
    }
}
