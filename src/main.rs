use crate::parser::Program;
use parser_lib::{split_words, Parser};
use std::collections::VecDeque;

mod parser;

fn main() {
    let mut args: VecDeque<String> = std::env::args().collect();
    args.pop_front();
    let Some(file) = args.pop_front() else {
        print!("No input file given");
        return;
    };
    let input = std::fs::read_to_string(file).expect("Failed to read file");
    let words = split_words(input.as_str(), "(){}[]");
    let program = Program::parse((&words).into());
    println!("{:#?}", program);
}
