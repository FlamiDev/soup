use std::fs;

mod parser;

fn main() {
    let input = fs::read_to_string("main.soup").expect("Failed to read input file");
    let result = parser::parse(&input, |file| println!("Using: {}", file));
    println!("{:#?}", result);
}
