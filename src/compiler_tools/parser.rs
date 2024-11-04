use rayon::prelude::*;
use std::{collections::VecDeque, fmt::Debug};

use super::{tokenizer::PositionedToken, ParseFile};

#[derive(Debug, PartialEq, Clone)]
pub struct Import {
    pub name: String,
    pub from: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportExport<Token> {
    token: Token,
    type_: ImportExportType,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ImportExportType {
    Internal,
    Exported,
    Imported(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct AST<Type, Value, Error> {
    pub types: Vec<ImportExport<Type>>,
    pub values: Vec<ImportExport<Value>>,
    pub errors: Vec<Error>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parser<Token, Type, Value, Error>(Vec<Token>, ParserFn<Token, Type, Value, Error>);

type ParserFn<Token, Type, Value, Error> =
    fn(VecDeque<PositionedToken<Token>>) -> ParseResult<Type, Value, Error>;

pub enum ParseResult<Type, Value, Error> {
    Type(Type),
    Value(Value),
    Error(Error),
}

pub enum ImportParseResult<Import, Error> {
    Success(Import),
    Error(Error),
    Failure,
}

pub fn parse<
    Token: PartialEq + Debug + Clone + Send,
    Type: PartialEq + Debug + Clone + Send,
    Value: PartialEq + Debug + Clone + Send,
    Error: PartialEq + Debug + Clone + Send,
>(
    tokens: Vec<PositionedToken<Token>>,
    import_token: Token,
    export_token: Token,
    type_token: Token,
    parse_import: fn(VecDeque<PositionedToken<Token>>) -> ImportParseResult<Import, Error>,
    parse_type: ParserFn<Token, Type, Value, Error>,
    parse_others: Vec<Parser<Token, Type, Value, Error>>,
    create_error: fn(PositionedToken<Token>, String, i64) -> Error,
    parse_file: ParseFile<AST<Type, Value, Error>>,
) -> AST<Type, Value, Error> {
    let mut types = vec![];
    let mut values = vec![];
    let mut errors = vec![];
    let mut split_on = vec![
        import_token.clone(),
        export_token.clone(),
        type_token.clone(),
    ];
    split_on.extend(parse_others.iter().flat_map(|p| p.0.clone()));
    let tokens = split_starting(tokens, split_on);
    let mut import_tokens = Vec::new();
    let mut type_tokens = Vec::new();
    let mut value_tokens = Vec::new();
    for mut token in tokens {
        if token[0].token == import_token {
            import_tokens.push(token);
            continue;
        }
        let mut exported = false;
        if token[0].token == export_token {
            exported = true;
            token.remove(0);
        }
        if token[0].token == type_token {
            type_tokens.push(ImportExport {
                token,
                type_: if exported {
                    ImportExportType::Exported
                } else {
                    ImportExportType::Internal
                },
            });
        } else {
            value_tokens.push(ImportExport {
                token,
                type_: if exported {
                    ImportExportType::Exported
                } else {
                    ImportExportType::Internal
                },
            });
        }
    }
    let imports: Vec<PositionedToken<Import>> = import_tokens
        .into_iter()
        .filter_map(|mut part| {
            part.remove(0);
            let line_no = part[0].line_no;
            let word_no = part[0].word_no;
            match parse_import(part.into()) {
                ImportParseResult::Success(import) => Some(PositionedToken {
                    line_no,
                    word_no,
                    token: import,
                }),
                ImportParseResult::Error(error) => {
                    errors.push(error);
                    None
                }
                ImportParseResult::Failure => None,
            }
        })
        .collect();
    let imported_files = imports
        .into_par_iter()
        .map(|import| {
            parse_file(import.token.from.clone())
                .map(|i| (import.token.from.clone(), i))
                .ok_or(import)
        })
        .collect::<Vec<_>>();
    for import in imported_files {
        match import {
            Ok((file, import)) => {
                types.extend(import.types.into_iter().map(|t| ImportExport {
                    token: t.token,
                    type_: ImportExportType::Imported(file.clone()),
                }));
                values.extend(import.values.into_iter().map(|v| ImportExport {
                    token: v.token,
                    type_: ImportExportType::Imported(file.clone()),
                }));
                errors.extend(import.errors);
            }
            Err(token) => {
                errors.push(create_error(
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
    for ImportExport { token, type_ } in type_tokens {
        match parse_type(token.into()) {
            ParseResult::Type(t) => types.push(ImportExport { token: t, type_ }),
            ParseResult::Value(v) => values.push(ImportExport { token: v, type_ }),
            ParseResult::Error(e) => errors.push(e),
        }
    }
    for ImportExport { token, type_ } in value_tokens {
        let mut found = false;
        let first = &token[0];
        for parser in &parse_others {
            if parser.0.contains(&first.token) {
                match (parser.1)(token.clone().into()) {
                    ParseResult::Type(t) => types.push(ImportExport { token: t, type_ }),
                    ParseResult::Value(v) => values.push(ImportExport { token: v, type_ }),
                    ParseResult::Error(e) => errors.push(e),
                }
                found = true;
                break;
            }
        }
        if !found {
            errors.push(create_error(
                first.clone(),
                "Unexpected top-level token".to_string(),
                0,
            ));
        }
    }
    AST {
        types: types
            .into_iter()
            .filter(|t| {
                if let ImportExportType::Exported = t.type_ {
                    true
                } else {
                    false
                }
            })
            .collect(),
        values: values
            .into_iter()
            .filter(|v| {
                if let ImportExportType::Exported = v.type_ {
                    true
                } else {
                    false
                }
            })
            .collect(),
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
