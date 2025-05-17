use std::io::Write;

mod basics;
mod boxes;
mod brackets;
mod collections;
mod separators;
mod split_words;
mod vec_window;

pub use basics::*;
pub use boxes::*;
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
