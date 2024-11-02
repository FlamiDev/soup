use std::collections::VecDeque;

use crate::{compiler_tools::tokenizer::PositionedToken, tokenizer::Token};

#[derive(Debug, PartialEq, Clone)]
pub struct Root {
    pub types: Vec<TypeDef>,
    pub values: Vec<LetDef>,
    pub errors: Vec<ParseError>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeDef {
    pub name: String,
    pub args: Vec<String>,
    pub value: Type,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Union(Vec<TypeRef>),
    Tuple(Vec<TypeRef>),
    NamedTuple(Vec<(String, TypeRef)>),
    JoinedTuples(Vec<TypeRef>, Vec<TypeRef>),
    JoinedNamedTuples(Vec<TypeRef>, Vec<(String, TypeRef)>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeRef(pub String, pub Vec<TypeRef>);

#[derive(Debug, PartialEq, Clone)]
pub enum LetDef {
    Standard(String, Expression),
    Exported(String, Expression),
    Destructure(Vec<String>, Expression),
    Matched(String, String, Expression),
    MatchedDestructure(String, Vec<String>, Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Function {
        doc: String,
        tests: Vec<Block>,
        args: Vec<String>,
        body: Block,
    },
    Import(String),
    Construction(Vec<Expression>),
    NamedConstruction(Vec<(String, Expression)>),
    Int(i64),
    Float(f64),
    String(String),
    InterpolatedString(Vec<Expression>),
    Ref(String),
    Match(String, Vec<MatchCase>),
    FunctionCall(String, Vec<Expression>),
    UnparsedCallChain(Vec<Token>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct MatchCase {
    pub matcher_label: String,
    pub matcher_values: Vec<String>,
    pub body: Block,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub instructions: Vec<Instruction>,
    pub returns: Box<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    ValueDef(LetDef),
    Assert(Expression),
    Mock(String, Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError {
    pub line_no: i64,
    pub word_no: i64,
    pub token: Token,
    pub why: String,
    pub priority: i64,
    pub parser_line: u32,
}

enum ParserReturn<T> {
    Some(VecDeque<PositionedToken<Token>>, Result<T, ParseError>),
    None,
}

fn error_err<T>(
    token: PositionedToken<Token>,
    why: &str,
    line: u32,
    priority: i64,
) -> Result<T, ParseError> {
    Err(ParseError {
        line_no: token.line_no,
        word_no: token.word_no,
        token: token.token,
        why: why.to_string(),
        priority,
        parser_line: line,
    })
}

fn error_priority<T>(
    tokens: VecDeque<PositionedToken<Token>>,
    token: PositionedToken<Token>,
    why: &str,
    line: u32,
    priority: i64,
) -> ParserReturn<T> {
    ParserReturn::Some(tokens, error_err(token, why, line, priority))
}

fn error<T>(
    tokens: VecDeque<PositionedToken<Token>>,
    token: PositionedToken<Token>,
    why: &str,
    line: u32,
) -> ParserReturn<T> {
    error_priority(tokens, token, why, line, 0)
}

macro_rules! error {
    ($tokens:expr, $token:expr, $why:expr) => {
        return error($tokens, $token, $why, line!())
    };
    ($tokens:expr, $token:expr, $why:expr, $priority:expr) => {
        return error_priority($tokens, $token, $why, line!(), $priority)
    };
}

pub fn parse(tokens: &Vec<PositionedToken<Token>>) -> Root {
    let mut tokens = VecDeque::from(tokens.clone());
    let mut types = Vec::new();
    let mut values = Vec::new();
    let mut errors = Vec::new();
    while !tokens.is_empty() {
        if let ParserReturn::Some(t, res) = parse_type_def(&tokens) {
            tokens = t;
            match res {
                Ok(t) => types.push(t),
                Err(e) => errors.push(e),
            }
            continue;
        };
        if let ParserReturn::Some(t, res) = parse_let_def(&tokens) {
            tokens = t;
            match res {
                Ok(v) => values.push(v),
                Err(e) => errors.push(e),
            }
            continue;
        };
        let Some(token) = tokens.pop_front() else {
            break;
        };
        errors.push(ParseError {
            line_no: token.line_no,
            word_no: token.word_no,
            token: token.token,
            why: "Unexpected token at the top level".to_string(),
            priority: -100,
            parser_line: line!(),
        });
    }
    Root {
        types,
        values,
        errors,
    }
}

fn parse_type_def(tokens: &VecDeque<PositionedToken<Token>>) -> ParserReturn<TypeDef> {
    let mut tokens = tokens.clone();
    let Some(type_keyword) = tokens.pop_front() else {
        return ParserReturn::None;
    };
    if type_keyword.token != Token::TypeKeyword {
        return ParserReturn::None;
    }
    let Some(name_token) = tokens.pop_front() else {
        error!(tokens, type_keyword, "Expected type name");
    };
    let Token::Type(ref name) = name_token.token else {
        error!(tokens, name_token, "Expected type name");
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
        error!(tokens, name_token, "Expected equals sign");
    };
    let Token::EqualsSign = equals_token.token else {
        error!(tokens, equals_token, "Expected equals sign");
    };
    let ParserReturn::Some(tokens, body) = parse_type(&tokens) else {
        error!(tokens, equals_token, "Expected type body");
    };
    ParserReturn::Some(
        tokens,
        body.map(|t| TypeDef {
            name: name.clone(),
            args,
            value: t,
        }),
    )
}

fn parse_type(tokens: &VecDeque<PositionedToken<Token>>) -> ParserReturn<Type> {
    let Some(first) = tokens.front() else {
        return ParserReturn::None;
    };
    let second = tokens.get(1);
    match first.token {
        Token::Parens(ref inner) => {
            ParserReturn::Some(tokens.clone(), parse_tuple_type(inner.clone()))
        }
        Token::Type(_) => match second {
            Some(PositionedToken {
                line_no: _,
                word_no: _,
                token: Token::Plus,
            }) => parse_joined_tuple_type(tokens),
            Some(PositionedToken {
                line_no: _,
                word_no: _,
                token: Token::Type(_),
            }) => parse_union_type(tokens),
            Some(PositionedToken {
                line_no: _,
                word_no: _,
                token: Token::VerticalBar,
            }) => parse_union_type(tokens),
            _ => ParserReturn::None,
        },
        _ => ParserReturn::None,
    }
}

fn parse_joined_tuple_type(tokens: &VecDeque<PositionedToken<Token>>) -> ParserReturn<Type> {
    let mut tokens = tokens.clone();
    let mut current_type = VecDeque::new();
    let mut joined = Vec::new();
    while let Some(token) = tokens.pop_front() {
        match token.token {
            Token::Type(_) => {
                current_type.push_back(token);
            }
            Token::Plus => {
                let Some(t) = parse_type_ref(&current_type) else {
                    error!(tokens, token, "Expected type");
                };
                joined.push(t);
                current_type.clear();
            }
            Token::Parens(ref inner) => {
                return ParserReturn::Some(
                    tokens,
                    parse_tuple_type(inner.clone()).and_then(|t| match t {
                        Type::Tuple(args) => Ok(Type::JoinedTuples(joined, args)),
                        Type::NamedTuple(args) => Ok(Type::JoinedNamedTuples(joined, args)),
                        _ => error_err(token, "Expected tuple type", line!(), 0),
                    }),
                );
            }
            _ => {
                error!(tokens, token, "Expected type or plus sign");
            }
        }
    }
    ParserReturn::None
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
                    return error_err(token, "Expected type or closing paren", line!(), 0);
                }
                current_name = name.clone();
            }
            Token::Comma => {
                let Some(t) = parse_type_ref(&current_type) else {
                    return error_err(token, "Expected type", line!(), 0);
                };
                if current_name.is_empty() {
                    args.push(t);
                } else {
                    named_args.push((current_name.clone(), t));
                }
                current_name.clear();
            }
            _ => {
                current_type.push_back(token);
            }
        }
    }
    let Some(t) = parse_type_ref(&current_type) else {
        return error_err(
            current_type.back().map(|t| t.clone()).unwrap_or(last_token),
            "Expected type",
            line!(),
            0,
        );
    };
    if current_name.is_empty() {
        args.push(t);
    } else {
        named_args.push((current_name.clone(), t));
    }
    if !args.is_empty() {
        if !named_args.is_empty() {
            return error_err(
                last_token,
                "Cannot mix named and unnamed tuple arguments",
                line!(),
                0,
            );
        } else {
            Ok(Type::Tuple(args))
        }
    } else if !named_args.is_empty() {
        Ok(Type::NamedTuple(named_args))
    } else {
        error_err(last_token, "Expected tuple arguments", line!(), 0)
    }
}

fn parse_union_type(tokens: &VecDeque<PositionedToken<Token>>) -> ParserReturn<Type> {
    let mut tokens = tokens.clone();
    let mut parts = Vec::new();
    let mut current_part = VecDeque::new();
    while let Some(token) = tokens.pop_front() {
        match token.token {
            Token::VerticalBar => {
                if current_part.is_empty() {
                    error!(tokens, token, "Expected labelled type");
                }
                let Some(t) = parse_type_ref(&current_part) else {
                    error!(tokens, token, "Expected valid labelled type");
                };
                parts.push(t);
                current_part.clear();
            }
            Token::NewLine => {
                if current_part.is_empty() {
                    error!(tokens, token, "Expected labelled type");
                }
                let Some(t) = parse_type_ref(&current_part) else {
                    error!(tokens, token, "Expected valid labelled type");
                };
                parts.push(t);
                current_part.clear();
                let Some(PositionedToken {
                    line_no: _,
                    word_no: _,
                    token: Token::VerticalBar,
                }) = tokens.front()
                else {
                    break;
                };
            }
            _ => {
                current_part.push_back(token);
            }
        }
    }
    ParserReturn::Some(tokens, Ok(Type::Union(parts)))
}

fn parse_type_ref(tokens: &VecDeque<PositionedToken<Token>>) -> Option<TypeRef> {
    let mut tokens = tokens.clone();
    let Some(first) = tokens.pop_front() else {
        return None;
    };
    let Token::Type(name) = first.token else {
        return None;
    };
    let mut type_args = Vec::new();
    while let Some(token) = tokens.pop_front() {
        match token.token {
            Token::Type(arg) => {
                type_args.push(TypeRef(arg, Vec::new()));
            }
            Token::Underscore => {
                type_args.push(TypeRef("_".to_string(), Vec::new()));
            }
            Token::Parens(inner) => {
                let Some(t) = parse_type_ref(&VecDeque::from(inner)) else {
                    return None;
                };
                type_args.push(t);
            }
            _ => {
                return None;
            }
        }
    }
    Some(TypeRef(name, type_args.to_vec()))
}

fn parse_let_def(tokens: &VecDeque<PositionedToken<Token>>) -> ParserReturn<LetDef> {
    let tokens = tokens.clone();
    let Some(value_keyword) = tokens.front() else {
        return ParserReturn::None;
    };
    match value_keyword.token {
        Token::LetKeyword => parse_let(&tokens),
        Token::ExportKeyword => parse_let(&tokens),
        Token::DocKeyword => parse_doc_func(&tokens),
        Token::TestKeyword => parse_doc_func(&tokens),
        _ => ParserReturn::None,
    }
}

fn parse_let(tokens: &VecDeque<PositionedToken<Token>>) -> ParserReturn<LetDef> {
    ParserReturn::None
}

fn parse_doc_func(tokens: &VecDeque<PositionedToken<Token>>) -> ParserReturn<LetDef> {
    ParserReturn::None
}

fn parse_func(tokens: &VecDeque<PositionedToken<Token>>) -> ParserReturn<LetDef> {
    ParserReturn::None
}

fn parse_block(tokens: &VecDeque<PositionedToken<Token>>) -> ParserReturn<LetDef> {
    ParserReturn::None
}

fn parse_expr(tokens: &VecDeque<PositionedToken<Token>>) -> ParserReturn<LetDef> {
    ParserReturn::None
}
