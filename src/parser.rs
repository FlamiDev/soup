use parser_lib::{Parser, Word};

#[derive(Clone, Debug, Eq, PartialEq, Parser)]
pub struct Program {
    pub items: Vec<AST>,
}

#[derive(Clone, Debug, Eq, PartialEq, Parser)]
#[allow(clippy::upper_case_acronyms)]
pub enum AST {
    Import {
        #[text = "import"]
        name: Word,
        from: String,
    },
    Type {
        #[text = "type"]
        name: Word,
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
        type_: Option<Word>,
        #[text = "="]
        from: MatchItem,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Parser)]
pub enum MatchItem {
    Array(Vec<MatchItem>),
    Tuple(Vec<(Option<Word>, MatchItem)>),
    Label(Word, Box<MatchItem>),
    Name(Word),
    Value(Value),
}

#[derive(Clone, Debug, Eq, PartialEq, Parser)]
pub enum Type {}

#[derive(Clone, Debug, Eq, PartialEq, Parser)]
pub enum Value {}

#[derive(Clone, Debug, Eq, PartialEq, Parser)]
pub struct Block {}
