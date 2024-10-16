use std::env;

mod tokenizer;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    match args.pop() {
        None => println!("No input file given"),
        Some(file) => {
            println!("Reading file {}", file);
            let Ok(input) = std::fs::read_to_string(file) else {
                println!("Could not read file");
                return;
            };
            let tokens = tokenizer::parse(&input);
            if !&tokens
                .iter()
                .map(|token| match token {
                    tokenizer::Token::Invalid(line_no, word_no, word) => {
                        println!("Invalid token at line {line_no}, word {word_no}: <{word}>");
                        false
                    }
                    tokenizer::Token::InvalidString(line_no, word_no, word) => {
                        println!("Invalid string at line {line_no}, word {word_no}: <{word}>");
                        false
                    }
                    _ => true,
                })
                .collect::<Vec<bool>>()
                .iter()
                .all(|x| *x)
            {
                println!("Invalid tokens found, exiting");
                return;
            }

            println!("DEBUG -- Tokens:");
            for token in tokens {
                println!("{token:?}");
            }
        }
    }
}
