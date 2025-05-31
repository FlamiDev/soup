use crate::{
    log_end, log_eof, log_error, log_start, ParseError, ParseResult, Parser, VecWindow, Word,
};

fn brackets_helper<B, T: Parser<T>>(
    mut words: VecWindow<Word>,
    start: char,
    end: char,
    create: fn(T) -> B,
) -> ParseResult<B> {
    let type_name = format!("{}{}", start, end);
    let Some(first) = words.first() else {
        log_eof(&type_name);
        return ParseResult(
            None,
            words.clone(),
            vec![ParseError {
                expected: start.to_string(),
                got: None,
                unlikely: false,
            }],
        );
    };
    let Some(inner) = first.get_brackets(start, end) else {
        log_error(&type_name, first);
        return ParseResult(
            None,
            words.clone(),
            vec![ParseError {
                expected: start.to_string(),
                got: Some(first.clone()),
                unlikely: false,
            }],
        );
    };
    log_start(&type_name);
    let ParseResult(inner_res, inner_words, errors) = T::parse(VecWindow::from(inner));
    if errors.is_empty() {
        if let Some(word) = inner_words.first() {
            log_error(&type_name, word);
            return ParseResult(
                None,
                words.clone(),
                vec![ParseError {
                    expected: end.to_string(),
                    got: Some(word.clone()),
                    unlikely: false,
                }],
            );
        }
    }
    log_end(&type_name);
    words.pop_first();
    ParseResult(inner_res.map(create), words, errors)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SquareBrackets<T>(T);

impl<T: Parser<T>> Parser<SquareBrackets<T>> for SquareBrackets<T> {
    fn parse(words: VecWindow<Word>) -> ParseResult<SquareBrackets<T>> {
        brackets_helper(words, '[', ']', SquareBrackets)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurlyBrackets<T>(T);

impl<T: Parser<T>> Parser<CurlyBrackets<T>> for CurlyBrackets<T> {
    fn parse(words: VecWindow<Word>) -> ParseResult<CurlyBrackets<T>> {
        brackets_helper(words, '{', '}', CurlyBrackets)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parentheses<T>(T);

impl<T: Parser<T>> Parser<Parentheses<T>> for Parentheses<T> {
    fn parse(words: VecWindow<Word>) -> ParseResult<Parentheses<T>> {
        brackets_helper(words, '(', ')', Parentheses)
    }
}

// all brackets use the same helper function
// so we only need to test one of them
#[cfg(test)]
mod test_parse_brackets {
    use super::*;
    use crate::{split_words, BracketPair};

    const BRACKET_PAIRS: [BracketPair; 3] = [
        BracketPair {
            open: '{',
            close: '}',
        },
        BracketPair {
            open: '(',
            close: ')',
        },
        BracketPair {
            open: '[',
            close: ']',
        },
    ];

    #[test]
    fn valid_square() {
        let words = split_words("[1]", BRACKET_PAIRS.into());
        let result = SquareBrackets::<i64>::parse((&words).into());
        assert_eq!(result.0, Some(SquareBrackets(1)));
        assert_eq!(result.1.size(), 0);
        assert_eq!(result.2, vec![]);
    }
    #[test]
    fn valid_curly() {
        let words = split_words("{1}", BRACKET_PAIRS.into());
        let result = CurlyBrackets::<i64>::parse((&words).into());
        assert_eq!(result.0, Some(CurlyBrackets(1)));
        assert_eq!(result.1.size(), 0);
        assert_eq!(result.2, vec![]);
    }
    #[test]
    fn valid_parentheses() {
        let words = split_words("(1)", BRACKET_PAIRS.into());
        let result = Parentheses::<i64>::parse((&words).into());
        assert_eq!(result.0, Some(Parentheses(1)));
        assert_eq!(result.1.size(), 0);
        assert_eq!(result.2, vec![]);
    }
    #[test]
    fn invalid_inside() {
        let words = split_words("[a]", BRACKET_PAIRS.into());
        let result = SquareBrackets::<i64>::parse((&words).into());
        assert_eq!(result.0, None);
        assert_eq!(result.1.size(), 1);
        assert_eq!(result.2.len(), 1);
    }
    #[test]
    fn invalid_no_brackets() {
        let words = split_words("1 2", BRACKET_PAIRS.into());
        let result = SquareBrackets::<i64>::parse((&words).into());
        assert_eq!(result.0, None);
        assert_eq!(result.1.size(), 2);
        assert_eq!(result.2.len(), 1);
    }
    #[test]
    fn invalid_did_not_expect_more() {
        let words = split_words("[1 2]", BRACKET_PAIRS.into());
        let result = SquareBrackets::<i64>::parse((&words).into());
        assert_eq!(result.0, None);
        assert_eq!(result.1.size(), 1);
        assert_eq!(result.2.len(), 1);
    }
}
