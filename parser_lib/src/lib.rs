use std::io::Write;
mod split_words;
mod vec_window;

pub use log;
pub use parser_lib_macros::Parser;
pub use split_words::{split_words, Word};
pub use vec_window::VecWindow;

pub fn setup_logging() {
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter_level(log::LevelFilter::max())
        .init();
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    pub expected: String,
    pub got: Option<Word>,
}

#[derive(Debug)]
pub struct ParseResult<'l, Out>(
    pub Option<Out>,
    pub VecWindow<'l, Word>,
    pub Vec<ParseError>,
);

pub trait Parser<Out> {
    fn parse(words: VecWindow<Word>) -> ParseResult<Out>;
}

#[inline(always)]
pub fn parse_to_type<T>(words: VecWindow<Word>) -> ParseResult<T>
where
    T: Parser<T>,
{
    T::parse(words)
}

/*
* Implementations for basic types
*/

#[inline(always)]
fn parse_helper<'l, T>(
    words: VecWindow<'l, Word>,
    message: &'static str,
    parse_one: fn(&Word) -> Option<T>,
) -> ParseResult<'l, T> {
    let Some(word) = words.first() else {
        log::debug!("{} !! EOF", message);
        return ParseResult(
            None,
            words,
            vec![ParseError {
                expected: message.to_string(),
                got: None,
            }],
        );
    };
    if let Some(res) = parse_one(word) {
        log::debug!("{} -> {}", message, word.text);
        ParseResult(Some(res), words.skip(1), Vec::new())
    } else {
        log::debug!("{} !! {}", message, word.text);
        ParseResult(
            None,
            words.clone(),
            vec![ParseError {
                expected: message.to_string(),
                got: Some(word.clone()),
            }],
        )
    }
}

impl Parser<String> for String {
    fn parse(words: VecWindow<Word>) -> ParseResult<String> {
        parse_helper(words, "<<string>>", |word| {
            (word.text.starts_with('"') && word.text.ends_with('"'))
                .then(|| word.text[1..word.text.len() - 1].to_string())
        })
    }
}

impl Parser<i64> for i64 {
    fn parse(words: VecWindow<Word>) -> ParseResult<i64> {
        parse_helper(words, "<<integer>>", |word| word.text.parse::<i64>().ok())
    }
}

impl Parser<f64> for f64 {
    fn parse(words: VecWindow<Word>) -> ParseResult<f64> {
        parse_helper(words, "<<decimal>>", |word| word.text.parse::<f64>().ok())
    }
}

impl Parser<bool> for bool {
    fn parse(words: VecWindow<Word>) -> ParseResult<bool> {
        parse_helper(words, "<<boolean>>", |word| match word.text.as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        })
    }
}

