use std::fmt::Debug;
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

pub fn setup_logging() {
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter_level(log::LevelFilter::max())
        .init();
}

#[inline(always)]
pub fn log_start(type_name: &str) {
    log::debug!("\x1b[37m{:25} parsing\x1b[0m", type_name);
}
#[inline(always)]
pub fn log_parsed<T: Debug>(type_name: &str, value: &T) {
    log::debug!("\x1b[32m{:25} parsed {:?}\x1b[0m", type_name, value);
}
#[inline(always)]
pub fn log_message(type_name: &str, message: &str) {
    log::debug!("\x1b[33m{:25} {}\x1b[0m", type_name, message);
}
#[inline(always)]
pub fn log_error<T: Debug>(type_name: &str, value: &T) {
    log::debug!("\x1b[31m{:25} error on {:?}\x1b[0m", type_name, value);
}
#[inline(always)]
pub fn log_eof(type_name: &str) {
    log::debug!("\x1b[31m{:25} EOF\x1b[0m", type_name);
}
#[inline(always)]
pub fn log_end(type_name: &str) {
    log::debug!("\x1b[34m{:25} end\x1b[0m", type_name);
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
