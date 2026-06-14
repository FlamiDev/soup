use crate::lexer::Lexeme;
use crate::parse_error::{ParseErrorToken, ParseResult};
use crate::walker::Walker;

pub trait Parser<'l, Token: 'l + ParseErrorToken, T>:
    (Fn(Walker<'l, Lexeme<'l, Token>>) -> ParseResult<'l, Token, T>) + Clone
{
}
impl<
    'l,
    Token: 'l + ParseErrorToken,
    T,
    F: (Fn(Walker<'l, Lexeme<'l, Token>>) -> ParseResult<'l, Token, T>) + Clone,
> Parser<'l, Token, T> for F
{
}

pub fn parse<'l, Token: 'l + PartialEq + ParseErrorToken, T>(
    tokens: &'l [Lexeme<'l, Token>],
    parser: impl Parser<'l, Token, T>,
) -> ParseResult<'l, Token, T> {
    let tokens = Walker::new(&tokens);
    parser(tokens)
}

pub fn split<'l, Token: 'l + PartialEq + ParseErrorToken, A, B>(
    on: &[Token],
    then: impl Parser<'l, Token, A>,
    combine: impl (Fn(Vec<A>) -> B) + Clone,
) -> impl Parser<'l, Token, B> {
    move |mut walker| {
        let mut split = vec![];
        loop {
            walker.next();
            let Some(current) = walker.current() else {
                break;
            };
            if on.contains(&current.token) {
                split.push(walker.drop_tail());
            }
        }
        walker.reset();
        split.push(walker);
        let parsed = split
            .into_iter()
            .map(then.clone())
            .collect::<Vec<ParseResult<'l, Token, A>>>();
        if parsed.iter().any(|it| it.is_err()) {
            return Err(parsed
                .into_iter()
                .filter_map(|it| it.err())
                .flatten()
                .collect());
        }
        Ok(combine(
            parsed
                .into_iter()
                .filter_map(|it| it.ok())
                .collect::<Vec<A>>(),
        ))
    }
}
