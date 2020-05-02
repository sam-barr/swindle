#![allow(dead_code)]

use crate::ast::*;
use crate::tokenizer::*;
use std::boxed::Box;

type ParserResult<'a, A> = Option<(A, &'a [Token])>;

pub fn parse_program(tokens: &[Token]) -> Option<Program<String>> {
    let mut leftover_tokens = tokens;
    let mut statements = Vec::new();
    while let Some((statement, toks)) = parse_statement(leftover_tokens) {
        statements.push(statement);
        match toks.split_first() {
            Some((Token::Semicolon(), toks)) => leftover_tokens = toks,
            None => {
                leftover_tokens = toks;
                break;
            }
            _ => return None,
        }
    }

    if leftover_tokens.is_empty() {
        Some(Program { statements })
    } else {
        None
    }
}

fn parse_statement(tokens: &[Token]) -> ParserResult<Box<Statement<String>>> {
    //println!("statement {:?}", tokens);
    parse_type(tokens)
        .and_then(|(typ, tokens)| {
            tokens.split_first().and_then(|(tok, tokens)| match tok {
                Token::Variable(varname) => {
                    tokens.split_first().and_then(|(tok, tokens)| match tok {
                        Token::Assign() => parse_expression(tokens).map(|(expression, tokens)| {
                            (
                                Box::new(Statement::Declare(typ, varname.to_string(), expression)),
                                tokens,
                            )
                        }),
                        _ => None,
                    })
                }
                _ => None,
            })
        })
        .or_else(|| {
            tokens.split_first().and_then(|(tok, tokens)| match tok {
                Token::Write() => parse_expression(tokens)
                    .map(|(expression, tokens)| (Box::new(Statement::Write(expression)), tokens)),
                Token::Writeln() => parse_expression(tokens)
                    .map(|(expression, tokens)| (Box::new(Statement::Writeln(expression)), tokens)),
                _ => None,
            })
        })
        .or_else(|| {
            parse_expression(tokens)
                .map(|(expression, tokens)| (Box::new(Statement::Expression(expression)), tokens))
        })
}

fn parse_type(tokens: &[Token]) -> ParserResult<Type> {
    //println!("type {:?}", tokens);
    tokens.split_first().and_then(|(tok, tokens)| match tok {
        Token::IntType() => Some((Type::Int(), tokens)),
        Token::StringType() => Some((Type::String(), tokens)),
        Token::BoolType() => Some((Type::Bool(), tokens)),
        Token::UnitType() => Some((Type::Unit(), tokens)),
        _ => None,
    })
}

fn parse_expression(tokens: &[Token]) -> ParserResult<Box<Expression<String>>> {
    //println!("expression {:?}", tokens);
    tokens
        .split_first()
        .and_then(|(tok, tokens)| match tok {
            Token::Variable(varname) => tokens.split_first().and_then(|(tok, tokens)| match tok {
                Token::Assign() => parse_expression(tokens).map(|(expression, tokens)| {
                    (
                        Box::new(Expression::Assign(varname.to_string(), expression)),
                        tokens,
                    )
                }),
                _ => None,
            }),
            _ => None,
        })
        .or_else(|| {
            parse_orexp(tokens).map(|(orexp, tokens)| (Box::new(Expression::OrExp(orexp)), tokens))
        })
}

fn parse_orexp(tokens: &[Token]) -> ParserResult<Box<OrExp<String>>> {
    //println!("or {:?}", tokens);
    parse_andexp(tokens)
        .and_then(|(andexp, tokens)| {
            tokens
                .split_first()
                .and_then(|(op_tok, tokens)| match op_tok {
                    Token::Or() => parse_orexp(tokens)
                        .map(|(orexp, tokens)| (Box::new(OrExp::Or(andexp, orexp)), tokens)),
                    _ => None,
                })
        })
        .or_else(|| {
            parse_andexp(tokens).map(|(andexp, tokens)| (Box::new(OrExp::AndExp(andexp)), tokens))
        })
}

fn parse_andexp(tokens: &[Token]) -> ParserResult<Box<AndExp<String>>> {
    //println!("and {:?}", tokens);
    parse_compexp(tokens)
        .and_then(|(compexp, tokens)| {
            tokens
                .split_first()
                .and_then(|(op_tok, tokens)| match op_tok {
                    Token::And() => parse_andexp(tokens)
                        .map(|(andexp, tokens)| (Box::new(AndExp::And(compexp, andexp)), tokens)),
                    _ => None,
                })
        })
        .or_else(|| {
            parse_compexp(tokens)
                .map(|(compexp, tokens)| (Box::new(AndExp::CompExp(compexp)), tokens))
        })
}

