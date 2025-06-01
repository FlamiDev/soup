use parser_lib::{
    separator, CurlyBrackets, NonEmptyStartTextVec, NonEmptyVec, Parentheses, Parser, SeparatedBy,
    SeparatedOnce, SquareBrackets, StartTextVec, TypeName, ValueName,
};

separator!(Comma = ",");
separator!(Colon = ":");
separator!(Semicolon = ";");
separator!(ArrowRight = "->");

#[derive(Clone, Debug, PartialEq, Parser)]
#[allow(clippy::upper_case_acronyms)]
pub struct AST {
    pub items: StartTextVec<Declaration>,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum Declaration {
    Use {
        #[text = "use"]
        imports: CurlyBrackets<SeparatedBy<Semicolon, Import>>,
        from: String,
    },
    Doc {
        #[text = "doc"]
        comment: String,
    },
    Type {
        #[text = "typ"]
        is_pub: Option<PubToken>,
        name: TypeName,
        type_args: Vec<TypeName>,
        dependencies: Option<CurlyBrackets<SeparatedBy<Semicolon, TypedValue>>>,
        #[text = "="]
        value: GreedyType,
    },
    Has {
        #[text = "has"]
        name: TypeName,
        type_args: Vec<TypeName>,
        #[text = "="]
        value: NonEmptyVec<HasRequirement>,
    },
    Def {
        #[text = "def"]
        is_pub: Option<PubToken>,
        name: ValueName,
        type_args: Vec<TypeName>,
        #[text = "="]
        value: GreedyTypeRef,
    },
    Let {
        #[text = "let"]
        name: ValueName,
        #[text = "="]
        be: Box<GreedyValue>,
    },
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum Import {
    Value(ValueName),
    Type(TypeName),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct PubToken {
    #[text = "pub"]
    _nothing: (),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum NonGreedyTypeRef {
    InParens(Parentheses<Box<GreedyTypeRef>>),
    Name(TypeName),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum GreedyTypeRef {
    Function(SeparatedOnce<ArrowRight, Box<GreedyTypeRef>, Box<GreedyTypeRef>>),
    Dependencies {
        type_: TypeName,
        args: Vec<NonGreedyTypeRef>,
        dependencies: CurlyBrackets<SeparatedBy<Semicolon, (ValueName, GreedyValue)>>,
    },
    Args {
        type_: TypeName,
        args: Vec<NonGreedyTypeRef>,
    },
    NonGreedy(NonGreedyTypeRef),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct TypedValue {
    name: ValueName,
    type_: GreedyTypeRef,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct HasRequirement {
    name: ValueName,
    #[text = "=>"]
    type_: GreedyTypeRef,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct UnionOption {
    #[text = "|"]
    name: TypeName,
    value: Option<GreedyTypeRef>,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum GreedyType {
    Union(NonEmptyStartTextVec<UnionOption>),
    Tuple(CurlyBrackets<SeparatedBy<Semicolon, GreedyTypeRef>>),
    Match {
        on: TypeOrValue,
        #[text = ":"]
        matchers: NonEmptyStartTextVec<Matcher<TypeOrValue, GreedyType>>,
    },
    Ref(GreedyTypeRef),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum TypeOrValue {
    Type(GreedyTypeRef),
    Value(NonGreedyValue),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct Matcher<MatchValue, Value>
where
    MatchValue: Parser<MatchValue>,
    Value: Parser<Value>,
{
    #[text = "|"]
    on: MatchItem<MatchValue>,
    #[text = "->"]
    value: Value,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum MatchItem<Value>
where
    Value: Parser<Value>,
{
    Union {
        name: TypeName,
        value: Option<Box<MatchItem<Value>>>,
    },
    Tuple(CurlyBrackets<SeparatedBy<Semicolon, MatchItem<Value>>>),
    Value(Value),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum NonGreedyValue {
    InParens(Parentheses<Box<GreedyValue>>),
    List(SquareBrackets<SeparatedBy<Semicolon, GreedyValue>>),
    Tuple(CurlyBrackets<SeparatedBy<Semicolon, GreedyValue>>),
    Boolean(bool),
    Int(i64),
    Float(f64),
    String(String),
    Ref(ValueName),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum GreedyValue {
    Function {
        args: NonEmptyVec<ValueName>,
        #[text = "->"]
        returns: Box<GreedyValue>,
        with: StartTextVec<FunctionWithBlock>,
    },
    CallSequence {
        start: CallsStart,
        continue_calls: Vec<CallsContinue>,
        maybe_match: Option<MatchValue>,
    },
    SimpleMatch {
        on: NonGreedyValue,
        match_: MatchValue,
    },
    NonGreedy(NonGreedyValue),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct FunctionWithBlock {
    #[text = "<-"]
    name: ValueName,
    #[text = "="]
    block: Box<GreedyValue>,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct CallsStart {
    initial: NonGreedyValue,
    function: ValueName,
    args: Vec<NonGreedyValue>,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct CallsContinue {
    #[text = ","]
    function: ValueName,
    args: Vec<NonGreedyValue>,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct MatchValue {
    #[text = ":"]
    matchers: StartTextVec<Matcher<ValueName, GreedyValue>>,
}
