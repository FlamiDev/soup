use super::tokenizer::PositionedToken;
use std::{collections::VecDeque, fmt::Debug};

#[derive(Debug, PartialEq, Clone)]
pub struct Import {
    pub name: String,
    pub from: String,
}

type ImportParser<Token, Error> =
    fn(PositionedToken<Token>, VecDeque<PositionedToken<Token>>) -> Result<Import, Error>;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, Clone)]
pub struct AST<Type: Debug, Value: Debug> {
    pub imports: Vec<Import>,
    pub types: Vec<Type>,
    pub values: Vec<Value>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parser<Token: Debug + PartialEq + Clone, Type, Value, Error>(
    pub Token,
    pub ParserFn<Token, Type, Value, Error>,
);

pub type ParserFn<Token, Type, Value, Error> = fn(
    Vec<(PositionedToken<Token>, VecDeque<PositionedToken<Token>>)>,
) -> ParserResult<Type, Value, Error>;

pub enum ParserResult<Type, Value, Error> {
    Type(Type),
    Value(Value),
    Error(Vec<Error>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParseResult<AST: Debug, Error: Debug>(
    pub Vec<PositionedToken<String>>,
    pub AST,
    pub Vec<Error>,
);

pub fn parse<
    Token: PartialEq + Debug + Clone,
    Type: PartialEq + Debug + Clone,
    Value: PartialEq + Debug + Clone,
    Error: PartialEq + Debug + Clone,
>(
    tokens: Vec<PositionedToken<Token>>,
    newline_token: Token,
    import_token: Token,
    parse_import: ImportParser<Token, Error>,
    take_next: Vec<Token>,
    parsers: Vec<Parser<Token, Type, Value, Error>>,
    create_error: fn(PositionedToken<Token>, String, i64) -> Error,
) -> ParseResult<AST<Type, Value>, Error> {
    let mut imports = Vec::new();
    let mut types = Vec::new();
    let mut values = Vec::new();
    let mut errors = Vec::new();
    let mut split_on = vec![import_token.clone()];
    split_on.extend(take_next.clone());
    split_on.extend(parsers.iter().map(|p| p.0.clone()));
    let split_tokens = split_starting(tokens, split_on);
    let mut import_tokens = Vec::new();
    let mut other_tokens = Vec::new();
    for tokens in split_tokens {
        if tokens[0].token == import_token {
            import_tokens.push(tokens);
            continue;
        }
        other_tokens.push(tokens);
    }
    let parsed_imports: Vec<PositionedToken<String>> = import_tokens
        .into_iter()
        .filter_map(|mut part| {
            let first = part.remove(0);
            match parse_import(first.clone(), part.into()) {
                Ok(import) => {
                    imports.push(import.clone());
                    Some(PositionedToken {
                        token: import.from,
                        line_no: first.line_no,
                        word_no: first.word_no,
                    })
                }
                Err(error) => {
                    errors.push(error);
                    None
                }
            }
        })
        .collect();
    let mut current_tokens = Vec::new();
    for tokens in other_tokens {
        let Some(start) = tokens.iter().position(|i| i.token != newline_token) else {
            continue;
        };
        let Some(end) = tokens.iter().rposition(|i| i.token != newline_token) else {
            continue;
        };
        if start > end {
            continue;
        }
        let mut tokens: VecDeque<PositionedToken<Token>> = tokens[start..=end].to_vec().into();
        let Some(first) = tokens.pop_front() else {
            continue;
        };
        if take_next.contains(&first.token) {
            current_tokens.push((first, tokens));
            continue;
        }
        let mut found = false;
        for parser in &parsers {
            if parser.0 == (first.token) {
                current_tokens.push((first.clone(), tokens));
                match parser.1(current_tokens) {
                    ParserResult::Type(t) => types.push(t),
                    ParserResult::Value(v) => values.push(v),
                    ParserResult::Error(e) => errors.extend(e),
                }
                current_tokens = Vec::new();
                found = true;
                break;
            }
        }
        if !found {
            errors.push(create_error(
                first,
                "Unexpected top-level token".to_string(),
                -100,
            ));
        }
    }
    ParseResult(
        parsed_imports,
        AST {
            imports,
            types,
            values,
        },
        errors,
    )
}

pub fn split_starting<Token: PartialEq + Debug>(
    tokens: Vec<PositionedToken<Token>>,
    split_on: Vec<Token>,
) -> Vec<Vec<PositionedToken<Token>>> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    for token in tokens {
        if split_on.contains(&token.token) {
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
