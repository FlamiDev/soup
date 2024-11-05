use std::collections::{HashMap, VecDeque};

use crate::{
    compiler_tools::{
        parser::{self, Import, ImportExport, ImportParseResult, ParseResult, AST},
        tokenizer::PositionedToken,
        ParseFile,
    },
    tokenizer::Token,
};

#[derive(Debug, PartialEq, Clone)]
pub struct TypeDef {
    pub name: String,
    pub args: Vec<String>,
    pub value: Type,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Builtin(String),
    Generic(String),
    CompilerFigureItOut,
    Union(Vec<(String, Option<Type>)>),
    Tuple(Vec<Type>),
    NamedTuple(Vec<(String, Type)>),
    JoinedTuples(Vec<Type>, Vec<Type>),
    JoinedNamedTuples(Vec<Type>, Vec<(String, Type)>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError {
    pub line_no: i64,
    pub word_no: i64,
    pub token: Token,
    pub why: String,
    pub priority: i64,
}

type TypeType<'l> = TypeDef;
type ValueType<'l> = String;
type ErrorType = ParseError;

fn parse_error(token: PositionedToken<Token>, why: &str, priority: i64) -> ErrorType {
    ParseError {
        line_no: token.line_no,
        word_no: token.word_no,
        token: token.token,
        why: why.to_string(),
        priority,
    }
}

fn err<T>(token: PositionedToken<Token>, why: &str, priority: i64) -> Result<T, ErrorType> {
    Err(ParseError {
        line_no: token.line_no,
        word_no: token.word_no,
        token: token.token,
        why: why.to_string(),
        priority,
    })
}

fn error<'l>(
    token: PositionedToken<Token>,
    why: &str,
    priority: i64,
) -> ParseResult<TypeType<'l>, ValueType<'l>, ErrorType> {
    ParseResult::Error(parse_error(token, why, priority))
}

pub fn parse<'l>(
    tokens: Vec<PositionedToken<Token>>,
    parse_file: ParseFile<'l, AST<TypeType<'l>, ValueType<'l>, ErrorType>>,
) -> AST<TypeType<'l>, ValueType<'l>, ErrorType> {
    parser::parse(
        tokens,
        Token::NewLine,
        Token::ImportKeyword,
        Token::ExportKeyword,
        Token::TypeKeyword,
        vec![
            TypeDef {
                name: "Int".to_string(),
                args: Vec::new(),
                value: Type::Builtin("Int".to_string()),
            },
            TypeDef {
                name: "Float".to_string(),
                args: Vec::new(),
                value: Type::Builtin("Float".to_string()),
            },
            TypeDef {
                name: "String".to_string(),
                args: Vec::new(),
                value: Type::Builtin("String".to_string()),
            },
        ],
        parse_import,
        parse_type_def,
        vec![],
        |token, message| ErrorType {
            line_no: token.line_no,
            word_no: token.word_no,
            token: token.token,
            why: message,
            priority: 0,
        },
        parse_file,
    )
}

fn parse_import(
    mut tokens: VecDeque<PositionedToken<Token>>,
) -> ImportParseResult<Import, ErrorType> {
    let Some(token) = tokens.pop_front() else {
        return ImportParseResult::Failure;
    };
    let Token::Type(ref name) = token.token else {
        return ImportParseResult::Error(ErrorType {
            line_no: token.line_no,
            word_no: token.word_no,
            token: token.token,
            why: "Expected type name after import keyword".to_string(),
            priority: 0,
        });
    };
    let Some(token) = tokens.pop_front() else {
        return ImportParseResult::Error(ErrorType {
            line_no: token.line_no,
            word_no: token.word_no,
            token: token.token,
            why: "Expected file path after type name".to_string(),
            priority: 0,
        });
    };
    let Token::String(path) = token.token else {
        return ImportParseResult::Error(ErrorType {
            line_no: token.line_no,
            word_no: token.word_no,
            token: token.token,
            why: "Expected file path after type name".to_string(),
            priority: 0,
        });
    };
    if let Some(end) = tokens.pop_front() {
        let Token::NewLine = end.token else {
            return ImportParseResult::Error(ErrorType {
                line_no: end.line_no,
                word_no: end.word_no,
                token: end.token,
                why: "Expected newline after import statement".to_string(),
                priority: 0,
            });
        };
    }
    ImportParseResult::Success(Import {
        name: name.clone(),
        from: path,
    })
}

