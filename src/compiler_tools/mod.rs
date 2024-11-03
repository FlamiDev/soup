use tokenizer::PositionedToken;

#[macro_use]
pub mod parser;
pub mod tokenizer;

pub fn parse_file<'l, Token: 'l, AST: 'l>(
    file: String,
    tokenize: fn(&str) -> Vec<PositionedToken<Token>>,
    parse: fn(
        Vec<PositionedToken<Token>>,
        Box<dyn Fn(String) -> Option<AST> + 'l + Sync + Send>,
    ) -> AST,
) -> Option<AST> {
    let input = std::fs::read_to_string(file).ok()?;
    let tokens = tokenize(input.as_str());
    let pf = move |file| parse_file(file, tokenize, parse);
    let ast = parse(tokens, Box::new(pf));
    Some(ast)
}
