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
    mut parse_file: ParseFile<AST<Type, Value, Error>>,
    error: fn(PositionedToken<Token>, String, i64) -> Error,
) -> AST<Type, Value, Error> {
    let mut types = vec![];
    let mut values = vec![];
    let mut errors = vec![];
    let tokens = split_starting(tokens, split_on);
    let imports: Vec<Import> = tokens
        .into_iter()
        .filter_map(|mut part| {
            if part[0].token != import_token {
                return None;
            }
            part.remove(0);
            match parse_import(part.into()) {
                ParseResult::Success(import) => Some(import),
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
        .map(|import| (import.name, parse_file(import.from.clone())))
        .collect::<Vec<_>>();
    AST {
        types,
        values,
        errors,
    }
}

fn split_starting<Token: PartialEq>(
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
            if !result.is_empty() {
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