impl Parser<Word> for Word {
    fn parse(mut words: VecWindow<Word>) -> ParseResult<Word> {
        if let Some(word) = words.pop_first() {
            log::debug!("<<Word>> -> {}", word.text);
            ParseResult(Some(word.clone()), words, Vec::new())
        } else {
            log::debug!("<<Word>> !! EOF");
            ParseResult(
                None,
                words,
                vec![ParseError {
                    expected: "<<anything>>".to_string(),
                    got: None,
                }],
            )
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeName {
    pub text: String,
    pub line_number: usize,
    pub column_number: usize,
}

impl Parser<TypeName> for TypeName {
    fn parse(words: VecWindow<Word>) -> ParseResult<TypeName> {
        parse_helper(words, "<<TypeName - PascalCase>>", |word| {
            let starts_uppercase = word.text.chars().next().is_some_and(|c| c.is_uppercase());
            let all_alphabetic = word.text.chars().all(|c| c.is_alphabetic());
            if starts_uppercase && all_alphabetic {
                Some(TypeName {
                    text: word.text.clone(),
                    line_number: word.line_number,
                    column_number: word.column_number,
                })
            } else {
                None
            }
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueName {
    pub text: String,
    pub line_number: usize,
    pub column_number: usize,
}

impl Parser<ValueName> for ValueName {
    fn parse(words: VecWindow<Word>) -> ParseResult<ValueName> {
        parse_helper(words, "<<ValueName - snake_case>>", |word| {
            let all_lowercase_or_underscore =
                word.text.chars().all(|c| c.is_lowercase() || c == '_');
            if !word.text.is_empty() || all_lowercase_or_underscore {
                Some(ValueName {
                    text: word.text.clone(),
                    line_number: word.line_number,
                    column_number: word.column_number,
                })
            } else {
                None
            }
        })
    }
}

/*
* Implementations for compound types
*/

impl<T: Parser<T>> Parser<Vec<T>> for Vec<T> {
    fn parse(mut words: VecWindow<Word>) -> ParseResult<Vec<T>> {
        let mut res = Vec::new();
        let mut errors = Vec::new();
        log::debug!(">> Vec");
        while !words.is_empty() {
            let ParseResult(item, new_words, new_errors) = T::parse(words);
            words = new_words;
            errors.extend(new_errors);
            if let Some(item) = item {
                res.push(item);
                log::debug!("--");
            } else {
                break;
            }
        }
        log::debug!("<< Vec");
        ParseResult(Some(res), words, errors)
    }
}

impl<T: Parser<Out>, Out> Parser<Option<Out>> for Option<T> {
    fn parse(words: VecWindow<Word>) -> ParseResult<Option<Out>> {
        log::debug!("-- Option");
        let ParseResult(res, words, errors) = T::parse(words);
        ParseResult(Some(res), words, errors)
    }
}

impl<T: Parser<Out>, Out> Parser<Box<Out>> for Box<T> {
    fn parse(words: VecWindow<Word>) -> ParseResult<Box<Out>> {
        log::debug!("-- Box");
        let ParseResult(res, words, errors) = T::parse(words);
        if let Some(res) = res {
            ParseResult(Some(Box::new(res)), words, errors)
        } else {
            ParseResult(None, words, errors)
        }
    }
}

impl<T1: Parser<Out1>, Out1, T2: Parser<Out2>, Out2> Parser<(Out1, Out2)> for (T1, T2) {
    fn parse(words: VecWindow<Word>) -> ParseResult<(Out1, Out2)> {
        log::debug!(">> 2-tuple");
        let ParseResult(res1, words, errors1) = T1::parse(words);
        let ParseResult(res2, words, errors2) = T2::parse(words);
        log::debug!("<< 2-tuple");
        if let Some(res1) = res1 {
            if let Some(res2) = res2 {
                ParseResult(Some((res1, res2)), words, Vec::new())
            } else {
                ParseResult(None, words, [errors1, errors2].concat())
            }
        } else {
            ParseResult(None, words, errors1)
        }
    }
}

fn brackets_helper<'l, B, T: Parser<T>>(
    mut words: VecWindow<'l, Word>,
    start: &'static str,
    end: &'static str,
    create: fn(T) -> B,
) -> ParseResult<'l, B> {
    log::debug!(">> \"{}\"", start);
    let first = words.pop_first();
    if first.is_some_and(|word| word.text != start) {
        log::debug!("<< \"{}\" !! {:?}", start, first);
        return ParseResult(
            None,
            words,
            vec![ParseError {
                expected: start.to_string(),
                got: first.cloned(),
            }],
        );
    }
    let mut nested = 0;
    let mut inner_count = 0;
    while let Some(word) = words.pop_first() {
        if word.text == start {
            nested += 1;
        } else if word.text == "]" {
            if nested == 0 {
                break;
            } else {
                nested -= 1;
            }
        }
        inner_count += 1;
    }
    log::debug!("inner: {}", inner_count);
    let inner_words = words.clone().take(inner_count);
    let ParseResult(inner_res, words, errors) = T::parse(inner_words);
    log::debug!("<< \"{}\"", end);
    ParseResult(inner_res.map(create), words, errors)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SquareBrackets<T>(T);

impl<T: Parser<T>> Parser<SquareBrackets<T>> for SquareBrackets<T> {
    fn parse(words: VecWindow<Word>) -> ParseResult<SquareBrackets<T>> {
        brackets_helper(words, "[", "]", SquareBrackets)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurlyBrackets<T>(T);

impl<T: Parser<T>> Parser<CurlyBrackets<T>> for CurlyBrackets<T> {
    fn parse(words: VecWindow<Word>) -> ParseResult<CurlyBrackets<T>> {
        brackets_helper(words, "{", "}", CurlyBrackets)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parentheses<T>(T);

impl<T: Parser<T>> Parser<Parentheses<T>> for Parentheses<T> {
    fn parse(words: VecWindow<Word>) -> ParseResult<Parentheses<T>> {
        brackets_helper(words, "(", ")", Parentheses)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommaSeparated<T>(Vec<T>);

impl<T: Parser<T>> Parser<CommaSeparated<T>> for CommaSeparated<T> {
    fn parse(words: VecWindow<Word>) -> ParseResult<CommaSeparated<T>> {
        log::debug!(">> comma-separated");
        let split_words = words.split(|word| word.text == ",");
        let mut res = Vec::new();
        let mut errors = Vec::new();
        let len = split_words.len();
        for (i, split_word) in split_words.into_iter().enumerate() {
            let ParseResult(item, new_words, new_errors) = T::parse(split_word);
            errors.extend(new_errors);
            if let Some(item) = item {
                res.push(item);
            }
            if i < len - 1 && !new_words.is_empty() {
                if let Some(word) = new_words.first() {
                    log::debug!("<< comma-separated !! end_part !! {:?}", word);
                    errors.push(ParseError {
                        expected: ",".to_string(),
                        got: Some(word.clone()),
                    });
                }
            }
        }
        log::debug!("<< comma-separated");
        ParseResult(Some(CommaSeparated(res)), words, errors)
    }
}
