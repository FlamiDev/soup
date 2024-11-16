use std::collections::VecDeque;

use crate::{
    compiler_tools::{
        parser::{self, Import, ImportParseResult, ValueParser, AST},
        tokenizer::PositionedToken,
        ParseFile,
    },
    tokenizer::Token,
    type_parser::{parse_type_def, Type, TypeDef},
    value_parser::{parse_doc, parse_test, parse_value_def, ValueDef},
};

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError {
    pub line_no: i64,
    pub word_no: i64,
    pub token: Token,
    pub why: String,
    pub priority: i64,
}

pub fn parse_error(token: PositionedToken<Token>, why: &str, priority: i64) -> ParseError {
    ParseError {
        line_no: token.line_no,
        word_no: token.word_no,
        token: token.token,
        why: why.to_string(),
        priority,
    }
}

pub fn err<T>(token: PositionedToken<Token>, why: &str, priority: i64) -> Result<T, ParseError> {
    Err(parse_error(token, why, priority))
}

pub fn parse(
    tokens: Vec<PositionedToken<Token>>,
    parse_file: ParseFile<AST<TypeDef, ValueDef, ParseError>>,
) -> AST<TypeDef, ValueDef, ParseError> {
    parser::parse(
        tokens,
        Token::NewLine,
        Token::ImportKeyword,
        Token::ExportKeyword,
        Token::TypeKeyword,
        vec![
            TypeDef {
                name: "Int".to_string(),
                args: Vec::new(),
                value: Type::Builtin("Int".to_string()),
            },
            TypeDef {
                name: "Float".to_string(),
                args: Vec::new(),
                value: Type::Builtin("Float".to_string()),
            },
            TypeDef {
                name: "String".to_string(),
                args: Vec::new(),
                value: Type::Builtin("String".to_string()),
            },
        ],
        parse_import,
        parse_type_def,
        vec![
            ValueParser(vec![Token::DocKeyword], parse_doc),
            ValueParser(vec![Token::TestKeyword], parse_test),
            ValueParser(vec![Token::LetKeyword], parse_value_def),
        ],
        |token, message| ParseError {
            line_no: token.line_no,
            word_no: token.word_no,
            token: token.token,
            why: message,
            priority: 0,
        },
        parse_file,
    )
}

fn parse_import(
    mut tokens: VecDeque<PositionedToken<Token>>,
) -> ImportParseResult<Import, ParseError> {
    let Some(token) = tokens.pop_front() else {
        return ImportParseResult::Failure;
    };
    let Token::Type(ref name) = token.token else {
        return ImportParseResult::Error(parse_error(
            token,
            "Expected type name after import keyword",
            0,
        ));
    };
    let Some(token) = tokens.pop_front() else {
        return ImportParseResult::Error(parse_error(
            token,
            "Expected file path after type name",
            0,
        ));
    };
    let Token::String(path) = token.token else {
        return ImportParseResult::Error(parse_error(
            token,
            "Expected file path after type name",
            0,
        ));
    };
    if let Some(end) = tokens.pop_front() {
        let Token::NewLine = end.token else {
            return ImportParseResult::Error(parse_error(
                end,
                "Expected newline after import statement",
                0,
            ));
        };
    }
    ImportParseResult::Success(Import {
        name: name.clone(),
        from: path,
    })
}
