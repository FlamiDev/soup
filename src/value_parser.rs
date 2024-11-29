use std::collections::VecDeque;

use crate::compiler_tools::parser::ParserResult;
use crate::compiler_tools::UnzipResult;
use crate::parser::{err_vec, error};
use crate::type_parser::{parse_type_ref, Type};
use crate::{
    compiler_tools::{parser::split_starting, tokenizer::PositionedToken},
    parser::{err, parse_error, ParseError},
    tokenizer::Token,
    type_parser::TypeDef,
};

#[derive(Debug, PartialEq, Clone)]
pub struct ValueDef {
    docs: Vec<String>,
    tests: Vec<(String, TestBlock)>,
    assignment: ValueAssignment,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ValueAssignment {
    Standard(String, Option<Type>, Expression),
    Matched(String, String, Option<Type>, Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expression(Type, ExpressionValue);
#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionValue {
    Function {
        args: Vec<(Type, String)>,
        body: Box<ExpressionValue>,
    },
    Block {
        instructions: Vec<Instruction>,
        returns: Box<ExpressionValue>,
        return_type: Type,
    },
    Array(Vec<Expression>),
    Construction(Vec<Expression>),
    NamedConstruction(Vec<(String, Expression)>),
    Comparison(Box<Expression>, ComparisonType, Box<Expression>),
    MathOperation(Box<Expression>, MathOperation, Box<Expression>),
    Int(i64),
    Float(f64),
    String(String),
    InterpolatedString(Vec<Expression>),
    Match(Box<Expression>, Vec<MatchCase>),
    FunctionCall(String, Vec<Expression>),
    Variable(String),
    Argument(Type, String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ComparisonType {
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MathOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MatchCase {
    pub matcher: ExpressionValue,
    pub body: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TestBlock {
    pub mocks: Vec<(String, Expression)>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    ValueDef(ValueDef),
    Assert(Expression),
}

pub fn parse_doc(
    first_token: PositionedToken<Token>,
    mut tokens: VecDeque<PositionedToken<Token>>,
) -> Result<String, ParseError> {
    let Token::DocKeyword = first_token.token else {
        return err(first_token, "Expected doc keyword", 0);
    };
    let Some(text) = tokens.pop_front() else {
        return err(first_token, "Expected doc text", 0);
    };
    let Token::String(text) = text.token else {
        return err(text, "Expected doc text string", 0);
    };
    if let Some(t) = tokens.pop_front() {
        return err(t, "Expected end of doc", 0);
    }
    Ok(text)
}

pub fn parse_test(
    first_token: PositionedToken<Token>,
    mut tokens: VecDeque<PositionedToken<Token>>,
) -> Result<(String, TestBlock), Vec<ParseError>> {
    let Token::TestKeyword = first_token.token else {
        return err_vec(first_token, "Expected test keyword", 0);
    };
    let Some(name_token) = tokens.pop_front() else {
        return err_vec(first_token, "Expected test name", 0);
    };
    let Token::String(ref name) = name_token.token else {
        return err_vec(name_token, "Expected test name string", 0);
    };
    let Some(body) = tokens.pop_front() else {
        return err_vec(name_token, "Expected test body", 0);
    };
    let Token::Parens(body) = body.token else {
        return err_vec(body, "Expected test body parens", 0);
    };
    if !tokens.is_empty() {
        return err_vec(
            tokens.pop_front().unwrap(),
            "Expected no tokens after test block",
            0,
        );
    }
    match parse_test_block(VecDeque::from(body)) {
        Ok(block) => Ok((name.clone(), block)),
        Err(e) => Err(e),
    }
}

pub fn parse_value_def(
    mut token_sets: Vec<(PositionedToken<Token>, VecDeque<PositionedToken<Token>>)>,
) -> ParserResult<TypeDef, ValueDef, ParseError> {
    let (first_token, mut tokens) = token_sets.pop().unwrap();
    let mut errors = Vec::new();
    let mut docs = Vec::new();
    let mut tests = Vec::new();
    for (first, set) in token_sets {
        match first.token {
            Token::DocKeyword => {
                match parse_doc(first, set) {
                    Ok(doc) => docs.push(doc),
                    Err(e) => errors.push(e),
                };
            }
            Token::TestKeyword => {
                match parse_test(first, set) {
                    Ok(test) => tests.push(test),
                    Err(e) => errors.extend(e),
                };
            }
            _ => errors.push(parse_error(first, "Unexpected block", 0)),
        }
    }
    let Token::LetKeyword = first_token.token else {
        return error(first_token, "Expected let keyword", 0);
    };
    let Some(name_token) = tokens.pop_front() else {
        return error(first_token, "Expected value name", 0);
    };
    let (matcher, name) = match name_token.token {
        Token::Name(ref name) => (None, name.clone()),
        Token::Type(ref matcher) => {
            let Some(name_token_2) = tokens.pop_front() else {
                return error(name_token, "Expected value name", 0);
            };
            match name_token_2.token {
                Token::Name(name) => (Some(matcher.clone()), name),
                _ => return error(name_token_2, "Expected value name", 0),
            }
        }
        _ => return error(name_token, "Expected value name", 0),
    };
    let mut type_tokens = take_until(&mut tokens, |t| matches!(t.token, Token::EqualsSign));
    let type_ = if let Some(first) = type_tokens.pop_front() {
        match parse_type_ref(&first, type_tokens) {
            Ok(t) => Some(t),
            Err(e) => return ParserResult::Error(vec![e]),
        }
    } else {
        None
    };
    let Some(equals_token) = tokens.pop_front() else {
        return error(name_token, "Expected equals sign", 0);
    };
    let Token::EqualsSign = equals_token.token else {
        return error(equals_token, "Expected equals sign", 0);
    };
    if tokens.is_empty() {
        return error(equals_token, "Expected value expression", 0);
    }

    match parse_expression(tokens) {
        Ok(v) => {
            let assignment = match matcher {
                Some(matcher) => ValueAssignment::Matched(matcher, name.clone(), type_, v),
                None => ValueAssignment::Standard(name.clone(), type_, v),
            };

            ParserResult::Value(ValueDef {
                docs,
                tests,
                assignment,
            })
        }
        Err(e) => ParserResult::Error(e),
    }
}

fn parse_test_block(
    mut body: VecDeque<PositionedToken<Token>>,
) -> Result<TestBlock, Vec<ParseError>> {
    err_vec(body.front().unwrap().clone(), "TODO", 0)
}

fn parse_instruction(
    mut body: VecDeque<PositionedToken<Token>>,
) -> Result<Instruction, Vec<ParseError>> {
    err_vec(body.front().unwrap().clone(), "TODO", 0)
}

fn parse_expression(
    mut body: VecDeque<PositionedToken<Token>>,
) -> Result<Expression, Vec<ParseError>> {
    let arguments = take_until(&mut body, |t| matches!(t.token, Token::ArrowRight));
    let (mut args, body) = if body.is_empty() {
        (VecDeque::new(), arguments)
    } else {
        (arguments, body)
    };
    let Some(first_body_token) = body.front().cloned() else {
        return err_vec(body.front().unwrap().clone(), "Expected expression body", 0);
    };
    let mut arguments = Vec::new();
    while !args.is_empty() {
        let Some(name_token) = args.pop_front() else {
            break;
        };
        let Token::Name(ref name) = name_token.token else {
            return err_vec(name_token, "Expected argument name", 0);
        };
        let mut arg = take_until(&mut args, |t| {
            matches!(t.token, Token::Name(_) | Token::ArrowRight)
        });
        let arg = arg
            .pop_front()
            .map(|f| parse_type_ref(&f, arg).map_err(|e| vec![e]))
            .transpose()?;
        arguments.push((arg, name.clone(), name_token));
    }
    let mut split = split_starting(
        body.into(),
        vec![Token::LetKeyword, Token::AssertKeyword, Token::RetKeyword],
    );
    let Some(res) = split.pop() else {
        return err_vec(first_body_token, "Expected expression return value", 0);
    };
    let (body, mut errors) = split
        .into_iter()
        .map(|i| parse_instruction(VecDeque::from(i)))
        .unzip_result();
    let guts = parse_expression_guts(res.into());
    let guts = match guts {
        Ok(g) => g,
        Err(e) => {
            errors.push(e);
            return Err(errors.into_iter().flatten().collect());
        }
    };
    if !errors.is_empty() {
        return Err(errors.into_iter().flatten().collect());
    }
    if body.is_empty() {
        Ok(guts)
    } else {
        let Expression(return_type, return_value) = guts;
        let block = ExpressionValue::Block {
            instructions: body,
            returns: Box::new(return_value),
            return_type: return_type.clone(),
        };
        if arguments.is_empty() {
            Ok(Expression(return_type, block))
        } else {
            Ok(Expression(
                return_type,
                ExpressionValue::Function {
                    args: arguments
                        .into_iter()
                        .map(|(t, n, token)| {
                            t.map(|t| (t, n.clone())).ok_or(parse_error(
                                token,
                                format!("Cannot infer type for argument {}", n).as_str(),
                                0,
                            ))
                        })
                        .all_ok()?,
                    body: Box::new(block),
                },
            ))
        }
    }
}

#[inline(always)]
fn parse_expression_guts(
    mut tokens: VecDeque<PositionedToken<Token>>,
) -> Result<Expression, Vec<ParseError>> {
    let mut left = VecDeque::new();
    while let Some(token) = tokens.pop_front() {
        let comp_type = match token.token {
            Token::DoubleEqualsSign => ComparisonType::Equal,
            Token::NotEqualsSign => ComparisonType::NotEqual,
            Token::LessThanSign => ComparisonType::LessThan,
            Token::GreaterThanSign => ComparisonType::GreaterThan,
            Token::LessThanEqualsSign => ComparisonType::LessThanOrEqual,
            Token::GreaterThanEqualsSign => ComparisonType::GreaterThanOrEqual,
            _ => {
                left.push_front(token);
                continue;
            }
        };
        let left_result = parse_expression_guts(left)?;
        let right_result = parse_expression_guts(tokens)?;
        if left_result.0 != right_result.0 {
            return err_vec(token, "Comparison types don't match", 0);
        }
        return Ok(Expression(
            left_result.0.clone(),
            ExpressionValue::Comparison(Box::new(left_result), comp_type, Box::new(right_result)),
        ));
    }
    let mut tokens = left;
    let mut left = VecDeque::new();
    while let Some(token) = tokens.pop_front() {
        let math_type = match token.token {
            Token::Plus => MathOperation::Add,
            Token::Minus => MathOperation::Subtract,
            _ => {
                left.push_front(token);
                continue;
            }
        };
        let left_result = parse_expression_guts(left)?;
        let right_result = parse_expression_guts(tokens)?;
        if left_result.0 != right_result.0 {
            return err_vec(token, "Math types don't match", 0);
        }
        return Ok(Expression(
            left_result.0.clone(),
            ExpressionValue::MathOperation(
                Box::new(left_result),
                math_type,
                Box::new(right_result),
            ),
        ));
    }
    let mut tokens = left;
    let mut left = VecDeque::new();
    while let Some(token) = tokens.pop_front() {
        let math_type = match token.token {
            Token::Asterisk => MathOperation::Multiply,
            Token::Slash => MathOperation::Divide,
            Token::Percent => MathOperation::Modulo,
            _ => {
                left.push_front(token);
                continue;
            }
        };
        let left_result = parse_expression_guts(left)?;
        let right_result = parse_expression_guts(tokens)?;
        if left_result.0 != right_result.0 {
            return err_vec(token, "Math types don't match", 0);
        }
        return Ok(Expression(
            left_result.0.clone(),
            ExpressionValue::MathOperation(
                Box::new(left_result),
                math_type,
                Box::new(right_result),
            ),
        ));
    }
    let mut tokens = left;
    err_vec(tokens.front().unwrap().clone(), "TODO_guts!!!", 0)
}

fn parse_array(
    mut tokens: VecDeque<PositionedToken<Token>>,
) -> Result<Expression, Vec<ParseError>> {
    let mut expressions = Vec::new();
    let mut current_tokens = VecDeque::new();
    while let Some(token) = tokens.pop_front() {
        match token.token {
            Token::Comma => {
                if current_tokens.is_empty() {
                    return err_vec(token, "Empty expression in array", 0);
                }
                expressions.push(parse_expression(current_tokens)?);
                current_tokens = VecDeque::new();
            }
            _ => {
                current_tokens.push_back(token);
            }
        }
    }
    expressions.push(parse_expression(current_tokens)?);
    if !expressions.iter().all(|e| e.0 == expressions[0].0) {
        return err_vec(
            tokens.front().unwrap().clone(),
            "Array elements must have the same type",
            0,
        );
    }
    Ok(Expression(
        expressions[0].0.clone(),
        ExpressionValue::Array(expressions),
    ))
}

fn take_until(
    tokens: &mut VecDeque<PositionedToken<Token>>,
    predicate: impl Fn(&PositionedToken<Token>) -> bool,
) -> VecDeque<PositionedToken<Token>> {
    let mut result = VecDeque::new();
    while let Some(token) = tokens.pop_front() {
        if predicate(&token) {
            tokens.push_front(token);
            break;
        }
        result.push_back(token);
    }
    result
}
