use crate::parser::basics::{bracketed, parse_match, tuple_shape};
use crate::parser::parse_value::{Value, ValuePattern, parse_value_pattern, parse_value};
use crate::parser::type_name;
use nom::branch::alt;
use nom::bytes::tag;
use nom::character::complete::multispace0;
use nom::combinator::opt;
use nom::multi::{many1, separated_list0, separated_list1};
use nom::sequence::separated_pair;
use nom::{IResult, Parser};

#[derive(Debug, PartialEq)]
pub enum Type {
    Tuple(Vec<Type>),
    Union(Vec<(String, Option<Type>)>),
    Function(Vec<Type>),
    MatchType(String, Vec<(TypePattern, Type)>),
    MatchValue(String, Vec<(ValuePattern, Type)>),
    Reference(String, Vec<Type>, Vec<Value>),
}

pub(crate) fn parse_type(input: &str) -> IResult<&str, Type> {
    alt((
        parse_tuple_type,
        parse_union_type,
        parse_function_type,
        parse_match_type_type,
        parse_match_value_type,
        parse_ref_type,
    ))
    .parse(input)
}

fn parse_lone_nested_type(input: &str) -> IResult<&str, Type> {
    alt((
        parse_tuple_type,
        parse_function_type,
        parse_ref_type,
    ))
        .parse(input)
}

fn parse_symbol_nested_type(input: &str) -> IResult<&str, Type> {
    alt((
        parse_tuple_type,
        bracketed(parse_function_type),
        parse_ref_type,
    ))
        .parse(input)
}

fn parse_text_nested_type(input: &str) -> IResult<&str, Type> {
    alt((
        parse_tuple_type,
        bracketed(parse_function_type),
        bracketed(parse_ref_type),
    ))
        .parse(input)
}

pub(crate) fn parse_tuple_type(input: &str) -> IResult<&str, Type> {
    tuple_shape(parse_lone_nested_type).map(Type::Tuple).parse(input)
}

pub(crate) fn parse_union_type(input: &str) -> IResult<&str, Type> {
    many1((
        tag("|"),
        multispace0,
        type_name,
        multispace0,
        opt(parse_lone_nested_type),
    ))
    .map(|options| {
        Type::Union(
            options
                .into_iter()
                .map(|(_, _, name, _, typ)| (name.to_string(), typ))
                .collect(),
        )
    })
    .parse(input)
}

pub(crate) fn parse_function_type(input: &str) -> IResult<&str, Type> {
    separated_pair(
        parse_symbol_nested_type,
        (multispace0, tag("->"), multispace0),
        separated_list1((multispace0, tag("->"), multispace0), parse_symbol_nested_type),
    )
    .map(|(arg, rets)| {
        let mut types = vec![arg];
        types.extend(rets);
        Type::Function(types)
    })
    .parse(input)
}

pub(crate) fn parse_match_type_type(input: &str) -> IResult<&str, Type> {
    parse_match(type_name, parse_type_pattern, parse_type)
        .map(|(t, p)| Type::MatchType(t, p))
        .parse(input)
}

pub(crate) fn parse_match_value_type(input: &str) -> IResult<&str, Type> {
    parse_match(type_name, parse_value_pattern, parse_type)
        .map(|(v, p)| Type::MatchValue(v, p))
        .parse(input)
}

pub(crate) fn parse_ref_type(input: &str) -> IResult<&str, Type> {
    (
        type_name,
        multispace0,
        separated_list0(multispace0, parse_text_nested_type),
        multispace0,
        separated_list0(multispace0, parse_value),
    )
        .map(|(name, _, type_args, _, value_args)| Type::Reference(name, type_args, value_args))
        .parse(input)
}

#[derive(Debug, PartialEq)]
pub enum TypePattern {
    Wildcard,
    Named(String),
    Tuple(Vec<TypePattern>),
    Union(String, String, Option<Box<TypePattern>>),
}

pub(crate) fn parse_type_pattern(input: &str) -> IResult<&str, TypePattern> {
    alt((
        tag("_").map(|_| TypePattern::Wildcard),
        type_name.map(|name| TypePattern::Named(name)),
        tuple_shape(parse_type_pattern).map(TypePattern::Tuple),
        (
            type_name,
            tag("."),
            type_name,
            multispace0,
            opt(parse_type_pattern),
        )
            .map(|(typ, _, variant, _, value)| {
                TypePattern::Union(typ, variant, value.map(Box::new))
            }),
    ))
    .parse(input)
}
