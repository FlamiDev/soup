use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Word {
    pub value: WordValue,
    pub line: usize,
    pub column_from: usize,
    pub column_to: usize,
}

impl Word {
    pub fn get_word(&self) -> Option<&str> {
        match &self.value {
            WordValue::Word(word) => Some(word),
            _ => None,
        }
    }
    pub fn get_brackets(&self, open: char, close: char) -> Option<&Vec<Word>> {
        match &self.value {
            WordValue::Brackets {
                open: o,
                inner,
                close: c,
            } if *o == open && *c == close => Some(inner),
            _ => None,
        }
    }
    pub fn display_text(&self) -> String {
        match &self.value {
            WordValue::Word(word) => word.clone(),
            WordValue::Brackets { open, close, .. } => {
                format!("{}{}", open, close)
            }
        }
    }
    pub fn pos(&self) -> (usize, usize) {
        (self.line, self.column_from)
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.display_text(), f)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WordValue {
    Word(String),
    Brackets {
        open: char,
        inner: Vec<Word>,
        close: char,
    },
}

impl Display for WordValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            WordValue::Word(word) => word.clone(),
            WordValue::Brackets { open, close, .. } => {
                format!("{}{}", open, close)
            }
        };
        write!(f, "{}", str)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BracketPair {
    pub open: char,
    pub close: char,
}

/// Splits the text into words. Parses nested brackets as well.
/// WARNING: Ignores incorrect closing brackets.
/// For example, if the text is `"a (b [c d)"`, the result will act like the text was `"a (b [c d])"`.
/// Similarly, if the text is `"a (b [c] d])"`, the result will act like the text was `"a (b [c] d)"`.
/// The reason for this is that these errors are better handled by the parser.
/// It has more context and can provide better error messages, also displaying what was expected instead.
pub fn split_words(text: &str, brackets: Vec<BracketPair>) -> Vec<Word> {
    let bracket_chars: Vec<char> = brackets
        .iter()
        .flat_map(|bp| vec![bp.open, bp.close])
        .collect();
    let mut res = TempBrackets::new(brackets);
    for (line_number, line) in text.lines().enumerate() {
        let line = line.split_once("//").map_or(line, |(line, _)| line);
        if line.trim().is_empty() {
            continue;
        }
        let mut current_text = String::new();
        let mut column_from = 0;
        for (column_number, character) in line.chars().enumerate() {
            if current_text.starts_with('"') {
                if character == '"' && !current_text.ends_with('\\') {
                    current_text.push(character);
                    res.push(line_number, column_from, column_number, current_text);
                    current_text = String::new();
                    continue;
                }
                current_text.push(character);
                continue;
            }
            if character.is_whitespace() {
                if !current_text.is_empty() {
                    res.push(line_number, column_from, column_number, current_text);
                }
                current_text = String::new();
                continue;
            }
            let Some(last) = current_text.chars().last() else {
                column_from = column_number;
                current_text.push(character);
                continue;
            };
            let is_or_was_bracket =
                bracket_chars.contains(&last) || bracket_chars.contains(&character);
            let is_same_word_type = (last.is_alphanumeric() == character.is_alphanumeric())
                || character == '_'
                || last == '_';
            let is_number_period = last.is_numeric() && character == '.' || last == '.' && character.is_numeric();
            let same_word = is_number_period || !is_or_was_bracket && is_same_word_type;
            if same_word {
                current_text.push(character);
            } else {
                res.push(line_number, column_from, column_number, current_text);
                current_text = character.to_string();
            }
        }
        if !current_text.is_empty() {
            res.push(line_number, column_from, line.len(), current_text);
        }
    }
    res.finish()
}

struct TempBrackets {
    root: Vec<Word>,
    stack: Vec<(BracketPair, usize, usize, Vec<Word>)>,
    brackets: Vec<BracketPair>,
}

impl TempBrackets {
    fn new(brackets: Vec<BracketPair>) -> Self {
        Self {
            root: Vec::new(),
            stack: Vec::new(),
            brackets,
        }
    }

    fn push(&mut self, line: usize, column_from: usize, column_to: usize, value: String) {
        if let Some(bp) = self.brackets.iter().find(|bp| bp.open.to_string() == value) {
            let inner = Vec::new();
            self.stack.push((bp.clone(), line, column_from, inner));
            return;
        }
        if let Some(bp) = self
            .brackets
            .iter()
            .find(|bp| bp.close.to_string() == value)
        {
            while let Some((brackets, line, column_from, words)) = self.stack.pop() {
                let level_higher = self
                    .stack
                    .last_mut()
                    .map_or(&mut self.root, |(_, _, _, inner)| inner);
                let word = Word {
                    value: WordValue::Brackets {
                        open: brackets.open,
                        inner: words,
                        close: brackets.close,
                    },
                    line,
                    column_from,
                    column_to,
                };
                level_higher.push(word);
                if &brackets == bp {
                    break;
                }
            }
            return;
        }
        let word = Word {
            value: WordValue::Word(value),
            line,
            column_from,
            column_to,
        };
        if let Some((_, _, _, inner)) = self.stack.last_mut() {
            inner.push(word);
        } else {
            self.root.push(word);
        }
    }

    fn finish(mut self) -> Vec<Word> {
        while let Some((brackets, line, column_from, words)) = self.stack.pop() {
            let level_higher = self
                .stack
                .last_mut()
                .map_or(&mut self.root, |(_, _, _, inner)| inner);
            let word = Word {
                value: WordValue::Brackets {
                    open: brackets.open,
                    inner: words,
                    close: brackets.close,
                },
                line,
                column_from,
                column_to: 0,
            };
            level_higher.push(word);
        }
        self.root
    }
}
