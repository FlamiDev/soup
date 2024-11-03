use std::{fmt::Debug, str::Chars};

fn split_words(input: &str) -> Vec<(i64, i64, String)> {
    fn split_recursive(mut input: Chars, mut total: Vec<String>, current: String) -> Vec<String> {
        let Some(ch) = input.next() else {
            if !current.is_empty() {
                total.push(current);
            }
            return total;
        };
        if current.chars().next() == Some('\"') {
            if ch == '\"' && current.chars().last() != Some('\\') {
                total.push(format!("{}{}", current, ch));
                return split_recursive(input, total, String::new());
            } else {
                return split_recursive(input, total, format!("{}{}", current, ch));
            }
        }
        if ch.is_whitespace() {
            if !current.is_empty() {
                total.push(current);
            }
            return split_recursive(input, total, String::new());
        }
        match current.chars().last() {
            None => split_recursive(input, total, ch.to_string()),
            Some(last) => {
                let same_word =
                    (last.is_alphanumeric() && ch.is_alphanumeric()) || ch == '_' || last == '_';
                if same_word {
                    split_recursive(input, total, format!("{}{}", current, ch))
                } else {
                    total.push(current);
                    split_recursive(input, total, ch.to_string())
                }
            }
        }
    }
    return input
        .lines()
        .enumerate()
        .flat_map(|(line_no, line)| {
            if line.starts_with("//") {
                return Vec::new();
            }
            let mut res = split_recursive(line.chars(), Vec::new(), String::new());
            res.push("\n".to_string());
            res.iter()
                .enumerate()
                .map(|(i, word)| (line_no as i64 + 1, i as i64 + 1, word.to_string()))
                .collect::<Vec<(i64, i64, String)>>()
        })
        .collect();
}

/// Parses any word that isn't a hardcoded keyword or operator.
/// Can parse types `CamelCase`, names `snake_case`, ints, floats.
/// Everything else becomes a string.
/// Empty words, names containing numbers or vice versa, and invalid string become the error token.
pub fn other<Token>(
    text: &str,
    type_token: fn(String) -> Token,
    name_token: fn(String) -> Token,
    int_token: fn(i64) -> Token,
    float_token: fn(f64) -> Token,
    string_token: fn(String) -> Token,
    error_token: fn(msg: String) -> Token,
) -> Token {
    #[derive(PartialEq)]
    enum CustomToken {
        Type,
        Name,
        Int,
        Float,
        String,
    }
    fn get_token(input: char) -> CustomToken {
        if input == '\"' {
            CustomToken::String
        } else if input == '.' {
            CustomToken::Float
        } else if input.is_numeric() {
            CustomToken::Int
        } else if input.is_uppercase() {
            CustomToken::Type
        } else {
            CustomToken::Name
        }
    }
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return error_token("Empty token".to_string());
    };
    let mut current_token = get_token(first);
    if current_token == CustomToken::String {
        if chars.last() != Some('\"') {
            return error_token(format!("Invalid string: {}", text));
        }
        return string_token(text[1..text.len() - 1].to_string());
    } else {
        for ch in chars {
            let new_token = get_token(ch);
            if new_token != current_token {
                if current_token == CustomToken::Int && new_token == CustomToken::Float {
                    current_token = CustomToken::Float;
                    continue;
                }
                if current_token == CustomToken::Float && new_token == CustomToken::Int {
                    continue;
                }
                if current_token == CustomToken::Name && new_token == CustomToken::Type {
                    continue;
                }
                if current_token == CustomToken::Type && new_token == CustomToken::Name {
                    continue;
                }
                return error_token(format!("Invalid token: {}", text));
            }
        }
    }
    match current_token {
        CustomToken::Type => type_token(text.to_string()),
        CustomToken::Name => name_token(text.to_string()),
        CustomToken::Int => int_token(text.parse::<i64>().expect("Tried to parse invalid integer")),
        CustomToken::Float => {
            float_token(text.parse::<f64>().expect("Tried to parse invalid float"))
        }
        CustomToken::String => string_token(text.to_string()),
    }
}

#[derive(Clone, PartialEq)]
pub struct PositionedToken<Token> {
    pub line_no: i64,
    pub word_no: i64,
    pub token: Token,
}
impl<Token: Debug> Debug for PositionedToken<Token> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} {} {:?}",
            self.line_no, self.word_no, self.token
        ))
    }
}

#[derive(Clone)]
struct StackItem<Token>(Vec<PositionedToken<Token>>, Option<MatchingBrackets<Token>>);
#[derive(Clone)]
pub struct MatchingBrackets<Token> {
    pub open: Token,
    pub close: Token,
    pub create: fn(Vec<PositionedToken<Token>>) -> Token,
}
pub fn brackets<Token>(
    open: Token,
    close: Token,
    create: fn(Vec<PositionedToken<Token>>) -> Token,
) -> MatchingBrackets<Token> {
    MatchingBrackets {
        open,
        close,
        create,
    }
}

/// Parses a complete input file into the correct tokens.
/// The matcher function should call `other` if it cannot find a matching keyword or operator.
pub fn parse<Token: Debug + Clone + PartialEq>(
    input: &str,
    matcher: fn(&str) -> Token,
    brackets: Vec<MatchingBrackets<Token>>,
    error_token: fn(msg: String) -> Token,
) -> Vec<PositionedToken<Token>> {
    let words = split_words(input);
    let mut stack = vec![StackItem(Vec::new(), None)];
    for (line_no, word_no, word) in words {
        let token = matcher(word.as_str());
        if let Some(bracket_open) = brackets.iter().find(|b| token == b.open) {
            stack.push(StackItem(Vec::new(), Some(bracket_open.clone())));
            continue;
        };
        if let Some(bracket_close) = brackets.iter().find(|b| token == b.close) {
            if stack.is_empty() {
                panic!("Stack should always contain the root element");
            }
            while let Some(StackItem(mut tokens, bracket)) = stack.pop() {
                let Some(bracket) = bracket else {
                    // Popped the root
                    tokens.push(PositionedToken {
                        line_no,
                        word_no,
                        token: error_token(format!(
                            "Missing opening bracket {:?} for closing {:?}",
                            bracket_close.open, bracket_close.close
                        )),
                    });
                    stack.push(StackItem(tokens, None));
                    break;
                };

                let last = stack
                    .last_mut()
                    .expect("Stack should always contain the root element");

                if bracket.close == bracket_close.close {
                    let pos = tokens
                        .first()
                        .map_or((line_no, word_no - 1), |t| (t.line_no, t.word_no));
                    last.0.push(PositionedToken {
                        line_no: pos.0,
                        word_no: pos.1,
                        token: (bracket.create)(tokens),
                    });
                    break;
                }

                tokens.push(PositionedToken {
                    line_no,
                    word_no,
                    token: error_token(format!(
                        "Missing closing bracket {:?} for {:?}, found {:?}",
                        bracket.close, bracket.open, bracket_close.close
                    )),
                });
            }
            continue;
        };
        stack
            .last_mut()
            .expect("Stack should always contain the root element")
            .0
            .push(PositionedToken {
                line_no,
                word_no,
                token,
            });
    }
    stack
        .into_iter()
        .reduce(|mut acc, i| {
            if let Some(bracket) = i.1 {
                acc.0.push(PositionedToken {
                    line_no: 0,
                    word_no: 0,
                    token: error_token(format!(
                        "Missing closing bracket {:?} for {:?}",
                        bracket.close, bracket.open
                    )),
                });
            }
            acc.0.extend(i.0);
            acc
        })
        .expect("Stack should always contain the root element")
        .0
}
