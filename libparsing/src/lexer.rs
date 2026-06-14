use std::cmp::PartialEq;
use std::collections::HashMap;
use std::panic::panic_any;

#[derive(Debug)]
pub struct Lexeme<'l, Token> {
    token: Token,
    line: (usize, usize),
    column: (usize, usize),
    source: &'l str,
}

#[derive(PartialEq, Copy, Clone)]
enum LexingState {
    None,
    String,
    Symbol,
    Number,
    Ident { upper: bool },
    Comment { block: bool },
}

fn char_to_lexing_state(
    c: char,
    line_comment: char,
    block_comment: Option<(char, char)>,
) -> LexingState {
    if c.is_whitespace() {
        return LexingState::None;
    }
    if c.is_numeric() {
        return LexingState::Number;
    }
    if c.is_alphabetic() || c == '_' {
        return LexingState::Ident {
            upper: c.is_uppercase(),
        };
    }
    if c == '"' {
        return LexingState::String;
    }
    if c == line_comment {
        return LexingState::Comment { block: false };
    }
    if block_comment.is_some_and(|(start, _)| c == start) {
        return LexingState::Comment { block: true };
    }
    LexingState::Symbol
}

pub fn lex<'l, Token: Copy>(
    source: &'l str,
    symbols: HashMap<&'static str, Token>,
    uppercase: Token,
    lowercase: Token,
    string: Token,
    number: Token,
    error: Token,
    line_comment: char,
    block_comment: Option<(char, char)>,
) -> Vec<Lexeme<'l, Token>> {
    let mut lexemes = vec![];
    let mut state = LexingState::None;
    let mut line = 0;
    let mut column = 0;
    let mut index_from = 0;
    let mut line_from = 0;
    let mut column_from = 0;
    for (i, char) in source.chars().enumerate() {
        let new_state = char_to_lexing_state(char, line_comment, block_comment);
        if new_state != state {
            let mut ignore = false;
            match state {
                LexingState::None => {}
                LexingState::String => {
                    ignore = true;
                }
                LexingState::Symbol => lexemes.push(Lexeme {
                    token: error,
                    line: (line_from, line),
                    column: (column_from, column),
                    source: &source[index_from..i],
                }),
                LexingState::Number => lexemes.push(Lexeme {
                    token: number,
                    line: (line_from, line),
                    column: (column_from, column),
                    source: &source[index_from..i],
                }),
                LexingState::Ident { upper } => lexemes.push(Lexeme {
                    token: if upper { uppercase } else { lowercase },
                    line: (line_from, line),
                    column: (column_from, column),
                    source: &source[index_from..i],
                }),
                LexingState::Comment { .. } => {
                    ignore = true;
                }
            }
            if !ignore {
                state = new_state;
                index_from = i;
                line_from = line;
                column_from = column;
            }
        }

        column += 1;
        match state {
            LexingState::None => {}
            LexingState::String => {
                if char == '"' && index_from < i {
                    lexemes.push(Lexeme {
                        token: string,
                        line: (line_from, line),
                        column: (column_from, column),
                        source: &source[index_from..=i],
                    });
                    state = LexingState::None;
                }
            }
            LexingState::Symbol => {
                let token = symbols.get(&source[index_from..=i]);
                if let Some(token) = token {
                    lexemes.push(Lexeme {
                        token: *token,
                        line: (line_from, line),
                        column: (column_from, column),
                        source: &source[index_from..=i],
                    });
                    state = LexingState::None;
                }
            }
            LexingState::Number => {}
            LexingState::Ident { .. } => {}
            LexingState::Comment { block } => {
                if block {
                    let Some((_, end)) = block_comment else {
                        panic_any(
                            "created a block comment without having a start character defined",
                        )
                    };
                    if char == end {
                        state = LexingState::None;
                    }
                } else {
                    if char == '\n' {
                        state = LexingState::None;
                    }
                }
            }
        }
        if char == '\n' {
            line += 1;
            column = 0;
        }
    }
    lexemes
}
