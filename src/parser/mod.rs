use nom::branch::alt;
use nom::bytes::{tag, take_while};
use nom::character::complete::{alpha1, multispace0, multispace1};
use nom::combinator::{opt, verify};
use nom::multi::{many0, many1, separated_list0};
use nom::sequence::{delimited, separated_pair};
use nom::{AsChar, IResult, Parser};
#[macro_use]
mod macros;
mod parse_type;
mod parse_value;
mod basics;

use parse_type::{Type, parse_type};
use parse_value::{Value, parse_value};
use basics::{alpha_under1, type_name, value_name};

enum ParseFn {
    Use,
    Doc,
    Typ,
    Def,
    Let,
    Has,
}

pub fn parse<F: Fn(&str)>(input: &str, found_use: F) -> Vec<IResult<&str, AST>> {
    let mut keyword_indices = vec![];
    let input_length = input.len();
    for (i, c) in input.chars().enumerate() {
        if i == 0 || c.is_newline() {
            let check = if i == 0 { 0 } else { i + 1 };
            if check >= input_length {
                continue;
            }
            let slice = &input[check..];
            if slice.starts_with("use ") {
                keyword_indices.push((check, ParseFn::Use));
            } else if slice.starts_with("doc") {
                keyword_indices.push((check, ParseFn::Doc));
            } else if slice.starts_with("typ ") {
                keyword_indices.push((check, ParseFn::Typ));
            } else if slice.starts_with("def ") {
                keyword_indices.push((check, ParseFn::Def));
            } else if slice.starts_with("let ") {
                keyword_indices.push((check, ParseFn::Let));
            } else if slice.starts_with("has ") {
                keyword_indices.push((check, ParseFn::Has));
            }
        }
    }
    let mut results = vec![];
    for (index, parse_fn) in keyword_indices.iter() {
        let slice = keyword_indices
            .get(index + 1)
            .map_or(&input[*index..], |(next_index, _)| {
                &input[*index..*next_index]
            });
        let res = match parse_fn {
            ParseFn::Use => {
                let r = parse_use(slice);
                if let Ok((_, AST::Use { from, .. })) = &r {
                    found_use(from);
                }
                r
            }
            ParseFn::Doc => parse_doc(slice),
            ParseFn::Typ => parse_typ(slice),
            ParseFn::Def => parse_def(slice),
            ParseFn::Let => parse_let(slice),
            ParseFn::Has => parse_has(slice),
        };
        results.push(res);
    }
    results
}

#[derive(Debug, PartialEq)]
pub enum AST {
    Use {
        items: Vec<String>,
        from: String,
    },
    Doc {
        content: String,
    },
    Typ {
        public: bool,
        name: String,
        type_args: Vec<String>,
        dependent_args: Vec<(String, String)>,
        body: Type,
    },
    Def {
        public: bool,
        name: String,
        type_args: Vec<String>,
        body: Type,
    },
    Let {
        name: String,
        body: Value,
    },
    Has {
        name: String,
        type_args: Vec<String>,
        body: Vec<Requirement>,
    },
}

fn parse_use(input: &str) -> IResult<&str, AST> {
    ws_sep!(
        tag("use"),
        delimited(tag("{"), separated_list0(ws, alpha_under1), tag("}"),),
        delimited(
            tag("\""),
            take_while(|c: char| !c.is_whitespace()),
            tag("\"")
        )
    )
    .map(|(_, (items, from))| AST::Use {
        items: items.into_iter().map(String::from).collect(),
        from: String::from(from),
    })
    .parse(input)
}

fn parse_doc(input: &str) -> IResult<&str, AST> {
    ws_sep!(
        tag("doc"),
        delimited(tag("\""), take_while(|c: char| c != '"'), tag("\""))
    )
    .map(|(_, content)| AST::Doc {
        content: String::from(content),
    })
    .parse(input)
}

fn parse_typ(input: &str) -> IResult<&str, AST> {
    ws_sep!(
        tag("typ"),
        is_pub,
        type_name,
        separated_list0(ws, type_name),
        separated_list0(ws, separated_pair(value_name, ws, type_name)),
        tag("="),
        parse_type
    )
    .map(
        |(_, (public, (name, (type_args, (dependent_args, (_, body))))))| AST::Typ {
            public,
            name,
            type_args,
            dependent_args,
            body,
        },
    )
    .parse(input)
}

fn parse_def(input: &str) -> IResult<&str, AST> {
    ws_sep!(
        tag("def"),
        is_pub,
        value_name,
        separated_list0(ws, type_name),
        tag("="),
        parse_type
    )
    .map(|(_, (public, (name, (type_args, (_, body)))))| AST::Def {
        public,
        name,
        type_args,
        body,
    })
    .parse(input)
}

fn parse_let(input: &str) -> IResult<&str, AST> {
    ws_sep!(tag("let"), value_name, tag("="), parse_value)
        .map(|(_, (name, (_, body)))| AST::Let { name, body })
        .parse(input)
}

fn parse_has(input: &str) -> IResult<&str, AST> {
    ws_sep!(
        tag("has"),
        type_name,
        separated_list0(ws, type_name),
        tag("="),
        many1(parse_requirement)
    )
    .map(|(_, (name, (type_args, (_, body))))| AST::Has {
        name,
        type_args,
        body,
    })
    .parse(input)
}

#[derive(Debug, PartialEq)]
pub struct Requirement {
    name: String,
    body: Type,
}

fn parse_requirement(input: &str) -> IResult<&str, Requirement> {
    ws_sep!(value_name, tag("=>"), parse_type)
        .map(|(name, (_, body))| Requirement { name, body })
        .parse(input)
}

fn ws(input: &str) -> IResult<&str, &str> {
    multispace0.parse(input)
}

fn is_pub(input: &str) -> IResult<&str, bool> {
    opt(tag("pub")).map(|opt| opt.is_some()).parse(input)
}