fn parse_compexp(tokens: &[Token]) -> ParserResult<Box<CompExp<String>>> {
    //println!("comp {:?}", tokens);
    parse_addexp(tokens)
        .and_then(|(addexp1, tokens)| {
            tokens
                .split_first()
                .and_then(|(op_tok, tokens)| match op_tok {
                    Token::Leq() => parse_addexp(tokens).map(|(addexp2, tokens)| {
                        (Box::new(CompExp::Leq(addexp1, addexp2)), tokens)
                    }),
                    Token::Lt() => parse_addexp(tokens)
                        .map(|(addexp2, tokens)| (Box::new(CompExp::Lt(addexp1, addexp2)), tokens)),
                    Token::Eq() => parse_addexp(tokens)
                        .map(|(addexp2, tokens)| (Box::new(CompExp::Eq(addexp1, addexp2)), tokens)),
                    Token::Neq() => parse_addexp(tokens).map(|(addexp2, tokens)| {
                        (Box::new(CompExp::Neq(addexp1, addexp2)), tokens)
                    }),
                    Token::Gt() => parse_addexp(tokens)
                        .map(|(addexp2, tokens)| (Box::new(CompExp::Gt(addexp1, addexp2)), tokens)),
                    Token::Geq() => parse_addexp(tokens).map(|(addexp2, tokens)| {
                        (Box::new(CompExp::Geq(addexp1, addexp2)), tokens)
                    }),
                    _ => None,
                })
        })
        .or_else(|| {
            parse_addexp(tokens).map(|(addexp, tokens)| (Box::new(CompExp::AddExp(addexp)), tokens))
        })
}

fn parse_addexp(tokens: &[Token]) -> ParserResult<Box<AddExp<String>>> {
    //println!("add {:?}", tokens);
    parse_mulexp(tokens)
        .and_then(|(mulexp, tokens)| {
            tokens
                .split_first()
                .and_then(|(op_tok, tokens)| match op_tok {
                    Token::Sum() => parse_addexp(tokens)
                        .map(|(addexp, tokens)| (Box::new(AddExp::Sum(mulexp, addexp)), tokens)),
                    Token::Difference() => parse_addexp(tokens).map(|(addexp, tokens)| {
                        (Box::new(AddExp::Difference(mulexp, addexp)), tokens)
                    }),
                    _ => None,
                })
        })
        .or_else(|| {
            parse_mulexp(tokens).map(|(mulexp, tokens)| (Box::new(AddExp::MulExp(mulexp)), tokens))
        })
}

fn parse_mulexp(tokens: &[Token]) -> ParserResult<Box<MulExp<String>>> {
    //println!("mul {:?}", tokens);
    parse_unary(tokens)
        .and_then(|(unary, tokens)| {
            tokens
                .split_first()
                .and_then(|(op_tok, tokens)| match op_tok {
                    Token::Product() => parse_mulexp(tokens)
                        .map(|(mulexp, tokens)| (Box::new(MulExp::Product(unary, mulexp)), tokens)),
                    Token::Quotient() => parse_mulexp(tokens).map(|(mulexp, tokens)| {
                        (Box::new(MulExp::Quotient(unary, mulexp)), tokens)
                    }),
                    _ => None,
                })
        })
        .or_else(|| {
            parse_unary(tokens).map(|(unary, tokens)| (Box::new(MulExp::Unary(unary)), tokens))
        })
}

fn parse_unary(tokens: &[Token]) -> ParserResult<Box<Unary<String>>> {
    //println!("unary {:?}", tokens);
    tokens
        .split_first()
        .and_then(|(tok, tokens)| match tok {
            Token::Difference() => {
                parse_unary(tokens).map(|(unary, tokens)| (Box::new(Unary::Negate(unary)), tokens))
            }
            Token::Not() => {
                parse_unary(tokens).map(|(unary, tokens)| (Box::new(Unary::Not(unary)), tokens))
            }
            _ => None,
        })
        .or_else(|| {
            parse_primary(tokens)
                .map(|(primary, tokens)| (Box::new(Unary::Primary(primary)), tokens))
        })
}

fn parse_primary(tokens: &[Token]) -> ParserResult<Box<Primary<String>>> {
    //println!("primary {:?}", tokens);
    tokens.split_first().and_then(|(tok, tokens)| {
        macro_rules! mk {
            ($result:expr) => {
                Some((Box::new($result), tokens))
            };
        }

        match tok {
            //Token::IntLit(n) => Some((Box<Primary::IntLit(*n)>, tokens)),
            Token::IntLit(n) => mk!(Primary::IntLit(*n)),
            Token::True() => mk!(Primary::BoolLit(true)),
            Token::False() => mk!(Primary::BoolLit(false)),
            Token::StringLit(s) => mk!(Primary::StringLit(s.to_string())),
            Token::Variable(v) => mk!(Primary::Variable(v.to_string())),
            Token::LParen() => parse_expression(tokens)
                .and_then(|(expression, tokens)| {
                    tokens.split_first().and_then(|(tok, tokens)| match tok {
                        Token::RParen() => Some((Box::new(Primary::Paren(expression)), tokens)),
                        _ => None,
                    })
                })
                .or_else(|| {
                    tokens.split_first().and_then(|(tok, tokens)| match tok {
                        Token::RParen() => Some((Box::new(Primary::Unit()), tokens)),
                        _ => None,
                    })
                }),
            _ => None,
        }
    })
}
