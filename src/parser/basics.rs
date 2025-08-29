use nom::branch::alt;
use nom::bytes::tag;
use nom::character::complete::{alpha1, multispace0};
use nom::combinator::verify;
use nom::error::ParseError;
use nom::multi::{many1, separated_list0};
use nom::sequence::delimited;
use nom::IResult;
use nom::Parser;

pub(crate) fn alpha_under1(input: &str) -> IResult<&str, String> {
    many1(alt((alpha1, tag("_"))))
        .map(|s| s.join(""))
        .parse(input)
}

pub(crate) fn type_name(input: &str) -> IResult<&str, String> {
    verify(alpha1, |s: &str| {
        s.chars().next().map_or(false, |c| c.is_uppercase()) && s.chars().all(|c| c != '_')
    })
    .map(|s: &str| s.to_string())
    .parse(input)
}

pub(crate) fn value_name(input: &str) -> IResult<&str, String> {
    verify(alpha_under1, |s: &str| {
        s.chars().all(|c| c.is_lowercase() || c == '_')
    })
    .parse(input)
}

pub(crate) fn bracketed<'l, P, O, E>(
    inside: P,
) -> impl Parser<&'l str, Output = O, Error = E>
where
    P: Parser<&'l str, Output = O, Error = E>,
    E: ParseError<&'l str>,
{
    delimited((tag("("), multispace0), inside, (multispace0, tag(")")))
}

pub(crate) fn tuple_shape<'l, P, E>(
    inside: P,
) -> impl Parser<&'l str, Output = Vec<<P>::Output>, Error = E>
where
    P: Parser<&'l str, Error = E>,
    E: ParseError<&'l str>,
{
    delimited(
        (tag("{"), multispace0),
        separated_list0((tag(";"), multispace0), inside),
        (multispace0, tag("}")),
    )
}

pub(crate) fn parse_match<'l, O, P, TP, I, TI, E>(on: O, pattern: P, inside: I) -> impl Parser<&'l str, Output = (String, Vec<(TP, TI)>), Error = E>
where
    O: Parser<&'l str, Output = String, Error = E>,
    P: Parser<&'l str, Output = TP, Error = E>,
    I: Parser<&'l str, Output = TI, Error = E>,
    E: ParseError<&'l str>,
{
    (
        on,
        (multispace0, tag(":"), multispace0),
        many1((
            (tag("|"), multispace0),
            pattern,
            (multispace0, tag("->"), multispace0),
            inside,
            multispace0,
        )),
    ).map(move |(name, _, items)| {
        (name, items.into_iter().map(|(_, pat, _, val, _)| (pat, val)).collect())
    })
}
