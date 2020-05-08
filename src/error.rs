use std::default::Default;
use std::fmt;

#[derive(Debug, Copy, Clone)]
pub struct FilePosition {
    pub line: usize,
    pub column: usize,
}

impl FilePosition {
    pub fn new() -> Self {
        FilePosition { line: 0, column: 0 }
    }
}

impl Default for FilePosition {
    fn default() -> Self {
        FilePosition::new()
    }
}

pub enum ErrorType {
    Tokenizer,
    Parser,
    Typechecker,
}

pub struct SwindleError {
    pub message: String,
    pub file_posn: FilePosition,
    pub error_type: ErrorType,
}

impl fmt::Display for SwindleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_type = match self.error_type {
            ErrorType::Tokenizer => "lexer",
            ErrorType::Parser => "syntax",
            ErrorType::Typechecker => "type",
        };
        write!(
            f,
            "{} error at line {}, column {}: {}",
            error_type, self.file_posn.line, self.file_posn.column, self.message
        )
    }
}
