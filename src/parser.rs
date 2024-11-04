use std::collections::VecDeque;

use crate::{
    compiler_tools::{
        parser::{self, Import, ImportParseResult, AST},
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
        Token::ImportKeyword,
        Token::ExportKeyword,
        Token::TypeKeyword,
        parse_import,
        parse_type,
        vec![],
        |token, message, line| message,
        parse_file,
    )
}

fn parse_import(mut tokens: VecDeque<PositionedToken<Token>>) -> ImportParseResult<Import, String> {
    let Some(token) = tokens.pop_front() else {
        return ImportParseResult::Failure;
    };
    let Token::Type(ref name) = token.token else {
        return ImportParseResult::Error("Expected type name after import keyword".to_string());
    };
    let Some(token) = tokens.pop_front() else {
        return ImportParseResult::Error("Expected file path after type name".to_string());
    };
    let Token::String(path) = token.token else {
        return ImportParseResult::Error("Expected file path after type name".to_string());
    };
    if let Some(end) = tokens.pop_front() {
        let Token::NewLine = end.token else {
            return ImportParseResult::Error("Expected newline after import statement".to_string());
        };
    }
    ImportParseResult::Success(Import {
        name: name.clone(),
        from: path,
    })
}
