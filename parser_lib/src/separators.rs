use crate::{
    log_end, log_eof, log_error, log_start, ParseError, ParseResult, Parser, VecWindow, Word,
};
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
        let type_name = format!("SeparatedBy<{}>", BY::SEPARATOR);
        log_start(&type_name);
        let split_words = words
            .clone()
            .split(|word| word.get_word().is_some_and(|t| t == BY::SEPARATOR));
        let mut res = Vec::new();
        let mut errors = Vec::new();
        let len = split_words.len();
        for (i, split_word) in split_words.into_iter().enumerate() {
            let ParseResult(item, new_words, new_errors) = T::parse(split_word);
            let no_errors = new_errors.is_empty();
            errors.extend(new_errors);
            if let Some(item) = item {
                res.push(item);
            }
            if i == len - 1 {
                words = new_words;
            } else if !new_words.is_empty() && no_errors {
                if let Some(word) = new_words.first() {
                    log_eof(&type_name);
                    errors.push(ParseError {
                        expected: BY::SEPARATOR.to_string(),
                        got: Some(word.clone()),
                        unlikely: false,
                    });
                }
            }
        }
        log_end(&type_name);
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
        assert_eq!(res.unwrap().0, vec![2, 3]);
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
        assert_eq!(errors.len(), 1);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeparatedOnce<BY: SeparatedBySeparator, A, B>(A, B, PhantomData<BY>);

impl<BY: SeparatedBySeparator, A: Parser<A>, B: Parser<B>> Parser<SeparatedOnce<BY, A, B>>
    for SeparatedOnce<BY, A, B>
{
    fn parse(words: VecWindow<Word>) -> ParseResult<SeparatedOnce<BY, A, B>> {
        let type_name = format!("SeparatedOnce<{}>", BY::SEPARATOR);
        log_start(&type_name);
        println!("{:?}", words);
        let Some((first, second)) = words
            .clone()
            .split_once(|word| word.get_word().is_some_and(|t| t == BY::SEPARATOR))
        else {
            log_eof(&type_name);
            return ParseResult(
                None,
                words,
                vec![ParseError {
                    expected: BY::SEPARATOR.to_string(),
                    got: None,
                    unlikely: false,
                }],
            );
        };
        let first_word = first.first().cloned();
        let ParseResult(res1, words, mut errors) = A::parse(first);
        let Some(res1) = res1 else {
            log_error(&type_name, &first_word);
            return ParseResult(None, words, errors);
        };
        if let Some(word) = words.first() {
            log_error(&type_name, &word);
            errors.push(ParseError {
                expected: BY::SEPARATOR.to_string(),
                got: Some(word.clone()),
                unlikely: false,
            });
            return ParseResult(None, words, errors);
        }
        let second_word = second.first().cloned();
        let ParseResult(res2, words, new_errors) = B::parse(second);
        errors.extend(new_errors);
        let Some(res2) = res2 else {
            log_error(&type_name, &second_word);
            return ParseResult(None, words, errors);
        };
        log_end(&type_name);
        ParseResult(Some(SeparatedOnce(res1, res2, PhantomData)), words, errors)
    }
}

#[cfg(test)]
mod test_separated_once {
    use super::*;
    use crate::split_words;

    use crate as parser_lib;
    separator!(Comma = ",");

    #[test]
    fn valid() {
        let input = "1,2,3";
        let words = split_words(input, vec![]);
        let ParseResult(res, words, errors) =
            SeparatedOnce::<Comma, i64, i64>::parse((&words).into());
        let value = res.unwrap();
        assert_eq!(value.0, 1);
        assert_eq!(value.1, 2);
        assert_eq!(words.size(), 2);
        assert_eq!(errors.len(), 0)
    }
    #[test]
    fn invalid() {
        let input = "1,";
        let words = split_words(input, vec![]);
        let ParseResult(res, words, errors) =
            SeparatedOnce::<Comma, i64, i64>::parse((&words).into());
        assert!(res.is_none());
        assert_eq!(words.size(), 0);
        assert_eq!(errors.len(), 1);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartTextVec<T>(Vec<T>);

impl<T: Parser<T>> Parser<StartTextVec<T>> for StartTextVec<T> {
    fn parse(words: VecWindow<Word>) -> ParseResult<StartTextVec<T>> {
        log_start("StartTextVec");
        let statement_keywords = T::starting_keywords();
        let parts = words.clone().split_including_start(|word| {
            statement_keywords.contains(&word.get_word().unwrap_or(""))
        });
        let mut res = Vec::new();
        let mut errors = Vec::new();
        for part in parts {
            let ParseResult(item, new_words, new_errors) = T::parse(part);
            let no_errors = new_errors.is_empty();
            errors.extend(new_errors);
            if let Some(item) = item {
                res.push(item);
            }
            if no_errors && !new_words.is_empty() {
                if let Some(word) = new_words.first() {
                    log_error("StartTextVec", &word);
                    errors.push(ParseError {
                        expected: "[end of statement]".to_string(),
                        got: Some(word.clone()),
                        unlikely: false,
                    });
                }
            }
        }
        log_end("StartTextVec");
        ParseResult(Some(StartTextVec(res)), words.empty(), errors)
    }
}

#[cfg(test)]
mod test_statement_vec {
    use super::*;
    use crate::split_words;

    use crate as parser_lib;
    separator!(Comma = ",");

    #[derive(Parser)]
    struct FancyInt {
        #[text = "int"]
        value: i64,
    }

    #[test]
    fn valid() {
        let input = "int 1 int 2 int 3";
        let words = split_words(input, vec![]);
        let ParseResult(res, words, errors) = StartTextVec::<FancyInt>::parse((&words).into());
        let value = res.unwrap();
        assert_eq!(value.0.len(), 3);
        assert_eq!(value.0[0].value, 1);
        assert_eq!(value.0[1].value, 2);
        assert_eq!(value.0[2].value, 3);
        assert_eq!(words.size(), 0);
        assert!(errors.is_empty());
    }

    #[test]
    fn invalid() {
        let input = "int 1 int 2 int";
        let words = split_words(input, vec![]);
        let ParseResult(res, words, errors) = StartTextVec::<FancyInt>::parse((&words).into());
        let value = res.unwrap();
        assert_eq!(value.0.len(), 2);
        assert_eq!(value.0[0].value, 1);
        assert_eq!(value.0[1].value, 2);
        assert_eq!(words.size(), 0);
        println!("errors: {:?}", errors);
        assert_eq!(errors.len(), 1);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NonEmptyStartTextVec<T>(Vec<T>);

impl <T: Parser<T>> Parser<NonEmptyStartTextVec<T>> for NonEmptyStartTextVec<T> {
    fn parse(words: VecWindow<Word>) -> ParseResult<NonEmptyStartTextVec<T>> {
        let ParseResult(res, words, errors) = StartTextVec::<T>::parse(words);
        if let Some(ref res) = res {
            if res.0.is_empty() {
                log_error("NonEmptyStartTextVec", &words.first());
                return ParseResult(
                    None,
                    words,
                    errors,
                );
            }
        }
        ParseResult(
            res.map(|r| NonEmptyStartTextVec(r.0)),
            words,
            errors,
        )
    }
}