#![allow(dead_code)]

use crate::ast::*;
use crate::error::*;
use crate::tokenizer::*;
use std::boxed::Box;
use std::default::Default;

#[derive(Debug)]
pub struct Parsed {}

impl Tag for Parsed {
    type VariableTag = ();
    type WriteTag = ();
    type StatementTag = FilePosition;
}

type ParserResult<'a, A> = Result<(A, &'a [PosnToken]), SwindleError>;

fn throw_error<'a, A>(message: String, file_posn: FilePosition) -> ParserResult<'a, A> {
    Err(SwindleError {
        message,
        file_posn,
        error_type: ErrorType::Parser,
    })
}

fn bad_token<'a, A>(token: &PosnToken) -> ParserResult<'a, A> {
    throw_error(format!("bad token: {:?}", token.token), token.file_posn)
}

fn item<'a>(tokens: &'a [PosnToken]) -> ParserResult<'a, &PosnToken> {
    match tokens.split_first() {
        Some((tok, rest)) => Ok((tok, rest)),
        None => throw_error("unexpected eof".to_string(), FilePosition::new()),
    }
}

fn token_lit<'a>(tokens: &'a [PosnToken], expected: Token) -> ParserResult<'a, &PosnToken> {
    match item(tokens) {
        Ok((tok, tokens)) => {
            if tok.token == expected {
                Ok((tok, tokens))
            } else {
                throw_error(
                    format!("expected {:?}, found {:?}", expected, tok.token),
                    tok.file_posn,
                )
            }
        }
        err => err,
    }
}

