use crate::lexer::Lexeme;

pub trait ParseErrorToken {
    fn as_text(&self) -> &'static str;
}

pub type ParseResult<'l, Token, T> = Result<T, Vec<ParseError<'l, Token>>>;

#[derive(Debug)]
pub struct ParseError<'l, Token: ParseErrorToken> {
    expected: Vec<Token>,
    got: Option<Lexeme<'l, Token>>,
}

impl<'l, Token: ParseErrorToken> ParseError<'l, Token> {
    pub fn none<T>(expected: Vec<Token>) -> Result<T, Vec<Self>> {
        Err(vec![ParseError {
            expected,
            got: None,
        }])
    }
    pub fn fancy_print(&self, file_name: String) -> String {
        let message = match self.got {
            None => format!("=> {}\n\tunexpected end of input", file_name),
            Some(ref got) => format!(
                "=> {}:{}:{}\n\tunexpected `{}`",
                file_name, got.line.0, got.column.0, got.source
            ),
        };
        format!(
            "{},\n\texpected {}",
            message,
            self.expected
                .iter()
                .map(|it| it.as_text())
                .collect::<Vec<&str>>()
                .join(", ")
        )
    }
}

impl<'l, Token: ParseErrorToken> Lexeme<'l, Token> {
    pub fn error<T>(self, expected: Vec<Token>) -> Result<T, Vec<ParseError<'l, Token>>> {
        Err(vec![ParseError {
            expected,
            got: Some(self),
        }])
    }
}
