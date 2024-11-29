use crate::compiler_tools::parser::ParseResult;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use std::collections::HashMap;
use std::fmt::Debug;
use tokenizer::PositionedToken;

#[macro_use]
pub mod parser;
pub mod tokenizer;

#[derive(Debug, PartialEq, Clone)]
pub struct ParseFileResult<AST: Debug, Error: Debug>(pub HashMap<String, AST>, pub Vec<Error>);

pub fn parse_file<
    'l,
    Token: 'l,
    AST: 'l + Debug + Clone + Sync + Send,
    Error: 'l + Debug + Clone + Sync + Send,
>(
    file: String,
    tokenize: fn(String) -> Vec<PositionedToken<Token>>,
    parse: fn(Vec<PositionedToken<Token>>) -> ParseResult<AST, Error>,
    create_error: fn(i64, i64, String) -> Error,
) -> ParseFileResult<AST, Error> {
    let mut result = HashMap::new();
    let mut errors = Vec::new();
    let mut imports = vec![PositionedToken {
        token: file,
        line_no: 0,
        word_no: 0,
    }];

    while !imports.is_empty() {
        let parsed = imports
            .into_par_iter()
            .filter(|file| !result.contains_key(&file.token))
            .map(|file| {
                let input = read_file(file.token.clone()).ok_or(file.clone())?;
                let tokens = tokenize(input);
                Ok((file.token, parse(tokens)))
            })
            .collect::<Vec<Result<(String, ParseResult<AST, Error>), PositionedToken<String>>>>()
            .into_iter()
            .unzip_result();
        imports = Vec::new();
        for (file, res) in parsed.0 {
            imports.extend(res.0);
            result.insert(file, res.1);
            errors.extend(res.2);
        }
        errors.extend(parsed.1.into_iter().map(|t| {
            create_error(
                t.line_no,
                t.word_no,
                format!("Could not read file '{}'", t.token),
            )
        }));
    }
    ParseFileResult(result, errors)
}

fn read_file(file: String) -> Option<String> {
    std::fs::read_to_string(file).ok()
}

pub trait UnzipResult<V, E> {
    fn unzip_result(&mut self) -> (Vec<V>, Vec<E>);
    fn all_ok(&mut self) -> Result<Vec<V>, Vec<E>>;
}

impl<V, E, I> UnzipResult<V, E> for I
where
    I: Iterator<Item = Result<V, E>>,
{
    fn unzip_result(&mut self) -> (Vec<V>, Vec<E>) {
        let size = self.size_hint().0;
        let mut values = Vec::with_capacity(size);
        let mut errors = Vec::with_capacity(size);

        for r in self {
            match r {
                Ok(v) => values.push(v),
                Err(e) => errors.push(e),
            }
        }
        (values, errors)
    }
    fn all_ok(&mut self) -> Result<Vec<V>, Vec<E>> {
        let (values, errors) = self.unzip_result();
        if errors.is_empty() {
            Ok(values)
        } else {
            Err(errors)
        }
    }
}
