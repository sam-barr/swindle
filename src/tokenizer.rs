use std::str::Chars;

#[derive(Debug, Clone)]
pub enum Token {
    // literals
    IntLit(u32),
    StringLit(String),
    Variable(String),
    LParen(),
    RParen(),

    // operands
    Quotient(),
    Product(),
    Difference(),
    Sum(),
    Leq(),
    Lt(),
    Eq(),
    Neq(),
    Gt(),
    Geq(),
    Assign(),
    Semicolon(),

    // Keywords
    IntType(),
    StringType(),
    BoolType(),
    UnitType(),
    True(),
    False(),
    Write(),
    Writeln(),
    And(),
    Or(),
    Not(),
}

struct PushableChars<'a> {
    chars: Chars<'a>,
    buffer: Vec<char>,
}

// TODO: depending on how things go, the buffer could be optimized to Option<Char>
impl<'a> PushableChars<'a> {
    fn new(string: &'a str) -> Self {
        PushableChars {
            chars: string.chars(),
            buffer: Vec::new(),
        }
    }

    fn next(&mut self) -> Option<char> {
        self.buffer.pop().or_else(|| self.chars.next())
        //let c = self.buffer.pop().or_else(|| self.chars.next());
        //println!("{:?}", c);
        //c
    }

    fn push(&mut self, c: char) {
        self.buffer.push(c);
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.next() {
            if !c.is_whitespace() {
                self.push(c);
                break;
            }
        }
    }
}

fn try_lex_keyword<F>(word: &str, chars: &mut PushableChars, f: F) -> Option<Token>
where
    F: FnOnce() -> Token,
{
    let mut used_chars = Vec::new();
    for c1 in word.chars() {
        if let Some(c2) = chars.next() {
            used_chars.push(c2);
            if c1 != c2 {
                while let Some(c) = used_chars.pop() {
                    chars.push(c);
                }
                return None;
            }
        } else {
            while let Some(c) = used_chars.pop() {
                chars.push(c);
            }
            return None;
        }
    }

    Some(f())
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, &str> {
    let mut chars = PushableChars::new(source);
    let mut tokens = Vec::new();

    macro_rules! try_lex {
        ($string:expr, $result:ident) => {
            if let Some(t) = try_lex_keyword($string, &mut chars, Token::$result) {
                tokens.push(t);
            }
        };
    }

    loop {
        chars.skip_whitespace();
        try_lex!("int", IntType);
        try_lex!("string", StringType);
        try_lex!("bool", BoolType);
        try_lex!("unit", UnitType);
        try_lex!("true", True);
        try_lex!("false", False);
        try_lex!("write", Write);
        try_lex!("writeln", Writeln);
        try_lex!("and", And);
        try_lex!("or", Or);
        try_lex!("not", Not);
        try_lex!("==", Eq);
        try_lex!("!=", Neq);
        try_lex!(">=", Geq);
        try_lex!("<=", Leq);
        chars.skip_whitespace();

        let c = match chars.next() {
            Some(c) => c,
            None => break,
        };

        if c == '/' {
            tokens.push(Token::Quotient());
        } else if c == '*' {
            tokens.push(Token::Product());
        } else if c == '-' {
            tokens.push(Token::Difference());
        } else if c == '+' {
            tokens.push(Token::Sum());
        } else if c == '<' {
            tokens.push(Token::Lt());
        } else if c == '>' {
            tokens.push(Token::Gt());
        } else if c == '=' {
            tokens.push(Token::Assign());
        } else if c == ';' {
            tokens.push(Token::Semicolon());
        } else if c == '(' {
            tokens.push(Token::LParen());
        } else if c == ')' {
            tokens.push(Token::RParen());
        } else if let Some(mut num) = c.to_digit(10) {
            while let Some(digit) = chars.next() {
                if let Some(digit) = digit.to_digit(10) {
                    num = num * 10 + digit;
                } else {
                    chars.push(digit);
                    break;
                }
            }
            tokens.push(Token::IntLit(num));
        } else if c == '"' {
            let mut string = String::new();
            let mut finished_string = false;
            while let Some(c) = chars.next() {
                if c == '"' {
                    finished_string = true;
                    break;
                } else if c == '\\' {
                    string.push(c);
                    match chars.next() {
                        Some(c) => string.push(c),
                        None => break,
                    }
                } else {
                    string.push(c);
                }
            }
            if finished_string {
                tokens.push(Token::StringLit(string));
            } else {
                return Err("Unexpected EOF while parsing string literal");
            }
        } else if c.is_ascii_alphabetic() {
            let mut varname = c.to_string();
            while let Some(c2) = chars.next() {
                if c2.is_ascii_alphanumeric() {
                    varname.push(c);
                } else {
                    chars.push(c2);
                    break;
                }
            }
            tokens.push(Token::Variable(varname));
        } else {
            chars.push(c);
        }
    }

    Ok(tokens)
}
