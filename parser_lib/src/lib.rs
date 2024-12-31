mod split_words;
mod vec_window;

pub use split_words::split_words;
use std::cell::OnceCell;
pub use vec_window::VecWindow;

use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Word {
    pub text: String,
    pub line_number: usize,
    pub column_number: usize,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Keyword(pub &'static str);

#[derive(Clone, Debug, Eq, PartialEq)]
enum TokenType {
    Keyword(Keyword),
    TypeName,
    ValueName,
    Int,
    Float,
    String,
    Bracket(Keyword),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    expected: TokenType,
    got: Option<Word>,
}

fn err<'l, Out>(index: usize, expected: TokenType, got: Word) -> ParseResult<'l, Out> {
    Err((
        index,
        vec![ParseError {
            expected,
            got: Some(got),
        }],
    ))
}

fn err_end<'l, Out>(index: usize, expected: TokenType) -> ParseResult<'l, Out> {
    Err((
        index,
        vec![ParseError {
            expected,
            got: None,
        }],
    ))
}

pub type ParseResult<'w, Out> = Result<(VecWindow<'w, Word>, usize, Out), (usize, Vec<ParseError>)>;

pub trait DynSafeParser<Out> {
    fn parse<'w>(&self, words: VecWindow<'w, Word>) -> ParseResult<'w, Out>;
}

pub trait Parser<Out>: DynSafeParser<Out> + Sized {
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
    fn int(self) -> IntParser<Out, Self> {
        IntParser {
            prev: self,
            out: PhantomData,
        }
    }
    fn float(self) -> FloatParser<Out, Self> {
        FloatParser {
            prev: self,
            out: PhantomData,
        }
    }
    fn string(self) -> StringParser<Out, Self> {
        StringParser {
            prev: self,
            out: PhantomData,
        }
    }
    fn in_brackets(self, open: Keyword, close: Keyword) -> BracketParser<Out, Self> {
        BracketParser {
            inner: self,
            open,
            close,
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
    fn or<Prev2: Parser<Out>>(self, prev2: Prev2) -> OrParser<Out, Self, Prev2> {
        OrParser {
            prev1: self,
            prev2,
            out: PhantomData,
        }
    }
    fn and<Out2, Prev2: Parser<Out2>>(self, prev2: Prev2) -> AndParser<Out, Out2, Self, Prev2> {
        AndParser {
            prev1: self,
            prev2,
            out: PhantomData,
        }
    }
    fn repeat(self) -> RepeatParser<Out, Self> {
        RepeatParser {
            prev: self,
            out: PhantomData,
        }
    }
    fn optional(self) -> OptionalParser<Out, Self> {
        OptionalParser {
            prev: self,
            out: PhantomData,
        }
    }
}

impl<Out: Debug, DSP: DynSafeParser<Out>> Parser<Out> for DSP {}

#[derive(Debug, Eq, PartialEq)]
pub struct NewParser {
    name: &'static str,
}

impl DynSafeParser<()> for NewParser {
    fn parse<'w>(&self, words: VecWindow<'w, Word>) -> ParseResult<'w, ()> {
        Ok((words, 0, ()))
    }
}

pub fn new_parser(name: &'static str) -> NewParser {
    NewParser { name }
}

#[derive(Debug, Eq, PartialEq)]
pub struct KeywordParser<Out, Prev: Parser<Out>> {
    prev: Prev,
    keyword: Keyword,
    out: PhantomData<Out>,
}

