use swindle::parser::parse_program;
use swindle::tokenizer::*;
use swindle::typechecker::*;

fn main() {
    let tokens = tokenize("int x = 0; unit y = (); writeln x+y; ");
    let result = parse_program(&tokens.unwrap());
    if let Some(program) = result {
        println!("{:?}", type_program(program));
    } else {
        println!("parsing failed");
    }
}
