use libparsing::lexer;
use std::fs;

macro_rules! map {
    ($($k:expr => $v:expr),* $(,)?) => {{
        core::convert::From::from([$(($k, $v),)*])
    }};
}

#[derive(Copy, Clone, Debug)]
enum Token {
    Equals,
    Pipe,
    Semicolon,
    Colon,
    Comma,
    Period,
    Hashtag,
    SquareOpen,
    SquareClose,
    RoundOpen,
    RoundClose,
    TypeName,
    ValueName,
    String,
    Number,
    LexError,
}

fn main() {
    let input = fs::read_to_string("main.soup").expect("Failed to read input file");
    let tokens = lexer::lex(
        input.as_ref(),
        map! {
            "=" => Token::Equals,
            "|" => Token::Pipe,
            ";" => Token::Semicolon,
            ":" => Token::Colon,
            "," => Token::Comma,
            "." => Token::Period,
            "#" => Token::Hashtag,
            "[" => Token::SquareOpen,
            "]" => Token::SquareClose,
            "(" => Token::RoundOpen,
            ")" => Token::RoundClose,
        },
        Token::TypeName,
        Token::ValueName,
        Token::String,
        Token::Number,
        Token::LexError,
        '/',
        Some(('<', '>')),
    );
    println!("{:#?}", tokens);
}
