use parser::ParseError;

#[macro_use]
mod compiler_tools;
mod parser;
mod tokenizer;

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0);
    let Some((file, args)) = args.split_first() else {
        print!("No input file given");
        return;
    };
    let verbose_mode = args.contains(&"verbose".to_string()) || args.contains(&"v".to_string());

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

    if verbose_mode {
        print!("Tokens:");
        for token in &tokens {
            if token.word_no == 1 {
                println!();
                print!("{} | ", token.line_no);
            }
            print!("{:?} ", token.token);
        }
        println!();
    }

    let ast = parser::parse(&tokens);
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
                .into_iter()
                .filter(|e| e.priority >= 0)
                .collect::<Vec<ParseError>>()
        }
    );
}
