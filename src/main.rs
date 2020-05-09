#![allow(dead_code)]
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::exit;
//use swindle::renamer::*;
use swindle::typechecker::*;

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
            println!("{:?}", program);
            //let (_program, num_variables) = rename_program(program);
            //println!("{:?}", num_variables);
        }
        Err(e) => println!("{}", e),
    }
}
