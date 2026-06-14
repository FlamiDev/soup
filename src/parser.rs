use libparsing::lexer::Lexeme;
use libparsing::parser;
use crate::token::Token;

pub fn parse(tokens: Vec<Lexeme<Token>>) -> Vec<i32> {
    parser::parse(
        tokens,
        parser::split(
            &[
                Token::KwDef,
                Token::KwLet,
                Token::KwTyp,
                Token::KwPub,
                Token::KwUse,
                Token::KwDoc,
            ],
            |part| 5,
            |all| all.to_vec(),
        ),
    )
}