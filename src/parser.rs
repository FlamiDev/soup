use std::collections::VecDeque;

use crate::{compiler_tools::tokenizer::PositionedToken, tokenizer::Token};

#[derive(Debug, PartialEq, Clone)]
pub enum AST {
    Root(Vec<AST>),
    TypeDecl {
        name: Box<AST>,
        args: Vec<AST>,
        value: Box<AST>,
    },
    UnionType(Vec<AST>),
    UnionPart {
        label: Box<AST>,
        content: Box<AST>,
    },
    TupleType(Vec<AST>),
    TypedParam {
        name: Box<AST>,
        type_name: Box<AST>,
    },
    LetAssignment {
        name: Box<AST>,
        value: Box<AST>,
    },
    Function {
        args: Vec<AST>,
        body: Box<AST>,
    },
    FunctionBody(Vec<AST>),
    FunctionCall {
        name: Box<AST>,
        args: Vec<AST>,
    },
    MatchOp {
        value: Box<AST>,
        options: Vec<AST>,
    },
    MatchCase {
        matcher: Box<AST>,
        body: Box<AST>,
    },
    LabelMatcher {
        label: Box<AST>,
        values: Vec<AST>,
    },
    Documented {
        doc: Box<AST>,
        tests: Vec<AST>,
        body: Box<AST>,
    },
    DocString(String),
    TestBlock {
        test_name: String,
        body: Box<AST>,
    },
    TestAssert(Box<AST>),
    TestMock {
        replace: Box<AST>,
        with: Box<AST>,
    },
    Import(Box<AST>),
    Export(Box<AST>),
    Construction(Vec<AST>),
    Destruction {
        type_name: Box<AST>,
        values: Vec<AST>,
    },
    LabeledValue(Box<AST>, Box<AST>),
    TypeName(String),
    ValueName(String),
    Int(i64),
    Float(f64),
    String(String),
    InterpolatedString(Vec<AST>),
    Trait(Box<AST>, Vec<AST>, Box<AST>),
    SyntaxError(i64, i64, Token, String, i64),
}

pub fn is_error(ast: &AST) -> bool {
    match ast {
        AST::SyntaxError(..) => true,
        _ => false,
    }
}

pub fn is_error_level(ast: &AST, level: i64) -> bool {
    match ast {
        AST::SyntaxError(.., priority) if *priority >= level => true,
        _ => false,
    }
}

fn error_priority(
    tokens: VecDeque<PositionedToken<Token>>,
    token: PositionedToken<Token>,
    why: &str,
    priority: i64,
) -> Option<(VecDeque<PositionedToken<Token>>, AST)> {
    Some((
        tokens,
        AST::SyntaxError(
            token.line_no,
            token.word_no,
            token.token,
            why.to_string(),
            priority,
        ),
    ))
}

fn error(
    tokens: VecDeque<PositionedToken<Token>>,
    token: PositionedToken<Token>,
    why: &str,
) -> Option<(VecDeque<PositionedToken<Token>>, AST)> {
    error_priority(tokens, token, why, 0)
}

pub fn parse(tokens: &Vec<PositionedToken<Token>>) -> AST {
    let mut tokens = VecDeque::from(tokens.clone());
    let mut body = Vec::new();
    while !tokens.is_empty() {
        if let Some((t, b)) = one_of!(tokens, parse_type_decl, parse_let_decl) {
            tokens = t;
            body.push(b);
        } else {
            if let Some(invalid) = tokens.pop_front() {
                body.push(AST::SyntaxError(
                    invalid.line_no,
                    invalid.word_no,
                    invalid.token,
                    "Unexpected token at the top level".to_string(),
                    -100,
                ));
            } else {
                break;
            }
        }
    }
    AST::Root(body)
}

fn parse_type_decl(
    tokens: &VecDeque<PositionedToken<Token>>,
) -> Option<(VecDeque<PositionedToken<Token>>, AST)> {
    let mut tokens = tokens.clone();
    let type_keyword = tokens.pop_front()?;
    if type_keyword.token != Token::TypeKeyword {
        return None;
    }
    let Some(name_token) = tokens.pop_front() else {
        return error(tokens, type_keyword, "Expected type name");
    };
    let Token::Type(ref name) = name_token.token else {
        return error(tokens, name_token, "Expected type name");
    };
    let mut args = Vec::new();
    while let Some(token) = tokens.pop_front() {
        let Token::Type(arg) = token.token else {
            tokens.push_front(token);
            break;
        };
        args.push(AST::TypeName(arg));
    }
    let Some(equals_token) = tokens.pop_front() else {
        return error(tokens, name_token, "Expected equals sign");
    };
    let Token::EqualsSign = equals_token.token else {
        return error(tokens, equals_token, "Expected equals sign");
    };
    let Some((tokens, body)) = parse_type(&tokens) else {
        return error(tokens, equals_token, "Expected type body");
    };
    Some((
        tokens,
        AST::TypeDecl {
            name: Box::new(AST::TypeName(name.clone())),
            args: args,
            value: Box::new(body),
        },
    ))
}

fn parse_type(
    tokens: &VecDeque<PositionedToken<Token>>,
) -> Option<(VecDeque<PositionedToken<Token>>, AST)> {
    let first = tokens.front()?;
    match first.token {
        Token::ParenOpen => parse_tuple_type(tokens),
        Token::Type(_) => parse_union_type(tokens),
        _ => None,
    }
}

fn parse_tuple_type(
    tokens: &VecDeque<PositionedToken<Token>>,
) -> Option<(VecDeque<PositionedToken<Token>>, AST)> {
    let mut tokens = tokens.clone();
    let open_paren = tokens.pop_front()?;
    let Token::ParenOpen = open_paren.token else {
        return error(tokens, open_paren, "Expected opening paren");
    };
    let mut args = Vec::new();
    while let Some(token) = tokens.pop_front() {
        match token.token {
            Token::Type(arg) => {
                args.push(AST::TypeName(arg));
            }
            Token::Name(ref arg) => {
                let Token::Colon = tokens.pop_front()?.token else {
                    return error(tokens, token, "Expected colon");
                };
                let Token::Type(type_) = tokens.pop_front()?.token else {
                    return error(tokens, token, "Expected type name");
                };
                args.push(AST::TypedParam {
                    name: Box::new(AST::ValueName(arg.clone())),
                    type_name: Box::new(AST::TypeName(type_)),
                });
            }
            Token::Comma => {
                // TODO Why are commas here if they don't have any meaning whatsoever?!
            }
            Token::ParenClose => {
                break;
            }
            _ => {
                return error(tokens, token, "Expected type, name or closing paren");
            }
        }
    }
    Some((tokens, AST::TupleType(args)))
}

fn parse_union_type(
    tokens: &VecDeque<PositionedToken<Token>>,
) -> Option<(VecDeque<PositionedToken<Token>>, AST)> {
    None
}

fn parse_let_decl(
    tokens: &VecDeque<PositionedToken<Token>>,
) -> Option<(VecDeque<PositionedToken<Token>>, AST)> {
    None
}
