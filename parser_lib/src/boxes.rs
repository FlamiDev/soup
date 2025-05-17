use crate::{ParseResult, Parser, VecWindow, Word};

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
    use crate::split_words;
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
    use crate::split_words;
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
