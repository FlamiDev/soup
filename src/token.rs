use libparsing::parse_error::ParseErrorToken;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
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
    KwDef,
    KwLet,
    KwTyp,
    KwPub,
    KwUse,
    KwDoc,
    LexError,
}

impl ParseErrorToken for Token {
    fn as_text(&self) -> &'static str {
        match self {
            Token::Equals => "`=`",
            Token::Pipe => "`|`",
            Token::Semicolon => "`;`",
            Token::Colon => "`:`",
            Token::Comma => "`,`",
            Token::Period => "`.`",
            Token::Hashtag => "`#`",
            Token::SquareOpen => "`[`",
            Token::SquareClose => "`]`",
            Token::RoundOpen => "`(`",
            Token::RoundClose => "`)`",
            Token::TypeName => "<type_name>",
            Token::ValueName => "<value_name>",
            Token::String => "<string>",
            Token::Number => "<number>",
            Token::KwDef => "`def`",
            Token::KwLet => "`let`",
            Token::KwTyp => "`typ`",
            Token::KwPub => "`pub`",
            Token::KwUse => "`use`",
            Token::KwDoc => "`doc`",
            Token::LexError => "<ERROR>",
        }
    }
}
