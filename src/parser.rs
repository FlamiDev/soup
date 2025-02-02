use parser_lib::{
    separator, CurlyBrackets, NonEmptyVec, Parentheses, Parser, SeparatedBy, SeparatedOnce,
    SquareBrackets, TypeName, ValueName,
};

separator!(Comma = ",");
separator!(Colon = ":");
separator!(Dot = ".");
separator!(Equals = "==");
separator!(NotEquals = "!=");
separator!(LessThan = "<");
separator!(LessThanOrEqual = "<=");
separator!(GreaterThan = ">");
separator!(GreaterThanOrEqual = ">=");
separator!(Plus = "+");
separator!(Minus = "-");
separator!(Multiply = "*");
separator!(Modulo = "%");

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct Program {
    pub items: Vec<AST>,
}

#[derive(Clone, Debug, PartialEq, Parser)]
#[allow(clippy::upper_case_acronyms)]
pub enum AST {
    Import {
        #[text = "import"]
        name: TypeName,
        from: String,
    },
    Type {
        #[text = "type"]
        name: TypeName,
        #[text = "="]
        value: Type,
    },
    DocComment {
        #[text = "doc"]
        comment: String,
    },
    TestBlock {
        #[text = "test"]
        description: String,
        block: Parentheses<Vec<TestItem>>,
    },
    Let {
        #[text = "let"]
        to: MatchItem,
        type_: Option<Type>,
        #[text = "="]
        from: NormalValue,
    },
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum Type {
    Array(SquareBrackets<Box<Type>>),
    Tuple(CurlyBrackets<SeparatedBy<Comma, (ValueName, Type)>>),
    Union(NonEmptyVec<UnionPart>),
    Function {
        #[text = "Fn"]
        params: Vec<Type>,
        #[text = "->"]
        return_type: Box<Type>,
    },
    Group(Parentheses<Vec<Type>>),
    Reference(NonEmptyVec<TypeName>),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct UnionPart {
    #[text = ":"]
    pub name: TypeName,
    pub type_: Option<Type>,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum NormalValue {
    Expression(Box<Expression>),
    MatchItem(MatchItem),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum AlwaysWrappedValue {
    Expression(Parentheses<Expression>),
    MatchItem(MatchItem),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum MatchItem {
    Array(SquareBrackets<SeparatedBy<Comma, MatchItem>>),
    Tuple(CurlyBrackets<SeparatedBy<Comma, (ValueName, MatchItem)>>),
    Label(TypeName, Box<MatchItem>),
    Name(ValueName),
    Value(Value),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum Expression {
    Equals(SeparatedOnce<Equals, NormalValue, NormalValue>),
    NotEquals(SeparatedOnce<NotEquals, NormalValue, NormalValue>),
    LessThan(SeparatedOnce<LessThan, NormalValue, NormalValue>),
    LessThanOrEqual(SeparatedOnce<LessThanOrEqual, NormalValue, NormalValue>),
    GreaterThan(SeparatedOnce<GreaterThan, NormalValue, NormalValue>),
    GreaterThanOrEqual(SeparatedOnce<GreaterThanOrEqual, NormalValue, NormalValue>),
    Plus(SeparatedOnce<Plus, NormalValue, NormalValue>),
    Minus(SeparatedOnce<Minus, NormalValue, NormalValue>),
    Multiply(SeparatedOnce<Multiply, NormalValue, NormalValue>),
    Modulo(SeparatedOnce<Modulo, NormalValue, NormalValue>),
    Negate {
        #[text = "-"]
        value: Box<NormalValue>,
    },
    Block(Parentheses<Block>),
    Function {
        params: Vec<(ValueName, Option<TypeName>)>,
        #[text = "->"]
        body: NormalValue,
    },
    FunctionCalls {
        input_value: Box<AlwaysWrappedValue>,
        function_name: ValueName,
        arguments: Vec<AlwaysWrappedValue>,
        piped_calls: Vec<FunctionCall>,
    },
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct FunctionCall {
    #[text = ","]
    pub name: ValueName,
    pub arguments: Vec<AlwaysWrappedValue>,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum TestItem {
    Mock {
        #[text = "mock"]
        name: ValueName,
        #[text = "="]
        value: NormalValue,
    },
    Assert {
        #[text = "assert"]
        value: NormalValue,
    },
    Let {
        #[text = "let"]
        to: MatchItem,
        type_: Option<TypeName>,
        #[text = "="]
        from: NormalValue,
    },
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct Block {
    pub lets: Vec<BlockLet>,
    #[text = "ret"]
    pub ret: NormalValue,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct BlockLet {
    #[text = "let"]
    pub to: MatchItem,
    pub type_: Option<Type>,
    #[text = "="]
    pub from: NormalValue,
}
