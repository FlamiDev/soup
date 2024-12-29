use parser_lib::{parser, split_words, Keyword, Parser, VecWindow};

mod compiler_tools;
mod parser;
mod tokenizer;
mod type_parser;
mod value_parser;
//
// fn main() {
//     let parser: chumsky::primitive::Just<&str, &str, Simple<&str>> = just("hey");
//     let res = parser.parse([]);
//     println!("{:#?}", res);
// }

fn main() {
    let words = split_words("foo baz", "(){}[]");
    let parser = parser().keyword(Keyword("foo")).keyword(Keyword("bar"));
    let res = parser.parse(VecWindow::from(&words));
    println!("{:#?}", res);
}

// fn main() {
//     let mut args: VecDeque<String> = std::env::args().collect();
//     args.pop_front();
//     let Some(file) = args.pop_front() else {
//         print!("No input file given");
//         return;
//     };
//     let verbose_mode = args.contains(&"verbose".to_string()) || args.contains(&"v".to_string());
//     let performance_mode =
//         args.contains(&"performance".to_string()) || args.contains(&"p".to_string());
//
//     let ast = parse_file(
//         file.clone(),
//         tokenizer::parse,
//         parser::parse,
//         |line_no, word_no, message| ParseError {
//             token: Token::ImportKeyword,
//             line_no,
//             word_no,
//             priority: 0,
//             why: message,
//         },
//     );
//
//     if !performance_mode {
//         println!("DEBUG -- AST:");
//         println!(">>>>>>>>>> TYPES <<<<<<<<<<");
//         println!("{:#?}", ast.0.get(&file.clone()).unwrap().types);
//         println!(">>>>>>>>>> VALUES <<<<<<<<<<");
//         println!("{:#?}", ast.0.get(&file.clone()).unwrap().values);
//         println!(">>>>>>>>>> ERRORS <<<<<<<<<<");
//         println!(
//             "{:#?}",
//             if verbose_mode {
//                 ast.1
//             } else {
//                 ast.1
//                     .into_iter()
//                     .filter(|e| e.priority >= 0)
//                     .collect::<Vec<_>>()
//             }
//         );
//     }
// }
