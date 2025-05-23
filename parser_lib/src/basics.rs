use crate::{
    log_eof, log_error, log_parsed, log_start, ParseError, ParseResult, Parser, VecWindow, Word,
};

impl Parser<Word> for Word {
    fn parse(words: VecWindow<Word>) -> ParseResult<Word> {
        ParseResult(words.first().cloned(), words.skip(1), Vec::new())
    }
}

#[inline(always)]
fn parse_helper<'l, T>(
    words: VecWindow<'l, Word>,
    type_name: &'static str,
    parse_one: fn(&Word) -> Option<T>,
) -> ParseResult<'l, T> {
    log_start(type_name);
    let Some(word) = words.first() else {
        log_eof(type_name);
        return ParseResult(
            None,
            words,
            vec![ParseError {
                expected: type_name.to_string(),
                got: None,
                unlikely: false,
            }],
        );
    };
    if let Some(res) = parse_one(word) {
        log_parsed(type_name, word);
        ParseResult(Some(res), words.skip(1), Vec::new())
    } else {
        log_error(type_name, word);
        ParseResult(
            None,
            words.clone(),
            vec![ParseError {
                expected: type_name.to_string(),
                got: Some(word.clone()),
                unlikely: false,
            }],
        )
    }
}

impl Parser<String> for String {
    fn parse(words: VecWindow<Word>) -> ParseResult<String> {
        parse_helper(words, "<<string>>", |word| {
            let word = word.get_word()?;
            (word.starts_with('"') && word.ends_with('"'))
                .then(|| word[1..word.len() - 1].to_string())
        })
    }
}

#[cfg(test)]
mod test_parse_string {
    use super::*;
    use crate::split_words;
    #[test]
    fn valid() {
        let words = split_words("\"hello\"", vec![]);
        let result = String::parse((&words).into());
        assert_eq!(result.0, Some("hello".to_string()));
        assert_eq!(result.1.size(), 0);
        assert_eq!(result.2.len(), 0);
    }
    #[test]
    fn empty() {
        let words = split_words("\"\"", vec![]);
        let result = String::parse((&words).into());
        assert_eq!(result.0, Some("".to_string()));
        assert_eq!(result.1.size(), 0);
        assert_eq!(result.2.len(), 0);
    }
    #[test]
    fn no_quotes() {
        let words = split_words("hello", vec![]);
        let result = String::parse((&words).into());
        assert_eq!(result.0, None);
        assert_eq!(result.1.size(), 1);
        assert_eq!(result.2.len(), 1);
    }
    #[test]
    fn words_left() {
        let words = split_words("\"hello\" world", vec![]);
        let result = String::parse((&words).into());
        assert_eq!(result.0, Some("hello".to_string()));
        assert_eq!(result.1.size(), 1);
        assert_eq!(result.2.len(), 0);
    }
}

impl Parser<i64> for i64 {
    fn parse(words: VecWindow<Word>) -> ParseResult<i64> {
        parse_helper(words, "<<integer>>", |word| {
            word.get_word()?.parse::<i64>().ok()
        })
    }
}

#[cfg(test)]
mod test_parse_i64 {
    use super::*;
    use crate::split_words;
    #[test]
    fn valid() {
        let words = split_words("123", vec![]);
        let result = i64::parse((&words).into());
        assert_eq!(result.0, Some(123));
        assert_eq!(result.1.size(), 0);
        assert_eq!(result.2.len(), 0);
    }
    #[test]
    fn invalid() {
        let words = split_words("hello", vec![]);
        let result = i64::parse((&words).into());
        assert_eq!(result.0, None);
        assert_eq!(result.1.size(), 1);
        assert_eq!(result.2.len(), 1);
    }
}

impl Parser<f64> for f64 {
    fn parse(words: VecWindow<Word>) -> ParseResult<f64> {
        parse_helper(words, "<<float>>", |word| {
            word.get_word()?.parse::<f64>().ok()
        })
    }
}

#[cfg(test)]
mod test_parse_f64 {
    use super::*;
    use crate::split_words;
    #[test]
    fn valid() {
        let words = split_words("123.456", vec![]);
        let result = f64::parse((&words).into());
        assert_eq!(result.0, Some(123.456));
        assert_eq!(result.1.size(), 0);
        assert_eq!(result.2.len(), 0);
    }
    #[test]
    fn invalid() {
        let words = split_words("hello", vec![]);
        let result = f64::parse((&words).into());
        assert_eq!(result.0, None);
        assert_eq!(result.1.size(), 1);
        assert_eq!(result.2.len(), 1);
    }
}

impl Parser<bool> for bool {
    fn parse(words: VecWindow<Word>) -> ParseResult<bool> {
        parse_helper(words, "<<boolean>>", |word| match word.get_word()? {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        })
    }
}

#[cfg(test)]
mod test_parse_bool {
    use super::*;
    use crate::split_words;
    #[test]
    fn valid_true() {
        let words = split_words("true", vec![]);
        let result = bool::parse((&words).into());
        assert_eq!(result.0, Some(true));
        assert_eq!(result.1.size(), 0);
        assert_eq!(result.2.len(), 0);
    }
    #[test]
    fn valid_false() {
        let words = split_words("false", vec![]);
        let result = bool::parse((&words).into());
        assert_eq!(result.0, Some(false));
        assert_eq!(result.1.size(), 0);
        assert_eq!(result.2.len(), 0);
    }
    #[test]
    fn invalid() {
        let words = split_words("hello", vec![]);
        let result = bool::parse((&words).into());
        assert_eq!(result.0, None);
        assert_eq!(result.1.size(), 1);
        assert_eq!(result.2.len(), 1);
    }
}

