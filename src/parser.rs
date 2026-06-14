use crate::ast::Ast;
use crate::token::Token;
use libparsing::lexer::Lexeme;
use libparsing::parse_error::{ParseError, ParseResult};
use libparsing::parser;
use libparsing::walker::Walker;

pub fn parse<'l>(tokens: &'l[Lexeme<Token>]) -> ParseResult<'l, Token, Vec<Ast>> {
    let top_level_keywords = vec![
        Token::KwUse,
        Token::KwDoc,
        Token::KwTyp,
        Token::KwDef,
        Token::KwLet,
    ];
    parser::parse(
        tokens,
        parser::split(
            &top_level_keywords,
            |walker| {
                let Some(current) = walker.current() else {
                    return ParseError::none(top_level_keywords.clone());
                };
                match current.token {
                    Token::KwUse => parse_use(walker),
                    Token::KwDoc => parse_doc(walker),
                    Token::KwTyp => parse_typ(walker),
                    Token::KwDef => parse_def(walker),
                    Token::KwLet => parse_let(walker),
                    _ => current.clone().error(top_level_keywords.clone()),
                }
            },
            |all| all,
        ),
    )
}

fn parse_use<'l>(walker: Walker<'l, Lexeme<'l, Token>>) -> ParseResult<'l, Token, Ast>{
    Ok(Ast::Use {
        from: "".to_string(),
        name: None,
        items: vec![],
    })
}
fn parse_doc<'l>(walker: Walker<'l, Lexeme<'l, Token>>) -> ParseResult<'l, Token, Ast>{
    Ok(Ast::Doc("".to_string()))
}
fn parse_typ<'l>(walker: Walker<'l, Lexeme<'l, Token>>) -> ParseResult<'l, Token, Ast>{
    Ok(Ast::Typ)
}
fn parse_def<'l>(walker: Walker<'l, Lexeme<'l, Token>>) -> ParseResult<'l, Token, Ast>{
    Ok(Ast::Def)
}
fn parse_let<'l>(walker: Walker<'l, Lexeme<'l, Token>>) -> ParseResult<'l, Token, Ast>{
    Ok(Ast::Let)
}
