use std::io::Write;

mod basics;
mod brackets;
mod collections;
mod separators;
mod split_words;
mod vec_window;

pub use basics::*;
pub use brackets::*;
pub use collections::*;
pub use log;
pub use parser_lib_macros::Parser;
pub use separators::*;
pub use split_words::{split_words, BracketPair, Word};
pub use vec_window::VecWindow;

pub fn setup_logging(debug: bool) {
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter_level(if debug {
            log::LevelFilter::max()
        } else {
            log::LevelFilter::Info
        })
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
    fn starting_keywords() -> Vec<&'static str> {
        Vec::new()
    }
}

#[inline(always)]
pub fn parse_to_type<T>(words: VecWindow<Word>) -> ParseResult<T>
where
    T: Parser<T>,
{
    T::parse(words)
}

/*
* Implementations for compound types
*/

impl<T: Parser<Out>, Out> Parser<Option<Out>> for Option<T> {
    fn parse(words: VecWindow<Word>) -> ParseResult<Option<Out>> {
        log::info!("- Option");
        let ParseResult(res, words, errors) = T::parse(words);
        if let Some(res) = res {
            ParseResult(Some(Some(res)), words, errors)
        } else {
            ParseResult(Some(None), words, Vec::new())
        }
    }
}

#[cfg(test)]
mod test_parse_option {
    use super::*;
    #[test]
    fn valid_existing() {
        let words = split_words("1 a true", vec![]);
        let result = Option::<i64>::parse((&words).into());
        assert_eq!(result.0, Some(Some(1)));
        assert_eq!(result.1.size(), 2);
        assert_eq!(result.2.len(), 0);
    }
    #[test]
    fn valid_none() {
        let words = split_words("a true", vec![]);
        let result = Option::<i64>::parse((&words).into());
        assert_eq!(result.0, Some(None));
        assert_eq!(result.1.size(), 2);
        assert_eq!(result.2.len(), 0);
    }
    #[test]
    fn invalid() {
        let words = split_words("a", vec![]);
        let result = Option::<i64>::parse((&words).into());
        assert_eq!(result.0, Some(None));
        assert_eq!(result.1.size(), 1);
        assert_eq!(result.2.len(), 0);
    }
}

impl<T: Parser<Out>, Out> Parser<Box<Out>> for Box<T> {
    fn parse(words: VecWindow<Word>) -> ParseResult<Box<Out>> {
        log::info!("- Box");
        let ParseResult(res, words, errors) = T::parse(words);
        ParseResult(res.map(Box::new), words, errors)
    }
}

#[cfg(test)]
mod test_parse_box {
    use super::*;
    #[test]
    fn valid() {
        let words = split_words("1 a true", vec![]);
        let result = Box::<i64>::parse((&words).into());
        assert_eq!(result.0, Some(Box::new(1)));
        assert_eq!(result.1.size(), 2);
        assert_eq!(result.2.len(), 0);
    }
    #[test]
    fn invalid() {
        let words = split_words("a", vec![]);
        let result = Box::<i64>::parse((&words).into());
        assert_eq!(result.0, None);
        assert_eq!(result.1.size(), 1);
        assert_eq!(result.2.len(), 1);
    }
}
