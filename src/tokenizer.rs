use crate::error::*;
use std::convert::TryFrom;
use std::str::Chars;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    // literals
    IntLit(i64),
    StringLit(String),
    Variable(String),
    LParen,
    RParen,
    LBrace,
    RBrace,

    // operands
    Quotient,
    Product,
    Remainder,
    Difference,
    Sum,
    Leq,
    Lt,
    Eq,
    Neq,
    Gt,
    Geq,
    Assign,
    Semicolon,
    Stringify,

    // Keywords
    IntType,
    StringType,
    BoolType,
    Unit,
    True,
    False,
    Write,
    Writeln,
    And,
    Or,
    Not,
    If,
    Elif,
    Else,
    While,
}

#[derive(Debug, Clone)]
pub struct PosnToken {
    pub token: Token,
    pub file_posn: FilePosition,
}

impl PosnToken {
    pub fn new(token: Token, file_posn: FilePosition) -> Self {
        PosnToken { token, file_posn }
    }
}

struct TokenizerState<'a> {
    file_posn: FilePosition,
    chars: Chars<'a>,
    buffer: Vec<char>,
}

// TODO: depending on how things go, the buffer could be optimized to Option<Char>
impl<'a> TokenizerState<'a> {
    fn new(string: &'a str) -> Self {
        TokenizerState {
            file_posn: FilePosition::new(),
            chars: string.chars(),
            buffer: Vec::new(),
        }
    }

    fn next(&mut self) -> Option<char> {
        let c = self.buffer.pop().or_else(|| self.chars.next());
        match c {
            Some('\n') => {
                self.file_posn.line += 1;
                self.file_posn.column = 0;
            }
            Some(_) => self.file_posn.column += 1,
            None => {}
        }

        c
    }

    fn push(&mut self, c: char) {
        // This is really naive and could crash at some point
        self.file_posn.column -= 1;
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

fn try_lex_keyword<F>(word: &str, chars: &mut TokenizerState, f: F) -> Option<Token>
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

#[allow(clippy::cognitive_complexity)]
pub fn tokenize(source: &str) -> Result<Vec<PosnToken>, SwindleError> {
    let mut chars = TokenizerState::new(source);
    let mut tokens = Vec::new();

    macro_rules! try_lex {
        ($string:expr, $result:ident) => {
            let posn = chars.file_posn;
            if let Some(t) = try_lex_keyword($string, &mut chars, || Token::$result) {
                tokens.push(PosnToken::new(t, posn));
                continue;
            }
        };
    }

    loop {
        chars.skip_whitespace();
        try_lex!("int", IntType);
        try_lex!("string", StringType);
        try_lex!("bool", BoolType);
        try_lex!("unit", Unit);
        try_lex!("true", True);
        try_lex!("false", False);
        try_lex!("writeln", Writeln);
        try_lex!("write", Write);
        try_lex!("and", And);
        try_lex!("or", Or);
        try_lex!("not", Not);
        try_lex!("==", Eq);
        try_lex!("!=", Neq);
        try_lex!(">=", Geq);
        try_lex!("<=", Leq);
        try_lex!("if", If);
        try_lex!("elif", Elif);
        try_lex!("else", Else);
        try_lex!("while", While);

        let posn = chars.file_posn;
        let c = match chars.next() {
            Some(c) => c,
            None => break,
        };

        if c == '/' {
            tokens.push(PosnToken::new(Token::Quotient, posn));
        } else if c == '*' {
            tokens.push(PosnToken::new(Token::Product, posn));
        } else if c == '%' {
            tokens.push(PosnToken::new(Token::Remainder, posn));
        } else if c == '-' {
            tokens.push(PosnToken::new(Token::Difference, posn));
        } else if c == '+' {
            tokens.push(PosnToken::new(Token::Sum, posn));
        } else if c == '<' {
            tokens.push(PosnToken::new(Token::Lt, posn));
        } else if c == '>' {
            tokens.push(PosnToken::new(Token::Gt, posn));
        } else if c == '=' {
            tokens.push(PosnToken::new(Token::Assign, posn));
        } else if c == ';' {
            tokens.push(PosnToken::new(Token::Semicolon, posn));
        } else if c == '$' {
            tokens.push(PosnToken::new(Token::Stringify, posn));
        } else if c == '(' {
            tokens.push(PosnToken::new(Token::LParen, posn));
        } else if c == ')' {
            tokens.push(PosnToken::new(Token::RParen, posn));
        } else if c == '{' {
            tokens.push(PosnToken::new(Token::LBrace, posn));
        } else if c == '}' {
            tokens.push(PosnToken::new(Token::RBrace, posn));
        } else if let Some(mut num) = c.to_digit(10) {
            while let Some(digit) = chars.next() {
                if let Some(digit) = digit.to_digit(10) {
                    num = num * 10 + digit;
                } else {
                    chars.push(digit);
                    break;
                }
            }
            match i64::try_from(num) {
                Ok(num) => tokens.push(PosnToken::new(Token::IntLit(num), posn)),
                Err(_) => {
                    return Err(SwindleError {
                        message: "integer literal is too large".to_string(),
                        file_posn: posn,
                        error_type: ErrorType::Tokenizer,
                    })
                }
            }
        } else if c == '"' {
            let mut string = String::new();
            let mut finished_string = false;
            while let Some(c) = chars.next() {
                if c == '"' {
                    finished_string = true;
                    break;
                } else if c == '\\' {
                    match chars.next() {
                        Some('"') => string.push('\"'),
                        Some('\\') => string.push('\\'),
                        Some('t') => string.push('\t'),
                        Some('n') => string.push('\n'),
                        _ => break,
                    }
                } else {
                    string.push(c);
                }
            }
            if finished_string {
                tokens.push(PosnToken::new(Token::StringLit(string), posn));
            } else {
                return Err(SwindleError {
                    message: "unexpected EOF while parsing string literal".to_string(),
                    file_posn: posn,
                    error_type: ErrorType::Tokenizer,
                });
            }
        } else if c.is_ascii_alphabetic() {
            let mut varname = c.to_string();
            while let Some(c2) = chars.next() {
                if c2.is_ascii_alphanumeric() {
                    varname.push(c2);
                } else {
                    chars.push(c2);
                    break;
                }
            }
            tokens.push(PosnToken::new(Token::Variable(varname), posn));
        } else {
            chars.push(c);
        }
    }

    Ok(tokens)
}
