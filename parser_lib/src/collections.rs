use crate::{ParseError, ParseResult, Parser, VecWindow, Word};

impl<T: Parser<T>> Parser<Vec<T>> for Vec<T> {
    fn parse(mut words: VecWindow<Word>) -> ParseResult<Vec<T>> {
        let mut res = Vec::new();
        let mut errors = Vec::new();
        log::info!("- Vec");
        while !words.is_empty() {
            let ParseResult(item, new_words, new_errors) = T::parse(words);
            words = new_words;
            if let Some(item) = item {
                errors.extend(new_errors);
                res.push(item);
                log::info!("--");
            } else {
                break;
            }
        }
        log::info!("> Vec");
        ParseResult(Some(res), words, errors)
    }
}

#[cfg(test)]
mod test_parse_vec {
    use super::*;
    use crate::split_words;

    #[test]
    fn valid() {
        let input = "1 2 3";
        let words = split_words(input, vec![]);
        let ParseResult(res, _, errors) = Vec::<i64>::parse((&words).into());
        assert_eq!(res, Some(vec![1, 2, 3]));
        assert!(errors.is_empty());
    }
    #[test]
    fn valid_empty() {
        let input = "";
        let words = split_words(input, vec![]);
        let ParseResult(res, _, errors) = Vec::<i64>::parse((&words).into());
        assert_eq!(res, Some(vec![]));
        assert!(errors.is_empty());
    }
    #[test]
    fn invalid() {
        let input = "1 2 a";
        let words = split_words(input, vec![]);
        let ParseResult(res, words, errors) = Vec::<i64>::parse((&words).into());
        assert_eq!(res, Some(vec![1, 2]));
        assert_eq!(errors.len(), 0);
        assert_eq!(words.size(), 1);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NonEmptyVec<T>(Vec<T>);

impl<T: Parser<T>> Parser<NonEmptyVec<T>> for NonEmptyVec<T> {
    fn parse(words: VecWindow<Word>) -> ParseResult<NonEmptyVec<T>> {
        let ParseResult(res, words, mut errors) = Vec::<T>::parse(words);
        if let Some(res) = res {
            if res.is_empty() {
                errors.push(ParseError {
                    expected: "[one or more items]".to_string(),
                    got: None,
                });
                ParseResult(None, words, errors)
            } else {
                ParseResult(Some(NonEmptyVec(res)), words, errors)
            }
        } else {
            ParseResult(None, words, errors)
        }
    }
}

#[cfg(test)]
mod test_parse_non_empty_vec {
    use super::*;
    use crate::split_words;

    #[test]
    fn valid() {
        let input = "1 2 3";
        let words = split_words(input, vec![]);
        let ParseResult(res, _, errors) = NonEmptyVec::<i64>::parse((&words).into());
        assert_eq!(res, Some(NonEmptyVec(vec![1, 2, 3])));
        assert!(errors.is_empty());
    }
    #[test]
    fn invalid() {
        let input = "";
        let words = split_words(input, vec![]);
        let ParseResult(res, words, errors) = NonEmptyVec::<i64>::parse((&words).into());
        assert_eq!(res, None);
        assert_eq!(errors.len(), 1);
        assert_eq!(words.size(), 0);
    }
}

impl<T1: Parser<Out1>, Out1, T2: Parser<Out2>, Out2> Parser<(Out1, Out2)> for (T1, T2) {
    fn parse(words: VecWindow<Word>) -> ParseResult<(Out1, Out2)> {
        log::info!("- 2-tuple");
        let ParseResult(res1, words, errors1) = T1::parse(words);
        let ParseResult(res2, words, errors2) = T2::parse(words);
        log::debug!("> 2-tuple");
        if let Some(res1) = res1 {
            if let Some(res2) = res2 {
                ParseResult(Some((res1, res2)), words, [errors1, errors2].concat())
            } else {
                ParseResult(None, words, errors2)
            }
        } else {
            ParseResult(None, words, errors1)
        }
    }
}

#[cfg(test)]
mod test_parse_tuple {
    use super::*;
    use crate::split_words;

    #[test]
    fn valid() {
        let input = "1 2";
        let words = split_words(input, vec![]);
        let ParseResult(res, _, errors) = <(i64, i64)>::parse((&words).into());
        assert_eq!(res, Some((1, 2)));
        assert!(errors.is_empty());
    }
    #[test]
    fn invalid() {
        let input = "1 a";
        let words = split_words(input, vec![]);
        let ParseResult(res, words, errors) = <(i64, i64)>::parse((&words).into());
        assert_eq!(res, None);
        assert_eq!(errors.len(), 1);
        assert_eq!(words.size(), 1);
    }
    #[test]
    fn invalid_both() {
        let input = "a b";
        let words = split_words(input, vec![]);
        let ParseResult(res, words, errors) = <(i64, i64)>::parse((&words).into());
        assert_eq!(res, None);
        assert_eq!(errors.len(), 1);
        assert_eq!(words.size(), 2);
    }
    #[test]
    fn invalid_empty() {
        let input = "";
        let words = split_words(input, vec![]);
        let ParseResult(res, words, errors) = <(i64, i64)>::parse((&words).into());
        assert_eq!(res, None);
        assert_eq!(errors.len(), 1);
        assert_eq!(words.size(), 0);
    }
}
