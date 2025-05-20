use parser_lib::{
    separator, CurlyBrackets, NonEmptyVec, Parentheses, Parser, SeparatedBy, SeparatedOnce,
    SquareBrackets, StatementVec, TypeName, ValueName,
};

separator!(Comma = ",");
separator!(Colon = ":");
separator!(Semicolon = ";");
separator!(ArrowRight = "->");

#[derive(Clone, Debug, PartialEq, Parser)]
#[allow(clippy::upper_case_acronyms)]
pub struct AST {
    pub items: StatementVec<Declaration>,
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
        value: Type,
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
        value: TypeRef,
    },
    Let {
        #[text = "let"]
        name: ValueName,
        #[text = "="]
        be: Box<Value>,
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
pub enum TypeRef {
    Function(SeparatedOnce<ArrowRight, Box<TypeRef>, Box<TypeRef>>),
    InParens(Parentheses<Box<TypeRef>>),
    WithDependencies {
        type_: TypeName,
        args: Vec<TypeRef>,
        dependencies: CurlyBrackets<SeparatedBy<Semicolon, (ValueName, Value)>>,
    },
    Raw {
        type_: TypeName,
        args: Vec<TypeRef>,
    },
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct TypedValue {
    name: ValueName,
    type_: TypeRef,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct HasRequirement {
    name: ValueName,
    #[text = "=>"]
    type_: TypeRef,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct UnionOption {
    #[text = "|"]
    name: TypeName,
    value: Option<TypeRef>,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum Type {
    Union(NonEmptyVec<UnionOption>),
    Tuple(CurlyBrackets<SeparatedBy<Semicolon, TypeRef>>),
    Match {
        on: TypeOrValue,
        #[text = ":"]
        matchers: NonEmptyVec<Matcher<TypeOrValue, Type>>,
    },
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum TypeOrValue {
    Type(TypeRef),
    Value(ValueName),
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
pub enum Value {
    InParens(Parentheses<Box<Value>>),
    List(SquareBrackets<SeparatedBy<Semicolon, Value>>),
    MatchSequence(SeparatedOnce<Colon, Box<Value>, NonEmptyVec<Matcher<ValueName, Value>>>),
    CallSequence(SeparatedOnce<Comma, CallsStart, SeparatedBy<Comma, CallsContinue>>),
    Boolean(bool),
    Int(i64),
    Float(f64),
    String(String),
    Function {
        args: NonEmptyVec<ValueName>,
        #[text = "->"]
        returns: Box<Value>,
        with: Vec<FunctionWithBlock>,
    },
    Ref(ValueName),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct FunctionWithBlock {
    #[text = "<-"]
    name: ValueName,
    #[text = "="]
    block: Box<Value>,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct CallsStart {
    initial: Box<Value>,
    function: ValueName,
    args: Vec<Value>,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct CallsContinue {
    function: ValueName,
    args: Vec<Value>,
}
