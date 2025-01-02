use parser_lib::{new_parser, recursive, todo, Keyword, Parser, StringResult, Word};

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum AST {
    Import {
        name: Word,
        from: StringResult,
    },
    Type {
        name: Word,
        value: Type,
    },
    DocComment {
        comment: StringResult,
    },
    TestBlock {
        description: StringResult,
        block: Block,
    },
    Let {
        to: MatchItem,
        type_: Option<Word>,
        from: MatchItem,
    },
}

const IMPORT_KEYWORD: Keyword = Keyword("import");
const TYPE_KEYWORD: Keyword = Keyword("type");
const DOC_COMMENT_KEYWORD: Keyword = Keyword("doc");
const TEST_BLOCK_KEYWORD: Keyword = Keyword("test");
const LET_KEYWORD: Keyword = Keyword("let");
const ARRAY_OPEN: Keyword = Keyword("[");
const ARRAY_CLOSE: Keyword = Keyword("]");
const TUPLE_OPEN: Keyword = Keyword("{");
const TUPLE_CLOSE: Keyword = Keyword("}");
const SCOPE_OPEN: Keyword = Keyword("(");
const SCOPE_CLOSE: Keyword = Keyword(")");
const EQUALS: Keyword = Keyword("=");
const COMMA: Keyword = Keyword(",");

pub fn parser() -> impl Parser<Vec<AST>> {
    new_parser("ast_import")
        .keyword(IMPORT_KEYWORD)
        .type_name()
        .string()
        .map(|((_, name), from)| AST::Import { name, from })
        .or(new_parser("ast_type")
            .keyword(TYPE_KEYWORD)
            .type_name()
            .keyword(EQUALS)
            .and(type_parser())
            .map(|((_, name), value)| AST::Type { name, value }))
        .or(new_parser("ast_doc")
            .keyword(DOC_COMMENT_KEYWORD)
            .string()
            .map(|(_, comment)| AST::DocComment { comment }))
        .or(new_parser("ast_test")
            .keyword(TEST_BLOCK_KEYWORD)
            .string()
            .and(block_parser())
            .map(|((_, description), block)| AST::TestBlock { description, block }))
        .or(new_parser("ast_let")
            .keyword(LET_KEYWORD)
            .and(match_item_parser())
            .and(
                new_parser("ast_let_maybe_type")
                    .type_name()
                    .optional()
                    .map(|a| a.map(|(_, t)| t)),
            )
            .keyword(EQUALS)
            .and(match_item_parser())
            .map(|(((_, to), type_), from)| AST::Let { to, type_, from }))
        .split_start(vec![
            IMPORT_KEYWORD,
            TYPE_KEYWORD,
            DOC_COMMENT_KEYWORD,
            TEST_BLOCK_KEYWORD,
            LET_KEYWORD,
        ])
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MatchItem {
    Array(Vec<MatchItem>),
    Tuple(Vec<(Option<Word>, MatchItem)>),
    Label(Word, Box<MatchItem>),
    Name(Word),
    Value(Value),
}

fn match_item_parser() -> impl Parser<MatchItem> {
    recursive(
        new_parser("match_name")
            .value_name()
            .map(|(_, name)| MatchItem::Name(name))
            .or(value_parser().map(MatchItem::Value)),
        |this| {
            this()
                .separated(COMMA)
                .in_brackets(ARRAY_OPEN, ARRAY_CLOSE)
                .map(MatchItem::Array)
                .or(new_parser("match_tuple")
                    .value_name()
                    .optional()
                    .and(this())
                    .separated(COMMA)
                    .in_brackets(TUPLE_OPEN, TUPLE_CLOSE)
                    .map(|items| {
                        MatchItem::Tuple(
                            items
                                .into_iter()
                                .map(|(a, b)| (a.map(|(_, w)| w), b))
                                .collect(),
                        )
                    }))
                .or(new_parser("match_label")
                    .type_name()
                    .and(this())
                    .map(|((_, name), item)| MatchItem::Label(name, Box::new(item))))
        },
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {}

fn type_parser() -> impl Parser<Type> {
    todo("Type")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {}

fn value_parser() -> impl Parser<Value> {
    todo("Value")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block {}

fn block_parser() -> impl Parser<Block> {
    todo("Block")
}
