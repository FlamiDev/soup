use rayon::prelude::*;
use std::{collections::VecDeque, fmt::Debug};

use super::{tokenizer::PositionedToken, ParseFile};

#[derive(Debug, PartialEq, Clone)]
pub struct Import {
    pub name: String,
    pub from: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AST<Type, Value, Error> {
    pub types: Vec<Type>,
    pub values: Vec<Value>,
    pub errors: Vec<Error>,
}

pub enum ParseResult<Token, T> {
    Success(T),
    Error(PositionedToken<Token>, String, i64),
    Failure,
}

pub fn parse<
    Token: PartialEq + Debug + Clone + Send,
    Type: PartialEq + Debug + Clone + Send,
    Value: PartialEq + Debug + Clone + Send,
    Error: PartialEq + Debug + Clone + Send,
>(
    tokens: Vec<PositionedToken<Token>>,
    split_on: Vec<Token>,
    import_token: Token,
    parse_import: fn(VecDeque<PositionedToken<Token>>) -> ParseResult<Token, Import>,
    parse_file: ParseFile<AST<Type, Value, Error>>,
    error: fn(PositionedToken<Token>, String, i64) -> Error,
) -> AST<Type, Value, Error> {
    let mut types = vec![];
    let mut values = vec![];
    let mut errors = vec![];
    let tokens = split_starting(tokens, split_on);
    let imports: Vec<PositionedToken<Import>> = tokens
        .into_iter()
        .filter_map(|mut part| {
            if part[0].token != import_token {
                return None;
            }
            part.remove(0);
            let line_no = part[0].line_no;
            let word_no = part[0].word_no;
            match parse_import(part.into()) {
                ParseResult::Success(import) => Some(PositionedToken {
                    line_no,
                    word_no,
                    token: import,
                }),
                ParseResult::Error(token, message, line) => {
                    errors.push(error(token, message, line));
                    None
                }
                ParseResult::Failure => None,
            }
        })
        .collect();
    let imported_files = imports
        .into_par_iter()
        .map(|import| parse_file(import.token.from.clone()).ok_or(import))
        .collect::<Vec<_>>();
    for import in imported_files {
        match import {
            Ok(file) => {
                types.extend(file.types);
                values.extend(file.values);
                errors.extend(file.errors);
            }
            Err(token) => {
                errors.push(error(
                    PositionedToken {
                        token: import_token.clone(),
                        line_no: token.line_no,
                        word_no: token.word_no,
                    },
                    format!(
                        "Could not import file {} from {}",
                        token.token.name, token.token.from
                    ),
                    0,
                ));
            }
        }
    }
    AST {
        types,
        values,
        errors,
    }
}

fn split_starting<Token: PartialEq + Debug>(
    tokens: Vec<PositionedToken<Token>>,
    split_on: Vec<Token>,
) -> Vec<Vec<PositionedToken<Token>>> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    for token in tokens {
        if split_on.contains(&token.token) {
            if current
                .last()
                .map(|t: &PositionedToken<Token>| split_on.contains(&t.token))
                .unwrap_or(false)
            {
                current.push(token);
                continue;
            }
            if !current.is_empty() {
                result.push(current);
            }
            current = vec![token];
        } else {
            current.push(token);
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}