fn many0<'a, A, P>(tokens: &'a [PosnToken], parser: P) -> (Vec<A>, &'a [PosnToken])
//ParserResult<'a, Vec<A>>
where
    P: Fn(&'a [PosnToken]) -> ParserResult<'a, A>,
{
    let mut tokens = tokens;
    let mut collected = Vec::new();

    loop {
        match parser(tokens) {
            Ok((a, toks)) => {
                tokens = toks;
                collected.push(a);
            }
            Err(_) => return (collected, tokens),
        }
    }
}

fn many1<'a, A, P>(tokens: &'a [PosnToken], parser: P) -> ParserResult<Vec<A>>
where
    P: Fn(&'a [PosnToken]) -> ParserResult<'a, A>,
{
    parser(tokens).and_then(|(a, tokens)| {
        let mut vec = vec![a];
        let (mut rest, tokens) = many0(tokens, parser);
        vec.append(&mut rest);
        Ok((vec, tokens))
    })
}

pub fn parse_program(tokens: &[PosnToken]) -> Result<Program<Parsed, String>, SwindleError> {
    let mut leftover_tokens = tokens;
    let mut statements = Vec::new();

    loop {
        // don't use many because error handling is different
        match parse_statement(leftover_tokens) {
            Ok((statement, tokens)) => {
                statements.push(TaggedStatement::new(
                    leftover_tokens[0].file_posn,
                    statement,
                ));
                leftover_tokens = tokens;
            }
            Err(e) => {
                // if its empty, that'll casue an error, but we don't want to die from that
                if !leftover_tokens.is_empty() {
                    return Err(e);
                } else {
                    break;
                }
            }
        }
    }

    match leftover_tokens.split_first() {
        None => Ok(Program { statements }),
        Some((tok, _)) => Err(SwindleError {
            message: "incomplete parse".to_string(),
            file_posn: tok.file_posn,
            error_type: ErrorType::Parser,
        }),
    }
}

fn parse_statement(tokens: &[PosnToken]) -> ParserResult<Statement<Parsed, String>> {
    parse_type(tokens)
        .and_then(|(typ, tokens)| {
            item(tokens).and_then(|(tok, tokens)| match &tok.token {
                Token::Variable(varname) => {
                    token_lit(tokens, Token::Assign).and_then(|(_, tokens)| {
                        parse_expression(tokens).map(|(expression, tokens)| {
                            (
                                Statement::Declare(typ, varname.to_string(), expression),
                                tokens,
                            )
                        })
                    })
                }
                _ => bad_token(tok),
            })
        })
        .or_else(|_| {
            item(tokens).and_then(|(tok, tokens)| match tok.token {
                Token::Write => parse_expression(tokens)
                    .map(|(expression, tokens)| (Statement::Write((), expression), tokens)),
                Token::Writeln => parse_expression(tokens)
                    .map(|(expression, tokens)| (Statement::Writeln((), expression), tokens)),
                _ => bad_token(tok),
            })
        })
        .or_else(|_| {
            parse_expression(tokens)
                .map(|(expression, tokens)| (Statement::Expression(expression), tokens))
        })
        .and_then(|(stmt, tokens)| {
            token_lit(tokens, Token::Semicolon).map(|(_, tokens)| (stmt, tokens))
        })
}

fn parse_type(tokens: &[PosnToken]) -> ParserResult<Type> {
    //println!("type {:?}", tokens);
    item(tokens).and_then(|(tok, tokens)| match tok.token {
        Token::IntType => Ok((Type::Int(), tokens)),
        Token::StringType => Ok((Type::String(), tokens)),
        Token::BoolType => Ok((Type::Bool(), tokens)),
        Token::Unit => Ok((Type::Unit(), tokens)),
        _ => bad_token(tok),
    })
}

fn parse_expression(tokens: &[PosnToken]) -> ParserResult<Box<Expression<Parsed, String>>> {
    //println!("expression {:?}", tokens);
    item(tokens)
        .and_then(|(tok, tokens)| match &tok.token {
            Token::Variable(varname) => item(tokens).and_then(|(tok, tokens)| match tok.token {
                Token::Assign => parse_expression(tokens).map(|(expression, tokens)| {
                    (
                        Box::new(Expression::Assign(varname.to_string(), expression)),
                        tokens,
                    )
                }),
                _ => bad_token(tok),
            }),
            _ => bad_token(tok),
        })
        .or_else(|_| {
            parse_whileexp(tokens)
                .map(|(whileexp, tokens)| (Box::new(Expression::WhileExp(whileexp)), tokens))
        })
        .or_else(|_| {
            parse_ifexp(tokens).map(|(ifexp, tokens)| (Box::new(Expression::IfExp(ifexp)), tokens))
        })
        .or_else(|_| {
            parse_orexp(tokens).map(|(orexp, tokens)| (Box::new(Expression::OrExp(orexp)), tokens))
        })
}

fn parse_whileexp(tokens: &[PosnToken]) -> ParserResult<Box<WhileExp<Parsed, String>>> {
    token_lit(tokens, Token::While).and_then(|(_, tokens)| {
        parse_expression(tokens).and_then(|(cond, tokens)| {
            parse_body(tokens).map(|(body, tokens)| (Box::new(WhileExp { cond, body }), tokens))
        })
    })
}

fn parse_ifexp(tokens: &[PosnToken]) -> ParserResult<Box<IfExp<Parsed, String>>> {
    token_lit(tokens, Token::If).and_then(|(_, tokens)| {
        parse_expression(tokens).and_then(|(cond, tokens)| {
            parse_body(tokens).and_then(|(body, tokens)| {
                let (elifs, tokens) = many0(tokens, parse_elif);
                let (els, tokens) = token_lit(tokens, Token::Else)
                    .and_then(|(_, tokens)| parse_body(tokens))
                    .unwrap_or_else(|_| (Default::default(), tokens));
                Ok((
                    Box::new(IfExp {
                        cond,
                        body,
                        elifs,
                        els,
                    }),
                    tokens,
                ))
            })
        })
    })
}

fn parse_body(tokens: &[PosnToken]) -> ParserResult<Body<Parsed, String>> {
    token_lit(tokens, Token::LBrace).and_then(|(_, tokens)| {
        let (statements, tokens) = many0(tokens, parse_statement);
        token_lit(tokens, Token::RBrace).map(|(_, tokens)| (Body { statements }, tokens))
    })
}

fn parse_elif(tokens: &[PosnToken]) -> ParserResult<Elif<Parsed, String>> {
    token_lit(tokens, Token::Elif).and_then(|(_, tokens)| {
        parse_expression(tokens).and_then(|(cond, tokens)| {
            parse_body(tokens).map(|(body, tokens)| (Elif { cond, body }, tokens))
        })
    })
}

fn parse_orexp(tokens: &[PosnToken]) -> ParserResult<Box<OrExp<Parsed, String>>> {
    //println!("or {:?}", tokens);
    parse_andexp(tokens)
        .and_then(|(andexp, tokens)| {
            token_lit(tokens, Token::Or).and_then(|(_, tokens)| {
                parse_orexp(tokens)
                    .map(|(orexp, tokens)| (Box::new(OrExp::Or(andexp, orexp)), tokens))
            })
        })
        .or_else(|_| {
            parse_andexp(tokens).map(|(andexp, tokens)| (Box::new(OrExp::AndExp(andexp)), tokens))
        })
}

fn parse_andexp(tokens: &[PosnToken]) -> ParserResult<Box<AndExp<Parsed, String>>> {
    //println!("and {:?}", tokens);
    parse_compexp(tokens)
        .and_then(|(compexp, tokens)| {
            token_lit(tokens, Token::And).and_then(|(_, tokens)| {
                parse_andexp(tokens)
                    .map(|(andexp, tokens)| (Box::new(AndExp::And(compexp, andexp)), tokens))
            })
        })
        .or_else(|_| {
            parse_compexp(tokens)
                .map(|(compexp, tokens)| (Box::new(AndExp::CompExp(compexp)), tokens))
        })
}

fn parse_compexp(tokens: &[PosnToken]) -> ParserResult<Box<CompExp<Parsed, String>>> {
    //println!("comp {:?}", tokens);
    parse_addexp(tokens)
        .and_then(|(addexp1, tokens)| {
            parse_compop(tokens).and_then(|(compop, tokens)| {
                parse_addexp(tokens).map(|(addexp2, tokens)| {
                    (Box::new(CompExp::Comp(compop, addexp1, addexp2)), tokens)
                })
            })
        })
        .or_else(|_| {
            parse_addexp(tokens).map(|(addexp, tokens)| (Box::new(CompExp::AddExp(addexp)), tokens))
        })
}

fn parse_compop(tokens: &[PosnToken]) -> ParserResult<CompOp> {
    item(tokens).and_then(|(tok, tokens)| match tok.token {
        Token::Leq => Ok((CompOp::Leq, tokens)),
        Token::Lt => Ok((CompOp::Lt, tokens)),
        Token::Eq => Ok((CompOp::Eq, tokens)),
        Token::Neq => Ok((CompOp::Neq, tokens)),
        Token::Gt => Ok((CompOp::Gt, tokens)),
        Token::Geq => Ok((CompOp::Geq, tokens)),
        _ => bad_token(tok),
    })
}

fn parse_addexp(tokens: &[PosnToken]) -> ParserResult<Box<AddExp<Parsed, String>>> {
    //println!("add {:?}", tokens);
    parse_mulexp(tokens)
        .and_then(|(mulexp, tokens)| {
            parse_addop(tokens).and_then(|(addop, tokens)| {
                parse_addexp(tokens)
                    .map(|(addexp, tokens)| (Box::new(AddExp::Add(addop, mulexp, addexp)), tokens))
            })
        })
        .or_else(|_| {
            parse_mulexp(tokens).map(|(mulexp, tokens)| (Box::new(AddExp::MulExp(mulexp)), tokens))
        })
}

fn parse_addop(tokens: &[PosnToken]) -> ParserResult<AddOp> {
    item(tokens).and_then(|(tok, tokens)| match tok.token {
        Token::Sum => Ok((AddOp::Sum, tokens)),
        Token::Difference => Ok((AddOp::Difference, tokens)),
        _ => bad_token(tok),
    })
}

fn parse_mulexp(tokens: &[PosnToken]) -> ParserResult<Box<MulExp<Parsed, String>>> {
    //println!("mul {:?}", tokens);
    parse_unary(tokens)
        .and_then(|(unary, tokens)| {
            parse_mulop(tokens).and_then(|(mulop, tokens)| {
                parse_mulexp(tokens)
                    .map(|(mulexp, tokens)| (Box::new(MulExp::Mul(mulop, unary, mulexp)), tokens))
            })
        })
        .or_else(|_| {
            parse_unary(tokens).map(|(unary, tokens)| (Box::new(MulExp::Unary(unary)), tokens))
        })
}

fn parse_mulop(tokens: &[PosnToken]) -> ParserResult<MulOp> {
    item(tokens).and_then(|(tok, tokens)| match tok.token {
        Token::Product => Ok((MulOp::Product, tokens)),
        Token::Quotient => Ok((MulOp::Quotient, tokens)),
        Token::Remainder => Ok((MulOp::Remainder, tokens)),
        _ => bad_token(tok),
    })
}

fn parse_unary(tokens: &[PosnToken]) -> ParserResult<Box<Unary<Parsed, String>>> {
    //println!("unary {:?}", tokens);
    item(tokens)
        .and_then(|(tok, tokens)| match tok.token {
            Token::Difference => {
                parse_unary(tokens).map(|(unary, tokens)| (Box::new(Unary::Negate(unary)), tokens))
            }
            Token::Not => {
                parse_unary(tokens).map(|(unary, tokens)| (Box::new(Unary::Not(unary)), tokens))
            }
            Token::Stringify => many1(tokens, parse_primary)
                .map(|(primaries, tokens)| (Box::new(Unary::Stringify(primaries)), tokens)),
            _ => throw_error("null".to_string(), tok.file_posn),
        })
        .or_else(|_| {
            parse_primary(tokens)
                .map(|(primary, tokens)| (Box::new(Unary::Primary(Box::new(primary))), tokens))
        })
}

fn parse_primary(tokens: &[PosnToken]) -> ParserResult<Primary<Parsed, String>> {
    //println!("primary {:?}", tokens);
    item(tokens).and_then(|(tok, tokens)| {
        macro_rules! mk {
            ($result:expr) => {
                Ok(($result, tokens))
            };
        }

        match &tok.token {
            //Token::IntLit(n) => Some((Box<Primary::IntLit(*n)>, tokens)),
            Token::IntLit(n) => mk!(Primary::IntLit(*n)),
            Token::True => mk!(Primary::BoolLit(true)),
            Token::False => mk!(Primary::BoolLit(false)),
            Token::StringLit(s) => mk!(Primary::StringLit(s.to_string())),
            Token::Variable(v) => mk!(Primary::Variable((), v.to_string())),
            Token::LParen => parse_expression(tokens).and_then(|(expression, tokens)| {
                token_lit(tokens, Token::RParen)
                    .map(|(_, tokens)| (Primary::Paren(expression), tokens))
            }),
            Token::Unit => mk!(Primary::Unit()),
            _ => bad_token(tok),
        }
    })
}
