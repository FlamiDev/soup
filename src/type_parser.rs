use std::collections::VecDeque;

use crate::compiler_tools::parser::ParserResult;
use crate::parser::error;
use crate::value_parser::ValueDef;
use crate::{
    compiler_tools::tokenizer::PositionedToken,
    parser::{err, parse_error, ParseError},
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
    Generic(String),
    CompilerFigureItOut,
    Reference(String, Vec<TypeRefPart>),
    Union(Vec<(String, Option<Type>)>),
    Tuple(Vec<Type>),
    NamedTuple(Vec<(String, Type)>),
    JoinedTuples(Vec<Type>, Vec<Type>),
    JoinedNamedTuples(Vec<Type>, Vec<(String, Type)>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeRefPart {
    Type(String),
    Underscore,
    Inner(Type),
}

pub fn parse_type_def(
    mut token_sets: Vec<(PositionedToken<Token>, VecDeque<PositionedToken<Token>>)>,
) -> ParserResult<TypeDef, ValueDef, ParseError> {
    let (first_token, mut tokens) = token_sets.pop().unwrap();
    let mut errors = Vec::new();
    let mut export = false;
    for set in token_sets {
        let first = set.0;
        match first.token {
            Token::ExportKeyword => {
                if export {
                    errors.push(parse_error(first, "Repeated export keyword", 0));
                }
                export = true;
            }
            _ => {
                errors.push(parse_error(
                    first,
                    "Expected only export keyword before type keyword",
                    0,
                ));
            }
        }
    }
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
    let body = match parse_type(tokens) {
        Ok(t) => t,
        Err(e) => return ParserResult::Error(vec![e]),
    };
    ParserResult::Type(TypeDef {
        name: name.clone(),
        args,
        value: body,
    })
}

fn parse_type(mut tokens: VecDeque<PositionedToken<Token>>) -> Result<Type, ParseError> {
    let first = tokens.front().unwrap().clone();
    match first.token {
        Token::Braces(inner) => parse_tuple_type(inner),
        Token::Type(_) => {
            if tokens.iter().any(|t| t.token == Token::VerticalBar) {
                parse_union_type(tokens)
            } else if tokens.iter().any(|t| t.token == Token::Plus) {
                parse_joined_tuple_type(tokens)
            } else {
                tokens.pop_front();
                parse_type_ref(&first, tokens)
            }
        }
        _ => err(first, "Invalid type", 0),
    }
}

fn parse_joined_tuple_type(
    mut tokens: VecDeque<PositionedToken<Token>>,
) -> Result<Type, ParseError> {
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
                let t = parse_type_ref(&first, current_type)?;
                joined.push(t);
                current_type = VecDeque::new();
            }
            Token::Braces(ref inner) => {
                return parse_tuple_type(inner.clone()).and_then(|t| match t {
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

fn parse_tuple_type(mut tokens: Vec<PositionedToken<Token>>) -> Result<Type, ParseError> {
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
                let t = parse_type_ref(&first, current_type)?;
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
    let t = parse_type_ref(&first, current_type)?;
    if current_name.is_empty() {
        args.push(t);
    } else {
        named_args.push((current_name.clone(), t));
    }
    if !args.is_empty() {
        if !named_args.is_empty() {
            err(
                last_token,
                "Cannot mix named and unnamed tuple arguments",
                0,
            )
        } else {
            Ok(Type::Tuple(args))
        }
    } else if !named_args.is_empty() {
        Ok(Type::NamedTuple(named_args))
    } else {
        err(last_token, "Expected tuple arguments", 0)
    }
}

fn parse_union_type(mut tokens: VecDeque<PositionedToken<Token>>) -> Result<Type, ParseError> {
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
                let t = parse_type_ref(&current_part.pop_front().unwrap(), current_part)?;
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

pub fn parse_type_ref(
    first: &PositionedToken<Token>,
    mut tokens: VecDeque<PositionedToken<Token>>,
) -> Result<Type, ParseError> {
    let Token::Type(mut type_name) = first.token.clone() else {
        return err(
            first.clone(),
            "Type references should start with a type name",
            0,
        );
    };
    let mut type_args = Vec::new();
    while let Some(token) = tokens.pop_front() {
        match token.token {
            Token::Dot => {
                let Some(next) = tokens.pop_front() else {
                    return err(token, "Expected type name after dot", 0);
                };
                let Token::Type(name) = next.token else {
                    return err(next, "Expected type name after dot", 0);
                };
                if let Some(last) = type_args.last_mut() {
                    match last {
                        TypeRefPart::Type(ref mut t) => {
                            t.push('.');
                            t.push_str(&name);
                        }
                        _ => {
                            return err(token, "Invalid dot token in type reference", 0);
                        }
                    }
                } else {
                    type_name.push('.');
                    type_name.push_str(&name);
                }
            }
            Token::Type(name) => {
                type_args.push(TypeRefPart::Type(name));
            }
            Token::Underscore => {
                type_args.push(TypeRefPart::Underscore);
            }
            Token::Parens(inner) => {
                let inner = parse_type(VecDeque::from(inner))?;
                type_args.push(TypeRefPart::Inner(inner));
            }
            _ => {
                return err(token, "Invalid token in type reference", 0);
            }
        }
    }
    Ok(Type::Reference(type_name.clone(), type_args))
}
