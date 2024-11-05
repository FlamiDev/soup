use std::{
    collections::{HashMap, VecDeque},
    sync::RwLock,
};

use compiler_tools::parse_file;

mod compiler_tools;
mod parser;
mod tokenizer;
mod type_parser;
mod value_parser;

fn main() {
    let mut args: VecDeque<String> = std::env::args().collect();
    args.pop_front();
    let Some(file) = args.pop_front() else {
        print!("No input file given");
        return;
    };
    let verbose_mode = args.contains(&"verbose".to_string()) || args.contains(&"v".to_string());
    let performance_mode =
        args.contains(&"performance".to_string()) || args.contains(&"p".to_string());

    let parse_cache = RwLock::new(HashMap::new());
    let Some(ast) = parse_file(file.clone(), tokenizer::parse, parser::parse, &parse_cache) else {
        println!("Could not read file '{}'", file);
        return;
    };

    if !performance_mode {
        println!("DEBUG -- AST:");
        println!(">>>>>>>>>> TYPES <<<<<<<<<<");
        println!("{:#?}", ast.types);
        println!(">>>>>>>>>> VALUES <<<<<<<<<<");
        println!("{:#?}", ast.values);
        println!(">>>>>>>>>> ERRORS <<<<<<<<<<");
        println!(
            "{:#?}",
            if verbose_mode {
                ast.errors
            } else {
                ast.errors
                // .into_iter()
                // .filter(|e| e.priority >= 0)
                // .collect::<Vec<_>>()
            }
        );
    }
}
