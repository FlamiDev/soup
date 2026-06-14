use crate::token::Token;
use libparsing::lexer;
use libparsing::lexer::Lexeme;

macro_rules! map {
    ($($k:expr => $v:expr),* $(,)?) => {{
        core::convert::From::from([$(($k, $v),)*])
    }};
}

pub fn lex(input: &str) -> Vec<Lexeme<'_, Token>> {
    lexer::lex(
        input,
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
        map! {
            "def" => Token::KwDef,
            "let" => Token::KwLet,
            "typ" => Token::KwTyp,
            "pub" => Token::KwPub,
            "use" => Token::KwUse,
            "doc" => Token::KwDoc,
        },
        Token::TypeName,
        Token::ValueName,
        Token::String,
        Token::Number,
        Token::LexError,
        '/',
        Some(('<', '>')),
    )
}
