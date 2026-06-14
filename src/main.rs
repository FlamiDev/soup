pub mod ast;
pub mod lexer;
pub mod parser;
pub mod token;

use crate::lexer::lex;
use std::fs;
use crate::parser::parse;

fn main() {
    let input = fs::read_to_string("main.soup").expect("Failed to read input file");
    let tokens = lex(&input);
    let ast = parse(tokens);
    println!("{:#?}", ast);
}
