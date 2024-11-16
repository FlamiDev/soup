use std::collections::{HashMap, VecDeque};

use crate::{
    compiler_tools::{
        parser::{ImportExport, TypeParseResult},
        tokenizer::PositionedToken,
    },
    parser::{err, parse_error, ParseError},
    tokenizer::Token,
};

pub fn error(
    token: PositionedToken<Token>,
    why: &str,
    priority: i64,
) -> TypeParseResult<TypeDef, ParseError> {
    TypeParseResult::Error(parse_error(token, why, priority))
}

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

pub fn parse_type_def(
    first_token: PositionedToken<Token>,
    mut tokens: VecDeque<PositionedToken<Token>>,
    current_types: &Vec<ImportExport<TypeDef>>,
) -> TypeParseResult<TypeDef, ParseError> {
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
    let (body, mut generics) = match parse_type(tokens, current_types) {
        Ok(t) => t,
        Err(e) => return TypeParseResult::Error(e),
    };
    args.sort();
    generics.sort();
    if args != generics {
        return error(
            equals_token,
            &format!(
                "Type arguments do not match: you defined {:?} but used {:?} while these should always be the same",
                args, generics
            ),
            0,
        );
    }
    TypeParseResult::Type(TypeDef {
        name: name.clone(),
        args,
        value: body,
    })
}

fn parse_type(
    mut tokens: VecDeque<PositionedToken<Token>>,
    current_types: &Vec<ImportExport<TypeDef>>,
) -> Result<(Type, Vec<String>), ParseError> {
    let first = tokens.front().unwrap().clone();
    match first.token {
        Token::Braces(inner) => parse_tuple_type(inner, current_types),
        Token::Type(_) => {
            if tokens.iter().any(|t| t.token == Token::VerticalBar) {
                parse_union_type(tokens, current_types)
            } else if tokens.iter().any(|t| t.token == Token::Plus) {
                parse_joined_tuple_type(tokens, current_types)
            } else {
                tokens.pop_front();
                parse_type_ref(&first, tokens, current_types)
            }
        }
        _ => err(first, "Invalid type", 0),
    }
}

fn parse_joined_tuple_type(
    mut tokens: VecDeque<PositionedToken<Token>>,
    current_types: &Vec<ImportExport<TypeDef>>,
) -> Result<(Type, Vec<String>), ParseError> {
    let mut current_type = VecDeque::new();
    let mut joined = Vec::new();
    let mut generics = Vec::new();
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
                let (t, g) = parse_type_ref(&first, current_type, current_types)?;
                joined.push(t);
                generics.extend(g);
                current_type = VecDeque::new();
            }
            Token::Braces(ref inner) => {
                return parse_tuple_type(inner.clone(), current_types).and_then(|(t, g)| match t {
                    Type::Tuple(args) => {
                        Ok((Type::JoinedTuples(joined, args), [generics, g].concat()))
                    }
                    Type::NamedTuple(args) => Ok((
                        Type::JoinedNamedTuples(joined, args),
                        [generics, g].concat(),
                    )),
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

fn parse_tuple_type(
    mut tokens: Vec<PositionedToken<Token>>,
    current_types: &Vec<ImportExport<TypeDef>>,
) -> Result<(Type, Vec<String>), ParseError> {
    if tokens.is_empty() {
        return Ok((Type::Tuple(Vec::new()), Vec::new()));
    }
    let last_token = tokens.last().unwrap().clone();
    let mut generics = Vec::new();
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
                let (t, g) = parse_type_ref(&first, current_type, current_types)?;
                if current_name.is_empty() {
                    args.push(t);
                } else {
                    named_args.push((current_name.clone(), t));
                }
                generics.extend(g);
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
    let (t, g) = parse_type_ref(&first, current_type, current_types)?;
    if current_name.is_empty() {
        args.push(t);
    } else {
        named_args.push((current_name.clone(), t));
    }
    generics.extend(g);
    if !args.is_empty() {
        if !named_args.is_empty() {
            err(
                last_token,
                "Cannot mix named and unnamed tuple arguments",
                0,
            )
        } else {
            Ok((Type::Tuple(args), generics))
        }
    } else if !named_args.is_empty() {
        Ok((Type::NamedTuple(named_args), generics))
    } else {
        err(last_token, "Expected tuple arguments", 0)
    }
}

fn parse_union_type(
    mut tokens: VecDeque<PositionedToken<Token>>,
    current_types: &Vec<ImportExport<TypeDef>>,
) -> Result<(Type, Vec<String>), ParseError> {
    tokens.push_back(PositionedToken {
        line_no: tokens.back().unwrap().line_no,
        word_no: tokens.back().unwrap().word_no + 1,
        token: Token::VerticalBar,
    });
    let mut generics = Vec::new();
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
                let (t, g) = parse_type_ref(
                    &current_part.pop_front().unwrap(),
                    current_part,
                    current_types,
                )?;
                generics.extend(g);
                parts.push((label, Some(t)));
                current_part = VecDeque::new();
            }
            _ => {
                current_part.push_back(token);
            }
        }
    }
    Ok((Type::Union(parts), generics))
}

