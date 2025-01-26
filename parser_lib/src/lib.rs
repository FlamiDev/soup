mod split_words;
mod vec_window;

pub use parser_lib_macros::Parser;
pub use split_words::{split_words, Word};
pub use vec_window::VecWindow;

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

fn parse_helper<'l, T>(
    mut words: VecWindow<'l, Word>,
    message: &'static str,
    parse_one: fn(&Word) -> Option<T>,
) -> ParseResult<'l, T> {
    let Some(word) = words.pop_first() else {
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
        ParseResult(Some(res), words, Vec::new())
    } else {
        ParseResult(
            None,
            words,
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
            ParseResult(Some(word.clone()), words, Vec::new())
        } else {
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

/*
* Implementations for compound types
*/

impl<T: Parser<T>> Parser<Vec<T>> for Vec<T> {
    fn parse(mut words: VecWindow<Word>) -> ParseResult<Vec<T>> {
        let mut res = Vec::new();
        let mut errors = Vec::new();
        while !words.is_empty() {
            let ParseResult(item, new_words, new_errors) = T::parse(words);
            words = new_words;
            errors.extend(new_errors);
            if let Some(item) = item {
                res.push(item);
            } else {
                break;
            }
        }
        ParseResult(Some(res), words, errors)
    }
}

impl<T: Parser<Out>, Out> Parser<Option<Out>> for Option<T> {
    fn parse(words: VecWindow<Word>) -> ParseResult<Option<Out>> {
        let ParseResult(res, words, errors) = T::parse(words);
        ParseResult(Some(res), words, errors)
    }
}

impl<T: Parser<Out>, Out> Parser<Box<Out>> for Box<T> {
    fn parse(words: VecWindow<Word>) -> ParseResult<Box<Out>> {
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
        let ParseResult(res1, words, errors1) = T1::parse(words);
        let ParseResult(res2, words, errors2) = T2::parse(words);
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
