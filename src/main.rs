#![allow(dead_code)]
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::exit;
use swindle::bytecode::*;
use swindle::renamer::*;
use swindle::typechecker::*;
use swindle::vm::*;

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(#[allow(clippy::all)] pub parser);

fn main() {
    let code = {
        let file_name = env::args().collect::<Vec<_>>().pop().unwrap();
        let mut file = File::open(&file_name).unwrap();
        let mut code = String::new();
        file.read_to_string(&mut code).unwrap();
        code
    };

    let parsed = parser::ProgramParser::new().parse(&code);
    let result = match parsed {
        Ok(p) => type_program(p),
        Err(err) => {
            println!("{:?}", err);
            exit(1);
        }
    };

    match result {
        Ok(program) => {
            let (program, num_variables) = rename_program(program);
            let (bytecode, strings, num_labels, num_strings) = byte_program(program);
            //unsafe {
            //    build_llvm(&bytecode, &[], num_variables);
            //}
            for i in 0..bytecode.len() {
                println!("{:?}", bytecode[i]);
            }
            let mut vm = VM::new(bytecode, strings, num_variables, num_labels, num_strings);
            vm.run(false);
            vm.debug();
        }
        Err(e) => println!("{}", e),
    }
}
