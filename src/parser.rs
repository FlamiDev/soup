use parser_lib::{
    CommaSeparated, CurlyBrackets, Parser, SquareBrackets, TypeName, ValueName,
};

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
        block: Block,
    },
    Let {
        #[text = "let"]
        to: MatchItem,
        type_: Option<TypeName>,
        #[text = "="]
        from: MatchItem,
    },
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum MatchItem {
    Array(SquareBrackets<CommaSeparated<MatchItem>>),
    Tuple(CurlyBrackets<CommaSeparated<(ValueName, MatchItem)>>),
    Label(TypeName, Box<MatchItem>),
    Name(ValueName),
    Value(Value),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum Type {
    Array(SquareBrackets<Box<Type>>),
    Tuple(CurlyBrackets<CommaSeparated<(ValueName, Type)>>),
    //Union(),
    Reference(Vec<TypeName>),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct Block {}
