mod split_words;
mod vec_window;

pub use split_words::split_words;
pub use vec_window::VecWindow;

use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Word {
    text: String,
    line_number: usize,
    column_number: usize,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Keyword(pub &'static str);

#[derive(Clone, Debug, Eq, PartialEq)]
enum TokenType {
    Keyword(Keyword),
    TypeName,
    ValueName,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    expected: TokenType,
    got: Option<Word>,
}

fn err<A>(expected: TokenType, got: Word) -> Result<A, Vec<ParseError>> {
    Err(vec![ParseError {
        expected,
        got: Some(got),
    }])
}

fn err_none<A>(expected: TokenType) -> Result<A, Vec<ParseError>> {
    Err(vec![ParseError {
        expected,
        got: None,
    }])
}

pub trait Parser<Out>: Clone {
    fn parse<'l>(
        &self,
        words: VecWindow<'l, Word>,
    ) -> Result<(VecWindow<'l, Word>, Out), Vec<ParseError>>;

    // For the following methods `Out` is the output type of the current parsers
    // and the methods themselves create parser with a new output type
    fn keyword(self, keyword: Keyword) -> KeywordParser<Out, Self> {
        KeywordParser {
            prev: self,
            keyword,
            out: PhantomData,
        }
    }
    fn type_name(self) -> TypeParser<Out, Self> {
        TypeParser {
            prev: self,
            out: PhantomData,
        }
    }
    fn value_name(self) -> ValueParser<Out, Self> {
        ValueParser {
            prev: self,
            out: PhantomData,
        }
    }
    fn map<AST>(self, map: fn(Out) -> AST) -> ParserMapper<AST, Out, Self> {
        ParserMapper {
            prev: self,
            map,
            out: PhantomData,
        }
    }
    fn many(self) -> ManyParser<Out, Self> {
        ManyParser {
            prev: self,
            out: PhantomData,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewParser {}

impl Parser<()> for NewParser {
    fn parse<'l>(
        &self,
        words: VecWindow<'l, Word>,
    ) -> Result<(VecWindow<'l, Word>, ()), Vec<ParseError>> {
        Ok((words, ()))
    }
}

pub fn parser() -> NewParser {
    NewParser {}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeywordParser<Out, Prev: Parser<Out>> {
    prev: Prev,
    keyword: Keyword,
    out: PhantomData<Out>,
}

impl<O: Debug + Clone, Prev: Parser<O>> Parser<O> for KeywordParser<O, Prev> {
    fn parse<'l>(
        &self,
        words: VecWindow<'l, Word>,
    ) -> Result<(VecWindow<'l, Word>, O), Vec<ParseError>> {
        let (mut words, prev) = self.prev.parse(words)?;
        let Some(word) = words.pop_front().cloned() else {
            return err_none(TokenType::Keyword(self.keyword.clone()));
        };
        if word.text == self.keyword.0 {
            Ok((words, prev))
        } else {
            err(TokenType::Keyword(self.keyword.clone()), word)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeParser<Out, Prev: Parser<Out>> {
    prev: Prev,
    out: PhantomData<Out>,
}

impl<O: Debug + Clone, Prev: Parser<O>> Parser<(O, Word)> for TypeParser<O, Prev> {
    fn parse<'l>(
        &self,
        words: VecWindow<'l, Word>,
    ) -> Result<(VecWindow<'l, Word>, (O, Word)), Vec<ParseError>> {
        let (mut words, prev) = self.prev.parse(words)?;
        let Some(word) = words.pop_front().cloned() else {
            return err_none(TokenType::TypeName);
        };
        if word.text.starts_with(|c: char| c.is_uppercase())
            && word.text.chars().all(|c| c.is_alphabetic())
        {
            Ok((words, (prev, word)))
        } else {
            err(TokenType::TypeName, word)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueParser<Out, Prev: Parser<Out>> {
    prev: Prev,
    out: PhantomData<Out>,
}

impl<O: Debug + Clone, Prev: Parser<O>> Parser<(O, Word)> for ValueParser<O, Prev> {
    fn parse<'l>(
        &self,
        words: VecWindow<'l, Word>,
    ) -> Result<(VecWindow<'l, Word>, (O, Word)), Vec<ParseError>> {
        let (mut words, prev) = self.prev.parse(words)?;
        let Some(word) = words.pop_front().cloned() else {
            return err_none(TokenType::ValueName);
        };
        if word.text.chars().all(|c| c.is_lowercase() || c == '_') {
            Ok((words, (prev, word)))
        } else {
            err(TokenType::ValueName, word)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParserMapper<AST, Out, Prev: Parser<Out>> {
    prev: Prev,
    map: fn(Out) -> AST,
    out: PhantomData<Out>,
}

impl<AST: Debug + Clone, O: Debug + Clone, Prev: Parser<O>> Parser<AST>
    for ParserMapper<AST, O, Prev>
{
    fn parse<'l>(
        &self,
        words: VecWindow<'l, Word>,
    ) -> Result<(VecWindow<'l, Word>, AST), Vec<ParseError>> {
        let (words, prev) = self.prev.parse(words)?;
        let res = (self.map)(prev);
        Ok((words, res))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManyParser<Out, Prev: Parser<Out>> {
    prev: Prev,
    out: PhantomData<Out>,
}

impl<O: Debug + Clone, Prev: Parser<Option<O>>> Parser<Vec<O>> for ManyParser<Option<O>, Prev> {
    fn parse<'l>(
        &self,
        mut words: VecWindow<'l, Word>,
    ) -> Result<(VecWindow<'l, Word>, Vec<O>), Vec<ParseError>> {
        let mut res = vec![];
        while !words.is_empty() {
            let (words_, prev) = self.prev.parse(words)?;
            words = words_;
            if let Some(prev) = prev {
                res.push(prev);
            } else {
                break;
            }
        }
        Ok((words, res))
    }
}
