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
                .map(|(i, word)| (line_no as i64, i as i64, word.to_string()))
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
        return error_token("<empty word>".to_string());
    };
    let mut current_token = get_token(first);
    if current_token == CustomToken::String {
        if chars.last() != Some('\"') {
            return error_token(text.to_string());
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
                return error_token(text.to_string());
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

#[derive(Clone)]
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

/// Parses a complete input file into the correct tokens.
/// The matcher function should call `other` if it cannot find a matching keyword or operator.
pub fn parse<Token>(input: &str, matcher: fn(&str) -> Token) -> Vec<PositionedToken<Token>> {
    let words = split_words(input);
    let mut tokens = Vec::new();
    for (line_no, word_no, word) in words {
        let token = matcher(word.as_str());
        tokens.push(PositionedToken {
            line_no,
            word_no,
            token,
        });
    }
    tokens
}

pub fn debug_invalid<Token: Debug>(
    tokens: &Vec<PositionedToken<Token>>,
    invalid: fn(&Token) -> bool,
) -> bool {
    let invalid_tokens: Vec<&PositionedToken<Token>> =
        tokens.iter().filter(|t| invalid(&t.token)).collect();
    if !invalid_tokens.is_empty() {
        for token in invalid_tokens {
            println!(
                "Invalid token at line {}, word {}: <{:?}>",
                token.line_no, token.word_no, token.token
            );
        }
        return true;
    }
    false
}
