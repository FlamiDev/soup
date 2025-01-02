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

#[derive(Debug)]
pub struct ParseResult<'w, Out>(usize, Option<(VecWindow<'w, Word>, Out)>, Vec<ParseError>);

impl<'w, Out> ParseResult<'w, Out> {
    fn map<Out2, F: FnOnce(VecWindow<'w, Word>, Out) -> ParseResult<'w, Out2>>(
        self,
        f: F,
    ) -> ParseResult<'w, Out2> {
        let ParseResult(index, out, errs) = self;
        match out {
            Some((words, out)) => {
                let ParseResult(index2, out, errs2) = f(words, out);
                ParseResult(index + index2, out, [errs, errs2].concat())
            }
            None => ParseResult(index, None, errs),
        }
    }
}

fn ok<'l, Out>(words: VecWindow<'l, Word>, index: usize, out: Out) -> ParseResult<'l, Out> {
    ParseResult(index, Some((words, out)), Vec::new())
}

fn err<'l, Out>(index: usize, expected: TokenType, got: Word) -> ParseResult<'l, Out> {
    ParseResult(
        index,
        None,
        vec![ParseError {
            expected,
            got: Some(got),
        }],
    )
}

fn err_end<'l, Out>(index: usize, expected: TokenType) -> ParseResult<'l, Out> {
    ParseResult(
        index,
        None,
        vec![ParseError {
            expected,
            got: None,
        }],
    )
}

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
    fn separated(self, sep: Keyword) -> SeparatedParser<Out, Self> {
        SeparatedParser {
            prev: self,
            sep,
            out: PhantomData,
        }
    }
    fn split_start(self, split_on: Vec<Keyword>) -> SplitStartParser<Out, Self> {
        SplitStartParser {
            part: self,
            split_on,
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
        ok(words, 0, ())
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
        self.prev.parse(words).map(|mut words, prev| {
            let Some(word) = words.pop_first().cloned() else {
                return err_end(1, TokenType::Keyword(self.keyword.clone()));
            };
            if word.text == self.keyword.0 {
                ok(words, 1, prev)
            } else {
                err(1, TokenType::Keyword(self.keyword.clone()), word)
            }
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TypeParser<Out, Prev: Parser<Out>> {
    prev: Prev,
    out: PhantomData<Out>,
}

impl<Out: Debug, Prev: Parser<Out>> DynSafeParser<(Out, Word)> for TypeParser<Out, Prev> {
    fn parse<'w>(&self, words: VecWindow<'w, Word>) -> ParseResult<'w, (Out, Word)> {
        self.prev.parse(words).map(|mut words, prev| {
            let Some(word) = words.pop_first().cloned() else {
                return err_end(1, TokenType::TypeName);
            };
            if word.text.starts_with(|c: char| c.is_uppercase())
                && word.text.chars().all(|c| c.is_alphabetic())
            {
                ok(words, 1, (prev, word))
            } else {
                err(1, TokenType::TypeName, word)
            }
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ValueParser<Out, Prev: Parser<Out>> {
    prev: Prev,
    out: PhantomData<Out>,
}

impl<Out: Debug, Prev: Parser<Out>> DynSafeParser<(Out, Word)> for ValueParser<Out, Prev> {
    fn parse<'w>(&self, words: VecWindow<'w, Word>) -> ParseResult<'w, (Out, Word)> {
        self.prev.parse(words).map(|mut words, prev| {
            let Some(word) = words.pop_first().cloned() else {
                return err_end(1, TokenType::ValueName);
            };
            if word.text.chars().all(|c| c.is_lowercase() || c == '_') {
                ok(words, 1, (prev, word))
            } else {
                err(1, TokenType::ValueName, word)
            }
        })
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
        self.prev.parse(words).map(|mut words, prev| {
            let Some(word) = words.pop_first().cloned() else {
                return err_end(1, TokenType::Int);
            };
            let Ok(int) = word.text.parse::<i64>() else {
                return err(1, TokenType::Int, word);
            };
            let int = IntResult {
                value: int,
                line_number: word.line_number,
                column_number: word.column_number,
            };
            ok(words, 1, (prev, int))
        })
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
        self.prev.parse(words).map(|mut words, prev| {
            let Some(word) = words.pop_first().cloned() else {
                return err_end(1, TokenType::Float);
            };
            let Ok(float) = word.text.parse::<f64>() else {
                return err(1, TokenType::Float, word);
            };
            let float = FloatResult {
                value: float,
                line_number: word.line_number,
                column_number: word.column_number,
            };
            ok(words, 1, (prev, float))
        })
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
        self.prev.parse(words).map(|mut words, prev| {
            let Some(word) = words.pop_first().cloned() else {
                return err_end(1, TokenType::String);
            };
            if word.text.starts_with('"') && word.text.ends_with('"') {
                let string = StringResult {
                    value: word.text[1..word.text.len() - 1].to_string(),
                    line_number: word.line_number,
                    column_number: word.column_number,
                };
                ok(words, 1, (prev, string))
            } else {
                err(1, TokenType::String, word)
            }
        })
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
        self.inner.parse(inner_words).map(|remaining_words, inner| {
            if let Some(word) = remaining_words.first().cloned() {
                return err(1, TokenType::Bracket(self.close.clone()), word);
            }
            ok(words, 1, inner)
        })
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
        self.prev.parse(words).map(|words, prev| {
            let res = (self.map)(prev);
            ok(words, 1, res)
        })
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
        let ParseResult(index1, out1, errs1) = self.prev1.parse(words.clone());
        match out1 {
            Some((words, out1)) => ok(words, index1, out1),
            None if index1 <= 1 => {
                let ParseResult(index2, out2, errs2) = self.prev2.parse(words);
                match out2 {
                    Some((words, out2)) => ok(words, index2, out2),
                    None if index2 <= 1 => ParseResult(index2, None, [errs1, errs2].concat()),
                    None => ParseResult(index2, None, errs2),
                }
            }
            None => ParseResult(index1, None, errs1),
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
        self.prev1.parse(words).map(|words, prev1| {
            self.prev2
                .parse(words)
                .map(|words, prev2| ok(words, 1, (prev1, prev2)))
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SeparatedParser<Out, Prev: Parser<Out>> {
    prev: Prev,
    sep: Keyword,
    out: PhantomData<Out>,
}

impl<Out: Debug, Prev: Parser<Out>> DynSafeParser<Vec<Out>> for SeparatedParser<Out, Prev> {
    fn parse<'w>(&self, mut words: VecWindow<'w, Word>) -> ParseResult<'w, Vec<Out>> {
        let parts = words.split(|word| word.text == self.sep.0);
        let mut last_index = 0;
        let mut res = Vec::new();
        let mut errs = Vec::new();
        for part in parts {
            let ParseResult(index, out, errs2) = self.prev.parse(part);
            last_index = index;
            if let Some(out) = out {
                res.push(out.1);
            };
            errs.extend(errs2);
        }
        if last_index == 0 {
            err_end(0, TokenType::ValueName)
        } else {
            words.shrink_start_to(words.end());
            ParseResult(last_index + 1, Some((words, res)), errs)
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SplitStartParser<Out, Part: Parser<Out>> {
    part: Part,
    split_on: Vec<Keyword>,
    out: PhantomData<Out>,
}

impl<Out, Part: Parser<Out>> DynSafeParser<Vec<Out>> for SplitStartParser<Out, Part> {
    fn parse<'w>(&self, mut words: VecWindow<'w, Word>) -> ParseResult<'w, Vec<Out>> {
        let mut last_index = 0;
        let mut res = Vec::new();
        let mut errs = Vec::new();
        loop {
            let Some(found) = words
                .skip(1)
                .find(|word| self.split_on.iter().any(|k| k.0 == word.text))
            else {
                break;
            };
            let Some((current, new_words)) = words.snip(found + 1) else {
                return err_end(0, TokenType::ValueName);
            };
            words = new_words;
            let ParseResult(index, out, errs2) = self.part.parse(current);
            last_index = index;
            if let Some(out) = out {
                res.push(out.1);
            };
            errs.extend(errs2);
        }
        if last_index == 0 {
            err_end(0, TokenType::ValueName)
        } else {
            words.shrink_start_to(words.end());
            ParseResult(last_index + 1, Some((words, res)), errs)
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct OptionalParser<Out, Prev: Parser<Out>> {
    prev: Prev,
    out: PhantomData<Out>,
}

impl<Out: Debug, Prev: Parser<Out>> DynSafeParser<Option<Out>> for OptionalParser<Out, Prev> {
    fn parse<'w>(&self, words: VecWindow<'w, Word>) -> ParseResult<'w, Option<Out>> {
        let ParseResult(index, out, errs) = self.prev.parse(words.clone());
        match out {
            Some((words, out)) => ParseResult(index, Some((words, Some(out))), errs),
            None if index <= 1 => ok(words, 0, None),
            None => ParseResult(index, None, errs),
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
        let ParseResult(index, out, errs) = self.base.parse(words.clone());
        match out {
            Some(out) => ParseResult(index, Some(out), errs),
            None if index <= 1 => self.recursive.parse(words),
            None => ParseResult(index, None, errs),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TodoParser {
    message: &'static str,
}

pub fn todo(message: &'static str) -> TodoParser {
    TodoParser { message }
}

impl<Out: Debug> DynSafeParser<Out> for TodoParser {
    fn parse<'w>(&self, _words: VecWindow<'w, Word>) -> ParseResult<'w, Out> {
        todo!("Unfinished parser branch: {}", self.message)
    }
}
