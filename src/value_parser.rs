use std::collections::VecDeque;

use crate::{
    compiler_tools::{
        parser::{ImportExport, ValueParseResult},
        tokenizer::PositionedToken,
    },
    parser::{parse_error, ParseError},
    tokenizer::Token,
    type_parser::{Type, TypeDef},
};

#[derive(Debug, PartialEq, Clone)]
pub enum ValueDef {
    Standard(String, Type, Expression),
    Destructure(Vec<String>, Type, Expression),
    Matched(String, String, Type, Expression),
    MatchedDestructure(Vec<String>, String, Type, Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expression(Type, ExpressionValue);
#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionValue {
    Function {
        doc: Vec<String>,
        tests: Vec<Block>,
        args: Vec<(Type, String)>,
        body: Block,
    },
    Construction(Vec<Expression>),
    NamedConstruction(Vec<(String, Expression)>),
    Int(i64),
    Float(f64),
    String(String),
    InterpolatedString(Vec<Expression>),
    Match(Box<Expression>, Vec<MatchCase>),
    FunctionCall(String, Vec<Expression>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct MatchCase {
    pub matcher: String,
    pub body: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Matcher {
    Label(String),
    List(Vec<String>, String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub instructions: Vec<Instruction>,
    pub returns: Box<Expression>,
    pub return_type: Type,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    ValueDef(ValueDef),
    Assert(Expression),
    Mock(String, Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Take {
    Doc(String),
    Test(String, VecDeque<PositionedToken<Token>>),
}

pub fn error(
    token: PositionedToken<Token>,
    why: &str,
    priority: i64,
) -> ValueParseResult<ValueDef, Take, ParseError> {
    ValueParseResult::Error(parse_error(token, why, priority))
}

pub fn parse_func_doc(
    first_token: PositionedToken<Token>,
    mut tokens: VecDeque<PositionedToken<Token>>,
    _take_to_next: &mut Vec<Take>,
    _current_types: &Vec<ImportExport<TypeDef>>,
    _current_values: &Vec<ImportExport<ValueDef>>,
) -> ValueParseResult<ValueDef, Take, ParseError> {
    let Token::DocKeyword = first_token.token else {
        return error(first_token, "Expected doc keyword", 0);
    };
    let Some(text) = tokens.pop_front() else {
        return error(first_token, "Expected doc text", 0);
    };
    let Token::String(text) = text.token else {
        return error(text, "Expected doc text string", 0);
    };
    if let Some(t) = tokens.pop_front() {
        return error(t, "Expected end of doc", 0);
    }
    return ValueParseResult::TakeToNext(Take::Doc(text));
}

pub fn parse_func_test(
    first_token: PositionedToken<Token>,
    mut tokens: VecDeque<PositionedToken<Token>>,
    _take_to_next: &mut Vec<Take>,
    _current_types: &Vec<ImportExport<TypeDef>>,
    _current_values: &Vec<ImportExport<ValueDef>>,
) -> ValueParseResult<ValueDef, Take, ParseError> {
    let Token::TestKeyword = first_token.token else {
        return error(first_token, "Expected test keyword", 0);
    };
    let Some(name_token) = tokens.pop_front() else {
        return error(first_token, "Expected test name", 0);
    };
    let Token::String(ref name) = name_token.token else {
        return error(name_token, "Expected test name string", 0);
    };
    let Some(body) = tokens.pop_front() else {
        return error(name_token, "Expected test body", 0);
    };
    let Token::Braces(body) = body.token else {
        return error(body, "Expected test body braces", 0);
    };
    return ValueParseResult::TakeToNext(Take::Test(name.clone(), body.into()));
}

pub fn parse_value_def(
    first_token: PositionedToken<Token>,
    mut tokens: VecDeque<PositionedToken<Token>>,
    take_to_next: &mut Vec<Take>,
    current_types: &Vec<ImportExport<TypeDef>>,
    current_values: &Vec<ImportExport<ValueDef>>,
) -> ValueParseResult<ValueDef, Take, ParseError> {
    return error(
        first_token,
        format!("Temp error, take_to_next is {:#?}", take_to_next).as_str(),
        100,
    );
}
