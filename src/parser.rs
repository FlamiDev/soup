use std::collections::VecDeque;

use crate::{
    compiler_tools::{
        parser::{self, Import, ParseResult, AST},
        tokenizer::PositionedToken,
        ParseFile,
    },
    tokenizer::Token,
};

pub fn parse(
    tokens: Vec<PositionedToken<Token>>,
    parse_file: ParseFile<AST<String, String, String>>,
) -> AST<String, String, String> {
    parser::parse(
        tokens,
        vec![
            Token::ImportKeyword,
            Token::ExportKeyword,
            Token::TypeKeyword,
            Token::LetKeyword,
            Token::DocKeyword,
            Token::TestKeyword,
        ],
        Token::ImportKeyword,
        parse_import,
        parse_file,
        |token, message, line| message,
    )
}

fn parse_import(mut tokens: VecDeque<PositionedToken<Token>>) -> ParseResult<Token, Import> {
    let Some(first) = tokens.pop_front() else {
        return ParseResult::Failure;
    };
    let Some(token) = tokens.pop_front() else {
        return ParseResult::Error(
            first,
            "Expected type name after import keyword".to_string(),
            0,
        );
    };
    let Token::Type(ref name) = token.token else {
        return ParseResult::Error(
            token,
            "Expected type name after import keyword".to_string(),
            0,
        );
    };
    let Some(token) = tokens.pop_front() else {
        return ParseResult::Error(token, "Expected file path after type name".to_string(), 0);
    };
    let Token::String(path) = token.token else {
        return ParseResult::Error(token, "Expected file path after type name".to_string(), 0);
    };
    if !tokens.is_empty() {
        return ParseResult::Error(
            tokens.back().unwrap().clone(),
            "Expected end of line after import statement".to_string(),
            0,
        );
    }
    ParseResult::Success(Import {
        name: name.clone(),
        from: path,
    })
}