fn parse_type_def<'l>(
    first_token: PositionedToken<Token>,
    mut tokens: VecDeque<PositionedToken<Token>>,
    current_types: &Vec<ImportExport<TypeType<'l>>>,
    _current_values: &Vec<ImportExport<ValueType<'l>>>,
) -> ParseResult<TypeType<'l>, ValueType<'l>, ErrorType> {
    let Token::TypeKeyword = first_token.token else {
        return error(first_token, "Expected type keyword", 0);
    };
    let Some(name_token) = tokens.pop_front() else {
        return error(first_token, "Expected type name", 0);
    };
    let Token::Type(ref name) = name_token.token else {
        return error(name_token, "Expected type name", 0);
    };
    let mut args = Vec::new();
    while let Some(token) = tokens.pop_front() {
        let Token::Type(arg) = token.token else {
            tokens.push_front(token);
            break;
        };
        args.push(arg);
    }
    let Some(equals_token) = tokens.pop_front() else {
        return error(name_token, "Expected equals sign", 0);
    };
    let Token::EqualsSign = equals_token.token else {
        return error(equals_token, "Expected equals sign", 0);
    };
    if tokens.is_empty() {
        return error(equals_token, "Expected type body", 0);
    }
    let body = match parse_type(tokens, current_types) {
        Ok(t) => t,
        Err(e) => return ParseResult::Error(e),
    };
    ParseResult::Type(TypeDef {
        name: name.clone(),
        args,
        value: body,
    })
}

fn parse_type<'l>(
    mut tokens: VecDeque<PositionedToken<Token>>,
    current_types: &Vec<ImportExport<TypeDef>>,
) -> Result<Type, ErrorType> {
    let first = tokens.front().unwrap().clone();
    match first.token {
        Token::Parens(inner) => parse_tuple_type(inner, current_types),
        Token::Type(_) => {
            if tokens.iter().any(|t| t.token == Token::VerticalBar) {
                parse_union_type(tokens, current_types)
            } else if tokens.iter().any(|t| t.token == Token::Plus) {
                parse_joined_tuple_type(tokens, current_types)
            } else {
                tokens.pop_front();
                parse_type_ref(first, tokens, current_types)
            }
        }
        _ => err(first, "Invalid type", 0),
    }
}

fn parse_joined_tuple_type<'l>(
    mut tokens: VecDeque<PositionedToken<Token>>,
    current_types: &Vec<ImportExport<TypeDef>>,
) -> Result<Type, ErrorType> {
    let mut current_type = VecDeque::new();
    let mut joined = Vec::new();
    let last = tokens.back().unwrap().clone();
    while let Some(token) = tokens.pop_front() {
        match token.token {
            Token::Type(_) => {
                current_type.push_back(token);
            }
            Token::Plus => {
                let Some(first) = current_type.pop_front() else {
                    return err(token, "Expected type", 0);
                };
                let t = parse_type_ref(first, current_type, current_types)?;
                joined.push(t);
                current_type = VecDeque::new();
            }
            Token::Parens(ref inner) => {
                return parse_tuple_type(inner.clone(), current_types).and_then(|t| match t {
                    Type::Tuple(args) => Ok(Type::JoinedTuples(joined, args)),
                    Type::NamedTuple(args) => Ok(Type::JoinedNamedTuples(joined, args)),
                    _ => err(token, "Expected tuple type", 0),
                });
            }
            _ => {
                return err(token, "Expected type or plus sign", 0);
            }
        }
    }
    err(last, "Expected parens ending the joined tuple type", 0)
}

fn parse_tuple_type<'l>(
    mut tokens: Vec<PositionedToken<Token>>,
    current_types: &Vec<ImportExport<TypeDef>>,
) -> Result<Type, ErrorType> {
    if tokens.is_empty() {
        return Ok(Type::Tuple(Vec::new()));
    }
    let last_token = tokens.last().unwrap().clone();
    let mut args = Vec::new();
    let mut named_args = Vec::new();
    let mut current_name = String::new();
    let mut current_type = VecDeque::new();
    while let Some(token) = tokens.pop() {
        match token.token {
            Token::Name(ref name) => {
                if !current_name.is_empty() {
                    return err(token, "Expected type or closing paren", 0);
                }
                current_name = name.clone();
            }
            Token::Comma => {
                let Some(first) = current_type.pop_front() else {
                    return err(token, "Expected type", 0);
                };
                let t = parse_type_ref(first, current_type, current_types)?;
                if current_name.is_empty() {
                    args.push(t);
                } else {
                    named_args.push((current_name.clone(), t));
                }
                current_name = String::new();
                current_type = VecDeque::new();
            }
            _ => {
                current_type.push_back(token);
            }
        }
    }
    let last = current_type.back().unwrap_or(&last_token).clone();
    let Some(first) = current_type.pop_front() else {
        return err(last, "Expected type", 0);
    };
    let t = parse_type_ref(first, current_type, current_types)?;
    if current_name.is_empty() {
        args.push(t);
    } else {
        named_args.push((current_name.clone(), t));
    }
    if !args.is_empty() {
        if !named_args.is_empty() {
            return err(
                last_token,
                "Cannot mix named and unnamed tuple arguments",
                0,
            );
        } else {
            Ok(Type::Tuple(args))
        }
    } else if !named_args.is_empty() {
        Ok(Type::NamedTuple(named_args))
    } else {
        err(last_token, "Expected tuple arguments", 0)
    }
}