impl Parser<()> for () {
    fn parse(words: VecWindow<Word>) -> ParseResult<()> {
        ParseResult(Some(()), words, Vec::new())
    }
}

#[cfg(test)]
mod test_parse_nothing {
    use super::*;
    use crate::split_words;
    #[test]
    fn valid() {
        let words = split_words("hello", vec![]);
        let result = <()>::parse((&words).into());
        assert_eq!(result.0, Some(()));
        assert_eq!(result.1.size(), 1);
        assert_eq!(result.2.len(), 0);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeName {
    pub text: String,
    pub line_number: usize,
    pub column_from: usize,
    pub column_to: usize,
}

impl Parser<TypeName> for TypeName {
    fn parse(words: VecWindow<Word>) -> ParseResult<TypeName> {
        parse_helper(words, "<<TypeName>>", |word| {
            let text = word.get_word()?;
            let starts_uppercase = text.chars().next().is_some_and(|c| c.is_uppercase());
            let all_alphanumeric = text.chars().all(|c| c.is_alphanumeric());
            if starts_uppercase && all_alphanumeric {
                Some(TypeName {
                    text: text.to_string(),
                    line_number: word.line,
                    column_from: word.column_from,
                    column_to: word.column_to,
                })
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod test_parse_type {
    use super::*;
    use crate::split_words;
    #[test]
    fn valid() {
        let words = split_words("Hello", vec![]);
        let result = TypeName::parse((&words).into());
        assert_eq!(
            result.0,
            Some(TypeName {
                text: "Hello".to_string(),
                line_number: 0,
                column_from: 0,
                column_to: 5,
            })
        );
        assert_eq!(result.1.size(), 0);
        assert_eq!(result.2.len(), 0);
    }
    #[test]
    fn valid_multiple() {
        let words = split_words("HelloWorld", vec![]);
        let result = TypeName::parse((&words).into());
        assert_eq!(
            result.0,
            Some(TypeName {
                text: "HelloWorld".to_string(),
                line_number: 0,
                column_from: 0,
                column_to: 10,
            })
        );
        assert_eq!(result.1.size(), 0);
        assert_eq!(result.2.len(), 0);
    }
    #[test]
    fn invalid() {
        let words = split_words("hello", vec![]);
        let result = TypeName::parse((&words).into());
        assert_eq!(result.0, None);
        assert_eq!(result.1.size(), 1);
        assert_eq!(result.2.len(), 1);
    }
    #[test]
    fn invalid_multiple() {
        let words = split_words("helloWorld", vec![]);
        let result = TypeName::parse((&words).into());
        assert_eq!(result.0, None);
        assert_eq!(result.1.size(), 1);
        assert_eq!(result.2.len(), 1);
    }
    #[test]
    fn invalid_underscore() {
        let words = split_words("Hello_World", vec![]);
        let result = TypeName::parse((&words).into());
        assert_eq!(result.0, None);
        assert_eq!(result.1.size(), 1);
        assert_eq!(result.2.len(), 1);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueName {
    pub text: String,
    pub line_number: usize,
    pub column_from: usize,
    pub column_to: usize,
}

impl Parser<ValueName> for ValueName {
    fn parse(words: VecWindow<Word>) -> ParseResult<ValueName> {
        parse_helper(words, "<<ValueName>>", |word| {
            let text = word.get_word()?;
            let starts_lowercase = text.chars().next().is_some_and(|c| c.is_lowercase());
            let all_lowercase_or_underscore_or_number = text.chars().all(|c| c.is_lowercase() || c == '_' || c.is_numeric());
            if starts_lowercase && all_lowercase_or_underscore_or_number {
                Some(ValueName {
                    text: text.to_string(),
                    line_number: word.line,
                    column_from: word.column_from,
                    column_to: word.column_to,
                })
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod test_parse_value_name {
    use super::*;
    use crate::split_words;
    #[test]
    fn valid() {
        let words = split_words("hello", vec![]);
        let result = ValueName::parse((&words).into());
        assert_eq!(
            result.0,
            Some(ValueName {
                text: "hello".to_string(),
                line_number: 0,
                column_from: 0,
                column_to: 5,
            })
        );
        assert_eq!(result.1.size(), 0);
        assert_eq!(result.2.len(), 0);
    }
    #[test]
    fn valid_multiple() {
        let words = split_words("hello_world", vec![]);
        let result = ValueName::parse((&words).into());
        assert_eq!(
            result.0,
            Some(ValueName {
                text: "hello_world".to_string(),
                line_number: 0,
                column_from: 0,
                column_to: 11,
            })
        );
        assert_eq!(result.1.size(), 0);
        assert_eq!(result.2.len(), 0);
    }
    #[test]
    fn invalid() {
        let words = split_words("Hello", vec![]);
        let result = ValueName::parse((&words).into());
        assert_eq!(result.0, None);
        assert_eq!(result.1.size(), 1);
        assert_eq!(result.2.len(), 1);
    }
    #[test]
    fn invalid_multiple() {
        let words = split_words("helloWorld", vec![]);
        let result = ValueName::parse((&words).into());
        assert_eq!(result.0, None);
        assert_eq!(result.1.size(), 1);
        assert_eq!(result.2.len(), 1);
    }
}
