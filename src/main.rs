use swindle::parser::parse_program;
use swindle::tokenizer::*;

fn main() {
    println!("{:?}", tokenize("1234 \"Hello, World!\\n\" 5678"));
    println!("{:?}", tokenize("and 1234   \n    or"));
    println!("{:?}", tokenize("1234 \"Hello, World!"));

    let tokens = tokenize("int x = 2; int y = x + 2; y");
    let result = parse_program(&tokens.unwrap());
    println!("{:?}", result);
    let tokens = tokenize("(x - 2) + y / 2");
    let result = parse_program(&tokens.unwrap());
    println!("{:?}", result);
}
