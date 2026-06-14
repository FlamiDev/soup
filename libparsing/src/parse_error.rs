use crate::lexer::Lexeme;

pub type ParseResult<'l, Token, T> = Result<T, Vec<ParseError<'l, Token>>>;

#[derive(Debug)]
pub struct ParseError<'l, Token> {
    expected: Vec<Token>,
    got: Option<Lexeme<'l, Token>>,
}

impl<'l, Token> ParseError<'l, Token> {
    pub fn none<T>(expected: Vec<Token>) -> Result<T, Vec<Self>> {
        Err(vec![ParseError {
            expected,
            got: None,
        }])
    }
}

impl<'l, Token> Lexeme<'l, Token> {
    pub fn error<T>(self, expected: Vec<Token>) -> Result<T, Vec<ParseError<'l, Token>>> {
        Err(vec![ParseError {
            expected,
            got: Some(self),
        }])
    }
}
