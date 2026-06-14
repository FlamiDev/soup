use crate::lexer::Lexeme;
use crate::walker::Walker;

pub trait Parser<'l, Token, Ast>: (Fn(Walker<Lexeme<'l, Token>>) -> Ast) + Clone {}
impl<'l, Token, Ast, F: (Fn(Walker<Lexeme<'l, Token>>) -> Ast) + Clone> Parser<'l, Token, Ast>
    for F
{
}

pub fn parse<'l, Token: PartialEq, Ast>(
    tokens: Vec<Lexeme<'l, Token>>,
    parser: impl Parser<'l, Token, Ast>,
) -> Ast {
    let tokens = Walker::new(&tokens);
    parser(tokens)
}

pub fn split<'l, Token: PartialEq, A, B>(
    on: &[Token],
    then: impl Parser<'l, Token, A>,
    combine: impl (Fn(&[A]) -> B) + Clone,
) -> impl Parser<'l, Token, B> {
    move |mut walker| {
        let mut split = vec![];
        loop {
            let Some(current) = walker.current() else {
                break;
            };
            if on.contains(&current.token) {
                let (start, new_walker) = walker.split();
                split.push(start);
                walker = new_walker;
            }
            walker.next();
        }
        split.push(walker);
        combine(&split.into_iter().map(then.clone()).collect::<Vec<A>>())
    }
}
