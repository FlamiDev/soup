use crate::parser::parser;
use parser_lib::{split_words, DynSafeParser, VecWindow};
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
    let parser = parser();
    let res = parser.parse(VecWindow::from(&words));
    println!("{:#?}", res);
}
