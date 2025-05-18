use crate::errors::show_errors;
use crate::parser::AST;
use parser_lib::{setup_logging, split_words, BracketPair, Parser};
use std::collections::VecDeque;

mod errors;
mod parser;

fn main() {
    unsafe { backtrace_on_stack_overflow::enable() };
    setup_logging(false);
    
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
    println!("{:#?}", program.0);
    show_errors(input.as_str(), program.2);
}
