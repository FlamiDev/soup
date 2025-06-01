use crate::errors::show_errors;
use crate::parser::AST;
use parser_lib::{setup_logging, split_words, BracketPair, Parser};
use std::collections::VecDeque;

mod errors;
mod parser;

fn main() {
    setup_logging();

    let mut args: VecDeque<String> = std::env::args().collect();
    args.pop_front();
    let Some(file) = args.pop_front() else {
        print!("No input file given");
        return;
    };
    let input = std::fs::read_to_string(file).expect("Failed to read file");
    let words = split_words(
        input.as_str(),
        vec![
            BracketPair {
                open: '{',
                close: '}',
            },
            BracketPair {
                open: '(',
                close: ')',
            },
            BracketPair {
                open: '[',
                close: ']',
            },
        ],
    );
    let program = AST::parse((&words).into());
    std::fs::write("output.txt", format!("{:#?}", program.0)).expect("Failed to write output file");
    std::fs::write("errors.txt", format!("{:#?}", program.2)).expect("Failed to write errors file");
    show_errors(input.as_str(), program.2, true);
}