fn parse_union_type<'l>(
    mut tokens: VecDeque<PositionedToken<Token>>,
    current_types: &Vec<ImportExport<TypeDef>>,
) -> Result<Type, ErrorType> {
    tokens.push_back(PositionedToken {
        line_no: tokens.back().unwrap().line_no,
        word_no: tokens.back().unwrap().word_no + 1,
        token: Token::VerticalBar,
    });
    let mut parts = Vec::new();
    let mut current_part: VecDeque<PositionedToken<Token>> = VecDeque::new();
    while let Some(token) = tokens.pop_front() {
        match token.token {
            Token::VerticalBar => {
                let Some(label) = current_part.pop_front() else {
                    return err(token, "Expected labelled type", 0);
                };
                let label = match label.token {
                    Token::Type(name) => name,
                    _ => return err(label, "Expected label name", 0),
                };
                if current_part.is_empty() {
                    parts.push((label, None));
                    current_part = VecDeque::new();
                    continue;
                }
                let t = parse_type_ref(
                    current_part.pop_front().unwrap(),
                    current_part,
                    current_types,
                )?;
                parts.push((label, Some(t)));
                current_part = VecDeque::new();
            }
            _ => {
                current_part.push_back(token);
            }
        }
    }
    Ok(Type::Union(parts))
}

fn parse_type_ref<'l>(
    first: PositionedToken<Token>,
    mut tokens: VecDeque<PositionedToken<Token>>,
    current_types: &'l Vec<ImportExport<TypeType>>,
) -> Result<Type, ErrorType> {
    let Token::Type(ref name) = first.token else {
        return err(first, "Type references should start with a type name", 0);
    };
    let Some(ImportExport {
        token: base_type, ..
    }) = current_types.iter().find(|t| &t.token.name == name)
    else {
        return Ok(Type::Generic(name.clone()));
    };
    let mut type_args = Vec::new();
    while let Some(token) = tokens.pop_front() {
        match &token.token {
            Token::Type(_) => {
                type_args.push(parse_type_ref(
                    token.clone(),
                    VecDeque::new(),
                    current_types,
                )?);
            }
            Token::Underscore => {
                type_args.push(Type::CompilerFigureItOut);
            }
            Token::Parens(inner) => {
                let mut inner = VecDeque::from(inner.clone());
                let Some(first) = inner.pop_front() else {
                    return err(token, "Expected type between parentheses", 0);
                };
                type_args.push(parse_type_ref(first, VecDeque::from(inner), current_types)?);
            }
            _ => {
                return err(token, "Invalid token in type reference", 0);
            }
        }
    }
    let generics: HashMap<&String, Type> =
        HashMap::from_iter(base_type.args.iter().zip(type_args.into_iter()));
    Ok(replace_generics(&base_type.value, &generics))
}

fn replace_generics(base: &Type, generics: &HashMap<&String, Type>) -> Type {
    match base {
        Type::Builtin(t) => Type::Builtin(t.clone()),
        Type::Generic(ref name) => generics.get(name).unwrap_or(&base).clone(),
        Type::CompilerFigureItOut => Type::CompilerFigureItOut,
        Type::Union(parts) => Type::Union(
            parts
                .into_iter()
                .map(|(name, t)| {
                    (
                        name.clone(),
                        t.clone().map(|t| replace_generics(&t, generics)),
                    )
                })
                .collect(),
        ),
        Type::Tuple(args) => Type::Tuple(
            args.into_iter()
                .map(|t| replace_generics(t, generics))
                .collect(),
        ),
        Type::NamedTuple(args) => Type::NamedTuple(
            args.into_iter()
                .map(|(name, t)| (name.clone(), replace_generics(t, generics)))
                .collect(),
        ),
        Type::JoinedTuples(a, b) => Type::JoinedTuples(
            a.into_iter()
                .map(|t| replace_generics(t, generics))
                .collect(),
            b.into_iter()
                .map(|t| replace_generics(t, generics))
                .collect(),
        ),
        Type::JoinedNamedTuples(a, b) => Type::JoinedNamedTuples(
            a.into_iter()
                .map(|t| replace_generics(t, generics))
                .collect(),
            b.into_iter()
                .map(|(name, t)| (name.clone(), replace_generics(t, generics)))
                .collect(),
        ),
    }
}
