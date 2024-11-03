use std::{collections::HashMap, sync::RwLock};

use tokenizer::PositionedToken;

#[macro_use]
pub mod parser;
pub mod tokenizer;

pub type ParseFile<'l, AST> = Box<dyn Fn(String) -> Option<AST> + 'l + Sync + Send>;

pub fn parse_file<'l, Token: 'l, AST: 'l + Clone + Sync + Send>(
    file: String,
    tokenize: fn(&str) -> Vec<PositionedToken<Token>>,
    parse: fn(Vec<PositionedToken<Token>>, ParseFile<'l, AST>) -> AST,
    cache: &'l RwLock<HashMap<String, AST>>,
) -> Option<AST> {
    if let Some(ast) = cache.read().ok()?.get(&file) {
        return Some(ast.clone());
    }
    let input = std::fs::read_to_string(file.clone()).ok()?;
    let tokens = tokenize(input.as_str());
    let pf = move |file: String| parse_file(file.clone(), tokenize, parse, &cache);
    let ast = parse(tokens, Box::new(pf));
    cache.write().ok()?.insert(file, ast.clone());
    Some(ast)
}
