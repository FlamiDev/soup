#[macro_use]
mod compiler_tools;
mod parser;
mod tokenizer;

use crate::parser::{is_error, is_error_level, AST};

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let Some(file) = args.pop() else {
        print!("No input file given");
        return;
    };
    println!("Reading file {}", file);
    let Ok(input) = std::fs::read_to_string(file) else {
        println!("Could not read file");
        return;
    };

    let tokens = tokenizer::parse(&input);

    if compiler_tools::tokenizer::debug_invalid(&tokens, |t| match t {
        tokenizer::Token::Invalid(..) => true,
        _ => false,
    }) {
        println!("Invalid tokens found, exiting");
        return;
    }

    println!("DEBUG -- Tokens:");
    for token in &tokens {
        println!("{token:?}");
    }

    let ast = parser::parse(&tokens);
    println!("DEBUG -- AST:");
    match ast {
        AST::Root(body) => {
            println!(
                "{:?}",
                body.iter().filter(|e| !is_error(e)).collect::<Vec<&AST>>()
            );
            println!("ERRORS:");
            println!(
                "{:?}",
                body.iter()
                    .filter(|e| is_error_level(e, 0))
                    .collect::<Vec<&AST>>()
            );
        }
        _ => {}
    }
}
