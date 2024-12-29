use std::collections::VecDeque;

use crate::compiler_tools::parser::{ParseResult, Parser, ParserResult, AST};
use crate::{
    compiler_tools::{
        parser::{self, Import},
        tokenizer::PositionedToken,
    },
    tokenizer::Token,
    type_parser::{parse_type_def, TypeDef},
    value_parser::{parse_value_def, ValueDef},
};

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError {
    pub line_no: i64,
    pub word_no: i64,
    pub token: Token,
    pub why: String,
    pub priority: i64,
}

pub fn parse_error(token: PositionedToken<Token>, why: &str, priority: i64) -> ParseError {
    ParseError {
        line_no: token.line_no,
        word_no: token.word_no,
        token: token.token,
        why: why.to_string(),
        priority,
    }
}

pub fn err<T>(token: PositionedToken<Token>, why: &str, priority: i64) -> Result<T, ParseError> {
    Err(parse_error(token, why, priority))
}

pub fn error_vec(token: PositionedToken<Token>, why: &str, priority: i64) -> Vec<ParseError> {
    vec![parse_error(token, why, priority)]
}

pub fn err_vec<T>(
    token: PositionedToken<Token>,
    why: &str,
    priority: i64,
) -> Result<T, Vec<ParseError>> {
    Err(error_vec(token, why, priority))
}

pub fn error(
    token: PositionedToken<Token>,
    why: &str,
    priority: i64,
) -> ParserResult<TypeDef, ValueDef, ParseError> {
    ParserResult::Error(error_vec(token, why, priority))
}

pub fn parse(
    tokens: Vec<PositionedToken<Token>>,
) -> ParseResult<AST<TypeDef, ValueDef>, ParseError> {
    parser::parse(
        tokens,
        Token::NewLine,
        Token::ImportKeyword,
        parse_import,
        vec![Token::DocKeyword, Token::TestKeyword, Token::ExportKeyword],
        vec![
            Parser(Token::TypeKeyword, parse_type_def),
            //Parser(Token::TraitKeyword, parse_trait_def),
            Parser(Token::LetKeyword, parse_value_def),
        ],
        |token, message, priority| ParseError {
            line_no: token.line_no,
            word_no: token.word_no,
            token: token.token,
            why: message,
            priority,
        },
    )
}

fn parse_import(
    token: PositionedToken<Token>,
    mut tokens: VecDeque<PositionedToken<Token>>,
) -> Result<Import, ParseError> {
    let Token::ImportKeyword = token.token else {
        return Err(parse_error(token, "Expected import keyword", 0));
    };
    let Some(token) = tokens.pop_front() else {
        return Err(parse_error(
            token,
            "Expected type name after import keyword",
            0,
        ));
    };
    let Token::Type(ref name) = token.token else {
        return Err(parse_error(
            token,
            "Expected type name after import keyword",
            0,
        ));
    };
    let Some(token) = tokens.pop_front() else {
        return Err(parse_error(token, "Expected file path after type name", 0));
    };
    let Token::String(path) = token.token else {
        return Err(parse_error(token, "Expected file path after type name", 0));
    };
    if let Some(end) = tokens.pop_front() {
        let Token::NewLine = end.token else {
            return Err(parse_error(
                end,
                "Expected newline after import statement",
                0,
            ));
        };
    }
    Ok(Import {
        name: name.clone(),
        from: path,
    })
}
