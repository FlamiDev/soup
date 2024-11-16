use rayon::prelude::*;
use std::{collections::VecDeque, fmt::Debug};

use super::{tokenizer::PositionedToken, ParseFile};

#[derive(Debug, PartialEq, Clone)]
pub struct Import {
    pub name: String,
    pub from: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportExport<Token: Debug> {
    pub token: Token,
    type_: ImportExportType,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ImportExportType {
    Internal,
    Exported,
    Imported(Import),
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, Clone)]
pub struct AST<Type: Debug, Value: Debug, Error: Debug> {
    pub types: Vec<ImportExport<Type>>,
    pub values: Vec<ImportExport<Value>>,
    pub errors: Vec<Error>,
}

pub enum ImportParseResult<Import, Error> {
    Success(Import),
    Error(Error),
    Failure,
}

type TypeParserFn<Token, Type, Error> = fn(
    PositionedToken<Token>,
    VecDeque<PositionedToken<Token>>,
    &Vec<ImportExport<Type>>,
) -> TypeParseResult<Type, Error>;

pub enum TypeParseResult<Type, Error> {
    Type(Type),
    Error(Error),
}

pub struct ValueParser<Token, Type: Debug, Value: Debug, Take, Error>(
    pub Vec<Token>,
    pub ValueParserFn<Token, Type, Value, Take, Error>,
);

type ValueParserFn<Token, Type, Value, Take, Error> = fn(
    PositionedToken<Token>,
    VecDeque<PositionedToken<Token>>,
    &mut Vec<Take>,
    &Vec<ImportExport<Type>>,
    &Vec<ImportExport<Value>>,
) -> ValueParseResult<Value, Take, Error>;

pub enum ValueParseResult<Value, Take, Error> {
    Value(Value),
    TakeToNext(Take),
    Error(Vec<Error>),
}

#[allow(clippy::too_many_arguments)]
pub fn parse<
    Token: PartialEq + Debug + Clone + Send,
    Type: PartialEq + Debug + Clone + Send,
    Value: PartialEq + Debug + Clone + Send,
    Take: PartialEq + Debug + Clone + Send,
    Error: PartialEq + Debug + Clone + Send,
>(
    tokens: Vec<PositionedToken<Token>>,
    newline_token: Token,
    import_token: Token,
    export_token: Token,
    type_token: Token,
    predefined_types: Vec<Type>,
    parse_import: fn(VecDeque<PositionedToken<Token>>) -> ImportParseResult<Import, Error>,
    parse_type: TypeParserFn<Token, Type, Error>,
    parse_others: Vec<ValueParser<Token, Type, Value, Take, Error>>,
    create_error: fn(PositionedToken<Token>, String) -> Error,
    parse_file: ParseFile<'_, AST<Type, Value, Error>>,
) -> AST<Type, Value, Error> {
    let mut types: Vec<ImportExport<Type>> = predefined_types
        .into_iter()
        .map(|t| ImportExport {
            token: t,
            type_: ImportExportType::Internal,
        })
        .collect();
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
        if token.is_empty() {
            continue;
        }
        let first = token.remove(0);
        if first.token == type_token {
            type_tokens.push(ImportExport {
                token: (first, token),
                type_: if exported {
                    ImportExportType::Exported
                } else {
                    ImportExportType::Internal
                },
            });
        } else {
            value_tokens.push(ImportExport {
                token: (first, token),
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
                .map(|i| (import.token.clone(), i))
                .ok_or(import)
        })
        .collect::<Vec<_>>();
    for import in imported_files {
        match import {
            Ok((import, tree)) => {
                types.extend(tree.types.into_iter().map(|t| ImportExport {
                    token: t.token,
                    type_: ImportExportType::Imported(import.clone()),
                }));
                values.extend(tree.values.into_iter().map(|v| ImportExport {
                    token: v.token,
                    type_: ImportExportType::Imported(import.clone()),
                }));
                errors.extend(tree.errors);
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
                ));
            }
        }
    }
    for ImportExport { mut token, type_ } in type_tokens {
        if token.0.token == newline_token {
            continue;
        }
        while let Some(PositionedToken { token: t, .. }) = token.1.last() {
            if t == &newline_token {
                token.1.pop();
            } else {
                break;
            }
        }
        match parse_type(token.0, token.1.into(), &types) {
            TypeParseResult::Type(t) => types.push(ImportExport { token: t, type_ }),
            TypeParseResult::Error(e) => errors.push(e),
        }
    }
    let mut take_to_next = Vec::new();
    for ImportExport { mut token, type_ } in value_tokens {
        if token.0.token == newline_token {
            continue;
        }
        while let Some(PositionedToken { token: t, .. }) = token.1.last() {
            if t == &newline_token {
                token.1.pop();
            } else {
                break;
            }
        }
        let mut found = false;
        let first = &token.0;
        for parser in &parse_others {
            if parser.0.contains(&first.token) {
                match (parser.1)(
                    token.0.clone(),
                    token.1.into(),
                    &mut take_to_next,
                    &types,
                    &values,
                ) {
                    ValueParseResult::Value(v) => values.push(ImportExport { token: v, type_ }),
                    ValueParseResult::TakeToNext(t) => take_to_next.push(t),
                    ValueParseResult::Error(e) => errors.extend(e),
                }
                found = true;
                break;
            }
        }
        if !found {
            errors.push(create_error(
                first.clone(),
                "Unexpected top-level token".to_string(),
            ));
        }
    }
    AST {
        types: types
            .into_iter()
            .filter(|t| matches!(t.type_, ImportExportType::Exported))
            .collect(),
        values: values
            .into_iter()
            .filter(|v| matches!(v.type_, ImportExportType::Exported))
            .collect(),
        errors,
    }
}

pub fn split_starting<Token: PartialEq + Debug>(
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