impl<Out: Debug, Prev: Parser<Out>> DynSafeParser<Out> for KeywordParser<Out, Prev> {
    fn parse<'w>(&self, words: VecWindow<'w, Word>) -> ParseResult<'w, Out> {
        let (mut words, index, prev) = self.prev.parse(words)?;
        let Some(word) = words.pop_first().cloned() else {
            return err_end(index + 1, TokenType::Keyword(self.keyword.clone()));
        };
        if word.text == self.keyword.0 {
            Ok((words, index + 1, prev))
        } else {
            err(index + 1, TokenType::Keyword(self.keyword.clone()), word)
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TypeParser<Out, Prev: Parser<Out>> {
    prev: Prev,
    out: PhantomData<Out>,
}

impl<Out: Debug, Prev: Parser<Out>> DynSafeParser<(Out, Word)> for TypeParser<Out, Prev> {
    fn parse<'w>(&self, words: VecWindow<'w, Word>) -> ParseResult<'w, (Out, Word)> {
        let (mut words, index, prev) = self.prev.parse(words)?;
        let Some(word) = words.pop_first().cloned() else {
            return err_end(index + 1, TokenType::TypeName);
        };
        if word.text.starts_with(|c: char| c.is_uppercase())
            && word.text.chars().all(|c| c.is_alphabetic())
        {
            Ok((words, index + 1, (prev, word)))
        } else {
            err(index + 1, TokenType::TypeName, word)
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ValueParser<Out, Prev: Parser<Out>> {
    prev: Prev,
    out: PhantomData<Out>,
}

impl<Out: Debug, Prev: Parser<Out>> DynSafeParser<(Out, Word)> for ValueParser<Out, Prev> {
    fn parse<'w>(&self, words: VecWindow<'w, Word>) -> ParseResult<'w, (Out, Word)> {
        let (mut words, index, prev) = self.prev.parse(words)?;
        let Some(word) = words.pop_first().cloned() else {
            return err_end(index + 1, TokenType::ValueName);
        };
        if word.text.chars().all(|c| c.is_lowercase() || c == '_') {
            Ok((words, index + 1, (prev, word)))
        } else {
            err(index + 1, TokenType::ValueName, word)
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct IntParser<Out, Prev: Parser<Out>> {
    prev: Prev,
    out: PhantomData<Out>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntResult {
    pub value: i64,
    pub line_number: usize,
    pub column_number: usize,
}

impl<Out: Debug, Prev: Parser<Out>> DynSafeParser<(Out, IntResult)> for IntParser<Out, Prev> {
    fn parse<'w>(&self, words: VecWindow<'w, Word>) -> ParseResult<'w, (Out, IntResult)> {
        let (mut words, index, prev) = self.prev.parse(words)?;
        let Some(word) = words.pop_first().cloned() else {
            return err_end(index + 1, TokenType::Int);
        };
        let Ok(int) = word.text.parse::<i64>() else {
            return err(index + 1, TokenType::Int, word);
        };
        let int = IntResult {
            value: int,
            line_number: word.line_number,
            column_number: word.column_number,
        };
        Ok((words, index + 1, (prev, int)))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct FloatParser<Out, Prev: Parser<Out>> {
    prev: Prev,
    out: PhantomData<Out>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FloatResult {
    pub value: f64,
    pub line_number: usize,
    pub column_number: usize,
}

impl<Out: Debug, Prev: Parser<Out>> DynSafeParser<(Out, FloatResult)> for FloatParser<Out, Prev> {
    fn parse<'w>(&self, words: VecWindow<'w, Word>) -> ParseResult<'w, (Out, FloatResult)> {
        let (mut words, index, prev) = self.prev.parse(words)?;
        let Some(word) = words.pop_first().cloned() else {
            return err_end(index + 1, TokenType::Float);
        };
        let Ok(float) = word.text.parse::<f64>() else {
            return err(index + 1, TokenType::Float, word);
        };
        let float = FloatResult {
            value: float,
            line_number: word.line_number,
            column_number: word.column_number,
        };
        Ok((words, index + 1, (prev, float)))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct StringParser<Out, Prev: Parser<Out>> {
    prev: Prev,
    out: PhantomData<Out>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StringResult {
    pub value: String,
    pub line_number: usize,
    pub column_number: usize,
}

impl<Out: Debug, Prev: Parser<Out>> DynSafeParser<(Out, StringResult)> for StringParser<Out, Prev> {
    fn parse<'w>(&self, words: VecWindow<'w, Word>) -> ParseResult<'w, (Out, StringResult)> {
        let (mut words, index, prev) = self.prev.parse(words)?;
        let Some(word) = words.pop_first().cloned() else {
            return err_end(index + 1, TokenType::String);
        };
        if word.text.starts_with('"') && word.text.ends_with('"') {
            let string = StringResult {
                value: word.text[1..word.text.len() - 1].to_string(),
                line_number: word.line_number,
                column_number: word.column_number,
            };
            Ok((words, index + 1, (prev, string)))
        } else {
            err(index + 1, TokenType::String, word)
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct BracketParser<Out, Inner: Parser<Out>> {
    inner: Inner,
    open: Keyword,
    close: Keyword,
    out: PhantomData<Out>,
}

impl<Out: Debug, Inner: Parser<Out>> DynSafeParser<Out> for BracketParser<Out, Inner> {
    fn parse<'w>(&self, mut words: VecWindow<'w, Word>) -> ParseResult<'w, Out> {
        let Some(word) = words.pop_first().cloned() else {
            return err_end(0, TokenType::ValueName);
        };
        if word.text != self.open.0 {
            return err(0, TokenType::Bracket(self.open.clone()), word);
        }
        let mut inner_brackets = 0;
        let mut inner_words = words.clone();
        loop {
            let Some(word) = words.pop_first().cloned() else {
                return err_end(0, TokenType::Bracket(self.close.clone()));
            };
            if word.text == self.open.0 {
                inner_brackets += 1;
            } else if word.text == self.close.0 {
                if inner_brackets == 0 {
                    break;
                } else {
                    inner_brackets -= 1;
                }
            }
        }
        inner_words.shrink_end_to(words.start() - 2); // start is now after the closing bracket
        let (remaining_words, index, inner) = self.inner.parse(inner_words)?;
        if let Some(word) = remaining_words.first().cloned() {
            return err(index + 1, TokenType::Bracket(self.close.clone()), word);
        }
        Ok((words, index + 1, inner))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ParserMapper<AST, Out, Prev: Parser<Out>> {
    prev: Prev,
    map: fn(Out) -> AST,
    out: PhantomData<Out>,
}

impl<AST: Debug, O: Debug, Prev: Parser<O>> DynSafeParser<AST> for ParserMapper<AST, O, Prev> {
    fn parse<'w>(&self, words: VecWindow<'w, Word>) -> ParseResult<'w, AST> {
        let (words, index, prev) = self.prev.parse(words)?;
        let res = (self.map)(prev);
        Ok((words, index + 1, res))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrParser<Out, Prev1: Parser<Out>, Prev2: Parser<Out>> {
    prev1: Prev1,
    prev2: Prev2,
    out: PhantomData<Out>,
}

impl<Out: Debug, Prev1: Parser<Out>, Prev2: Parser<Out>> DynSafeParser<Out>
    for OrParser<Out, Prev1, Prev2>
{
    fn parse<'w>(&self, words: VecWindow<'w, Word>) -> ParseResult<'w, Out> {
        match self.prev1.parse(words.clone()) {
            Ok(res) => Ok(res),
            Err((index, errs)) if index <= 1 => match self.prev2.parse(words) {
                Ok(res) => Ok(res),
                Err((index, errs2)) if index <= 1 => Err((index, [errs, errs2].concat())),
                Err((index, errs)) => Err((index, errs)),
            },
            Err((index, errs)) => Err((index, errs)),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct AndParser<Out1, Out2, Prev1: Parser<Out1>, Prev2: Parser<Out2>> {
    prev1: Prev1,
    prev2: Prev2,
    out: PhantomData<(Out1, Out2)>,
}

impl<Out1: Debug, Out2: Debug, Prev1: Parser<Out1>, Prev2: Parser<Out2>> DynSafeParser<(Out1, Out2)>
    for AndParser<Out1, Out2, Prev1, Prev2>
{
    fn parse<'w>(&self, words: VecWindow<'w, Word>) -> ParseResult<'w, (Out1, Out2)> {
        let (words, index1, prev1) = self.prev1.parse(words)?;
        let (words, index2, prev2) = self.prev2.parse(words)?;
        Ok((words, index1 + index2, (prev1, prev2)))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct RepeatParser<Out, Prev: Parser<Out>> {
    prev: Prev,
    out: PhantomData<Out>,
}

impl<Out: Debug, Prev: Parser<Out>> DynSafeParser<Vec<Out>> for RepeatParser<Out, Prev> {
    fn parse<'w>(&self, mut words: VecWindow<'w, Word>) -> ParseResult<'w, Vec<Out>> {
        let initial_length = words.size();
        let mut res = Vec::new();
        let mut index = 0;
        loop {
            match self.prev.parse(words.clone()) {
                Ok((new_words, i, out)) => {
                    res.push(out);
                    index += i;
                    words = new_words;
                }
                Err((i, errs)) => {
                    if i <= 1 && words.size() != initial_length {
                        break;
                    }
                    return Err((index + i, errs));
                }
            }
        }
        Ok((words, index, res))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct OptionalParser<Out, Prev: Parser<Out>> {
    prev: Prev,
    out: PhantomData<Out>,
}

impl<Out: Debug, Prev: Parser<Out>> DynSafeParser<Option<Out>> for OptionalParser<Out, Prev> {
    fn parse<'w>(&self, words: VecWindow<'w, Word>) -> ParseResult<'w, Option<Out>> {
        match self.prev.parse(words.clone()) {
            Ok((words, index, out)) => Ok((words, index, Some(out))),
            Err((index, errs)) => {
                if index <= 1 {
                    Ok((words, index, None))
                } else {
                    Err((index, errs))
                }
            }
        }
    }
}

pub struct RecursiveParser<Out>(Rc<OnceCell<Box<dyn DynSafeParser<Out>>>>);

#[derive(Debug, Eq, PartialEq)]
pub struct RecursiveParserInner<Out, Base: Parser<Out>, Recursive: Parser<Out>> {
    base: Base,
    recursive: Recursive,
    out: PhantomData<Out>,
}

pub fn recursive<
    Out: Debug + 'static,
    Base: Parser<Out> + 'static,
    Recursive: Parser<Out> + 'static,
>(
    base: Base,
    recursive: fn(&dyn Fn() -> RecursiveParser<Out>) -> Recursive,
) -> RecursiveParser<Out> {
    let parser = RecursiveParser(Rc::new(OnceCell::new()));
    parser
        .0
        .set(Box::new(RecursiveParserInner {
            base,
            recursive: recursive(&|| parser.clone()),
            out: PhantomData,
        }))
        .unwrap_or(());
    parser
}

impl<Out> Clone for RecursiveParser<Out> {
    fn clone(&self) -> Self {
        RecursiveParser(self.0.clone())
    }
}

impl<Out> DynSafeParser<Out> for RecursiveParser<Out> {
    fn parse<'w>(&self, words: VecWindow<'w, Word>) -> ParseResult<'w, Out> {
        self.0.get().unwrap().parse(words)
    }
}

impl<Out: Debug, Base: Parser<Out>, Recursive: Parser<Out>> DynSafeParser<Out>
    for RecursiveParserInner<Out, Base, Recursive>
{
    fn parse<'w>(&self, words: VecWindow<'w, Word>) -> ParseResult<'w, Out> {
        match self.base.parse(words.clone()) {
            Ok(res) => Ok(res),
            Err((index, errs)) => {
                if index <= 1 {
                    self.recursive.parse(words)
                } else {
                    Err((index, errs))
                }
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TodoParser {}

pub fn todo() -> TodoParser {
    TodoParser {}
}

impl<Out: Debug> DynSafeParser<Out> for TodoParser {
    fn parse<'w>(&self, _words: VecWindow<'w, Word>) -> ParseResult<'w, Out> {
        todo!()
    }
}
