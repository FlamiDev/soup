use std::collections::VecDeque;

use crate::{
    compiler_tools::{
        parser::{ImportExport, ValueParseResult},
        tokenizer::PositionedToken,
    },
    parser::{err, parse_error, ParseError},
    tokenizer::Token,
    type_parser::TypeDef,
};

#[derive(Debug, PartialEq, Clone)]
pub struct TypeRef(String, Vec<TypeRef>);

#[derive(Debug, PartialEq, Clone)]
pub struct ValueRef(Box<ValueDef>);

#[derive(Debug, PartialEq, Clone)]
pub struct ValueDef {
    docs: Vec<String>,
    tests: Vec<(String, TestBlock)>,
    assignment: ValueAssignment,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ValueAssignment {
    Standard(String, Expression),
    Destructure(Vec<String>, Expression),
    // TODO: Change String into label type ref
    Matched(String, String, Expression),
    MatchedDestructure(Vec<String>, String, Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expression(TypeRef, ExpressionValue);
#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionValue {
    Function {
        args: Vec<(TypeRef, String)>,
        body: Block,
    },
    Array(Vec<Expression>),
    Construction(Vec<Expression>),
    NamedConstruction(Vec<(String, Expression)>),
    Int(i64),
    Float(f64),
    String(String),
    InterpolatedString(Vec<Expression>),
    Match(Box<Expression>, Vec<MatchCase>),
    FunctionCall(ValueRef, Vec<Expression>),
    Variable(ValueRef),
}

#[derive(Debug, PartialEq, Clone)]
pub struct MatchCase {
    pub matcher: ExpressionValue,
    pub body: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub instructions: Vec<Instruction>,
    pub returns: Box<Expression>,
    pub return_type: TypeRef,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TestBlock {
    pub mocks: Vec<(String, Expression)>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    ValueDef(ValueAssignment),
    Assert(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Take {
    Doc(String),
    Test(String, VecDeque<PositionedToken<Token>>),
}

pub fn error<T>(
    token: PositionedToken<Token>,
    why: &str,
    priority: i64,
) -> ValueParseResult<T, Take, ParseError> {
    ValueParseResult::Error(vec![parse_error(token, why, priority)])
}

pub fn parse_doc<'l>(
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
    ValueParseResult::TakeToNext(Take::Doc(text))
}

pub fn parse_test<'l>(
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
    let Token::Parens(body) = body.token else {
        return error(body, "Expected test body parens", 0);
    };
    ValueParseResult::TakeToNext(Take::Test(name.clone(), body.into()))
}

pub fn parse_value_def<'l>(
    first_token: PositionedToken<Token>,
    mut tokens: VecDeque<PositionedToken<Token>>,
    take_to_next: &mut Vec<Take>,
    current_types: &Vec<ImportExport<TypeDef>>,
    current_values: &Vec<ImportExport<ValueDef>>,
) -> ValueParseResult<ValueDef, Take, ParseError> {
    let Token::LetKeyword = first_token.token else {
        return error(first_token, "Expected let keyword", 0);
    };
    let Some(name_token) = tokens.pop_front() else {
        return error(first_token, "Expected value name", 0);
    };
    let Token::Name(ref name) = name_token.token else {
        return error(name_token, "Expected value name string", 0);
    };
    let mut type_tokens = take_until(&mut tokens, |t| matches!(t.token, Token::EqualsSign));
    let type_ = if let Some(first) = type_tokens.pop_front() {
        match parse_type_ref(&first, type_tokens, current_types) {
            Ok(t) => Some(t),
            Err(e) => return ValueParseResult::Error(vec![e]),
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

    let mut docs = Vec::new();
    let mut tests = Vec::new();
    for take in take_to_next.drain(..) {
        match take {
            Take::Doc(doc) => docs.push(doc.clone()),
            Take::Test(name, body) => tests.push((name, body)),
        }
    }

    let first = tokens[0].clone();
    match parse_expression(&type_, tokens, current_types, current_values) {
        Ok(v) => {
            if type_.as_ref().map_or(true, |t| t == &v.0) {
                let assignment = ValueAssignment::Standard(name.clone(), v);
                let (tests, errors) = tests
                    .into_iter()
                    .map(|(n, t)| {
                        (
                            n,
                            parse_test_block(t, current_types, current_values, &assignment),
                        )
                    })
                    .map(|(s, r)| r.map(|r| (s, r)))
                    .unzip_result();

                if !errors.is_empty() {
                    return ValueParseResult::Error(errors.into_iter().flatten().collect());
                }

                ValueParseResult::Value(ValueDef {
                    docs,
                    tests,
                    assignment,
                })
            } else {
                error(
                    first,
                    format!("Types don't match. Defined as {:?}, found {:?}", type_, v.0).as_str(),
                    0,
                )
            }
        }
        Err(e) => ValueParseResult::Error(e),
    }
}

fn parse_expression<'l>(
    expected_type: &Option<TypeRef>,
    mut body: VecDeque<PositionedToken<Token>>,
    current_types: &Vec<ImportExport<TypeDef>>,
    current_values: &Vec<ImportExport<ValueDef>>,
) -> Result<Expression, Vec<ParseError>> {
    return Err(vec![parse_error(body.front().unwrap().clone(), "", 0)]);
}

fn parse_block<'l>(
    mut body: VecDeque<PositionedToken<Token>>,
    current_types: &Vec<ImportExport<TypeDef>>,
    current_values: &Vec<ImportExport<ValueDef>>,
) -> Result<Block, Vec<ParseError>> {
    return Err(vec![parse_error(body.front().unwrap().clone(), "", 0)]);
}

fn parse_test_block<'l>(
    mut body: VecDeque<PositionedToken<Token>>,
    current_types: &Vec<ImportExport<TypeDef>>,
    current_values: &Vec<ImportExport<ValueDef>>,
    current_assignment: &ValueAssignment,
) -> Result<TestBlock, Vec<ParseError>> {
    return Err(vec![parse_error(body.front().unwrap().clone(), "", 0)]);
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

pub fn parse_type_ref<'l>(
    first: &PositionedToken<Token>,
    mut tokens: VecDeque<PositionedToken<Token>>,
    current_types: &'l Vec<ImportExport<TypeDef>>,
) -> Result<TypeRef, ParseError> {
    let Token::Type(ref name) = first.token else {
        return err(first.clone(), "Types should start with a type name", 0);
    };
    let Some(ImportExport {
        token: base_type, ..
    }) = current_types.iter().find(|t| &t.token.name == name)
    else {
        return err(
            first.clone(),
            format!("Type {} not found", name).as_str(),
            0,
        );
    };
    let mut type_args = Vec::new();
    while let Some(token) = tokens.pop_front() {
        match &token.token {
            Token::Type(_) => {
                type_args.push(parse_type_ref(&token, VecDeque::new(), current_types)?);
            }
            Token::Parens(inner) => {
                let mut inner = VecDeque::from(inner.clone());
                let Some(first) = inner.pop_front() else {
                    return err(token, "Expected type between parentheses", 0);
                };
                type_args.push(parse_type_ref(
                    &first,
                    VecDeque::from(inner),
                    current_types,
                )?);
            }
            _ => {
                return err(token, "Invalid token in type", 0);
            }
        }
    }
    if base_type.args.len() != type_args.len() {
        return err(
            first.clone(),
            format!(
                "Type {} expects {} arguments, found {}",
                name,
                base_type.args.len(),
                type_args.len()
            )
            .as_str(),
            0,
        );
    }
    Ok(TypeRef(base_type.name.clone(), type_args))
}

trait UnzipResult<V, E> {
    fn unzip_result(&mut self) -> (Vec<V>, Vec<E>);
}

impl<V, E, I> UnzipResult<V, E> for I
where
    I: Iterator<Item = Result<V, E>>,
{
    fn unzip_result(&mut self) -> (Vec<V>, Vec<E>) {
        let size = self.size_hint().0;
        let mut values = Vec::with_capacity(size);
        let mut errors = Vec::with_capacity(size);

        for r in self {
            match r {
                Ok(v) => values.push(v),
                Err(e) => errors.push(e),
            }
        }
        (values, errors)
    }
}
