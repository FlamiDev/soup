use crate::{ParseError, ParseResult, Parser, VecWindow, Word};
use std::marker::PhantomData;

pub trait SeparatedBySeparator {
    const SEPARATOR: &'static str;
}

#[macro_export]
macro_rules! separator {
    ($name:ident = $sep:literal) => {
        #[derive(Clone, Debug, PartialEq, Parser)]
        pub struct $name {}
        impl parser_lib::SeparatedBySeparator for $name {
            const SEPARATOR: &'static str = $sep;
        }
    };
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeparatedBy<BY: SeparatedBySeparator, T>(Vec<T>, PhantomData<BY>);

impl<BY: SeparatedBySeparator, T: Parser<T>> Parser<SeparatedBy<BY, T>> for SeparatedBy<BY, T> {
    fn parse(mut words: VecWindow<Word>) -> ParseResult<SeparatedBy<BY, T>> {
        log::info!("- SeparatedBy {}", BY::SEPARATOR);
        let split_words = words
            .clone()
            .split(|word| word.get_word().is_some_and(|t| t == BY::SEPARATOR));
        let mut res = Vec::new();
        let mut errors = Vec::new();
        let len = split_words.len();
        for (i, split_word) in split_words.into_iter().enumerate() {
            let ParseResult(item, new_words, new_errors) = T::parse(split_word);
            errors.extend(new_errors);
            if let Some(item) = item {
                res.push(item);
            }
            if i == len - 1 {
                words = new_words;
            } else if !new_words.is_empty() {
                if let Some(word) = new_words.first() {
                    log::debug!("! SeparatedBy {} - end_part !! {}", BY::SEPARATOR, word);
                    errors.push(ParseError {
                        expected: BY::SEPARATOR.to_string(),
                        got: Some(word.clone()),
                    });
                }
            }
        }
        log::info!("> SeparatedBy {}", BY::SEPARATOR);
        ParseResult(Some(SeparatedBy(res, PhantomData)), words, errors)
    }
}

#[cfg(test)]
mod test_separated_by {
    use super::*;
    use crate::split_words;

    use crate as parser_lib;
    separator!(Comma = ",");

    #[test]
    fn valid() {
        let input = "1,2,3";
        let words = split_words(input, vec![]);
        let ParseResult(res, _, errors) = SeparatedBy::<Comma, i64>::parse((&words).into());
        assert_eq!(res.unwrap().0, vec![1, 2, 3]);
        assert!(errors.is_empty());
    }
    #[test]
    fn valid_empty() {
        let input = "";
        let words = split_words(input, vec![]);
        let ParseResult(res, words, errors) = SeparatedBy::<Comma, i64>::parse((&words).into());
        assert!(res.unwrap().0.is_empty());
        assert_eq!(words.size(), 0);
        assert!(errors.is_empty());
    }
    #[test]
    fn invalid_trailing() {
        let input = "1,2,3,";
        let words = split_words(input, vec![]);
        let ParseResult(res, words, errors) = SeparatedBy::<Comma, i64>::parse((&words).into());
        assert_eq!(res.unwrap().0, vec![1, 2, 3]);
        assert_eq!(words.size(), 0);
        assert_eq!(errors.len(), 1);
    }
    #[test]
    fn invalid_leading() {
        let input = ",1,2,3";
        let words = split_words(input, vec![]);
        
        let ParseResult(res, words, errors) = SeparatedBy::<Comma, i64>::parse((&words).into());
        assert_eq!(res.unwrap().0, vec![1, 2, 3]);
        assert_eq!(words.size(), 0);
        assert_eq!(errors.len(), 1);
    }
    #[test]
    fn invalid_value() {
        let input = "1,b,3";
        let words = split_words(input, vec![]);
        let ParseResult(res, words, errors) = SeparatedBy::<Comma, i64>::parse((&words).into());
        assert_eq!(res.unwrap().0, vec![1, 3]);
        assert_eq!(words.size(), 0);
        println!("{:?}", errors);
        assert_eq!(errors.len(), 1);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeparatedOnce<BY: SeparatedBySeparator, A, B>(A, B, PhantomData<BY>);

impl<BY: SeparatedBySeparator, A: Parser<A>, B: Parser<B>> Parser<SeparatedOnce<BY, A, B>>
    for SeparatedOnce<BY, A, B>
{
    fn parse(words: VecWindow<Word>) -> ParseResult<SeparatedOnce<BY, A, B>> {
        let Some((first, second)) = words
            .clone()
            .split_once(|word| word.get_word().is_some_and(|t| t == BY::SEPARATOR))
        else {
            log::debug!("! SeparatedOnce {} !! EOF", BY::SEPARATOR);
            return ParseResult(
                None,
                words,
                vec![ParseError {
                    expected: BY::SEPARATOR.to_string(),
                    got: None,
                }],
            );
        };
        let ParseResult(res1, words, mut errors) = A::parse(first);
        let Some(res1) = res1 else {
            log::debug!("! SeparatedOnce {} !! first_part", BY::SEPARATOR);
            return ParseResult(None, words, errors);
        };
        if let Some(word) = words.first() {
            log::debug!("! SeparatedOnce {} - separator !! {}", BY::SEPARATOR, word);
            errors.push(ParseError {
                expected: BY::SEPARATOR.to_string(),
                got: Some(word.clone()),
            });
            return ParseResult(None, words, errors);
        }
        let ParseResult(res2, words, new_errors) = B::parse(second);
        let Some(res2) = res2 else {
            log::debug!("! SeparatedOnce {} !! second_part", BY::SEPARATOR);
            return ParseResult(None, words, errors);
        };
        errors.extend(new_errors);
        log::info!("> SeparatedOnce {}", BY::SEPARATOR);
        ParseResult(Some(SeparatedOnce(res1, res2, PhantomData)), words, errors)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatementVec<T>(Vec<T>);

impl<T: Parser<T>> Parser<StatementVec<T>> for StatementVec<T> {
    fn parse(words: VecWindow<Word>) -> ParseResult<StatementVec<T>> {
        let statement_keywords = T::starting_keywords();
        let parts = words.clone().split_including_start(|word| {
            statement_keywords.contains(&word.get_word().unwrap_or(""))
        });
        let mut res = Vec::new();
        let mut errors = Vec::new();
        for part in parts {
            let ParseResult(item, new_words, new_errors) = T::parse(part);
            if let Some(item) = item {
                res.push(item);
            }
            errors.extend(new_errors);
            if let Some(word) = new_words.first() {
                log::debug!("! StatementVec - end_part !! {}", word);
                errors.push(ParseError {
                    expected: "[end of statement]".to_string(),
                    got: Some(word.clone()),
                });
            }
        }
        ParseResult(Some(StatementVec(res)), words.empty(), errors)
    }
}