pub fn parse_type_ref(
    first: &PositionedToken<Token>,
    mut tokens: VecDeque<PositionedToken<Token>>,
    current_types: &Vec<ImportExport<TypeDef>>,
) -> Result<(Type, Vec<String>), ParseError> {
    let Token::Type(ref name) = first.token else {
        return err(
            first.clone(),
            "Type references should start with a type name",
            0,
        );
    };
    let Some(ImportExport {
        token: base_type, ..
    }) = current_types.iter().find(|t| &t.token.name == name)
    else {
        return Ok((Type::Generic(name.clone()), vec![name.clone()]));
    };
    let mut type_args = Vec::new();
    while let Some(token) = tokens.pop_front() {
        match &token.token {
            Token::Type(_) => {
                type_args.push(parse_type_ref(&token, VecDeque::new(), current_types)?);
            }
            Token::Underscore => {
                type_args.push((Type::CompilerFigureItOut, Vec::new()));
            }
            Token::Parens(inner) => {
                let mut inner = VecDeque::from(inner.clone());
                let Some(first) = inner.pop_front() else {
                    return err(token, "Expected type between parentheses", 0);
                };
                type_args.push(parse_type_ref(&first, inner, current_types)?);
            }
            _ => {
                return err(token, "Invalid token in type reference", 0);
            }
        }
    }
    let generics_from_base: HashMap<&String, (Type, Vec<String>)> =
        HashMap::from_iter(base_type.args.iter().zip(type_args));
    Ok(replace_generics(&base_type.value, &generics_from_base))
}

fn replace_generics(
    base: &Type,
    generics: &HashMap<&String, (Type, Vec<String>)>,
) -> (Type, Vec<String>) {
    let mut g = Vec::new();
    let mut fix = |rg: (Type, Vec<String>)| {
        g.extend(rg.1);
        rg.0
    };
    let t = match base {
        Type::Builtin(t) => Type::Builtin(t.clone()),
        Type::Generic(ref name) => generics
            .get(name)
            .map(|t| fix(t.clone()))
            .unwrap_or(base.clone()),
        Type::CompilerFigureItOut => Type::CompilerFigureItOut,
        Type::Union(parts) => Type::Union(
            parts
                .iter()
                .map(|(name, t)| {
                    (
                        name.clone(),
                        t.clone().map(|t| fix(replace_generics(&t, generics))),
                    )
                })
                .collect(),
        ),
        Type::Tuple(args) => Type::Tuple(
            args.iter()
                .map(|t| fix(replace_generics(t, generics)))
                .collect(),
        ),
        Type::NamedTuple(args) => Type::NamedTuple(
            args.iter()
                .map(|(name, t)| (name.clone(), fix(replace_generics(t, generics))))
                .collect(),
        ),
        Type::JoinedTuples(a, b) => Type::JoinedTuples(
            a.iter()
                .map(|t| fix(replace_generics(t, generics)))
                .collect(),
            b.iter()
                .map(|t| fix(replace_generics(t, generics)))
                .collect(),
        ),
        Type::JoinedNamedTuples(a, b) => Type::JoinedNamedTuples(
            a.iter()
                .map(|t| fix(replace_generics(t, generics)))
                .collect(),
            b.iter()
                .map(|(name, t)| (name.clone(), fix(replace_generics(t, generics))))
                .collect(),
        ),
    };
    (t, g)
}
