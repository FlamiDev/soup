#[derive(Debug, PartialEq)]
pub enum Token {
    TypeKeyword,
    LetKeyword,
    MatchKeyword,
    DocKeyword,
    TestKeyword,
    AssertKeyword,
    MockKeyword,
    ImportKeyword,
    ExportKeyword,
    TraitKeyword,
    EqualsSign,
    DoubleEqualsSign,
    NotEqualsSign,
    LessThanSign,
    GreaterThanSign,
    LessThanEqualsSign,
    GreaterThanEqualsSign,
    SpreadRangeOperator,
    VerticalBar,
    ArrowRight,
    ArrayOpen,
    ArrayClose,
    ParenOpen,
    ParenClose,
    BraceOpen,
    BraceClose,
    Comma,
    Dot,
    Colon,
    Semicolon,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Underscore,
    Bang,
    QuestionMark,
    Type(String),
    Name(String),
    Int(i64),
    Float(f64),
    String(String),
    Invalid(i64, i64, String),
    InvalidString(i64, i64, String),
}

use std::str::Chars;

fn split_words(input: &str) -> Vec<(i64, i64, String)> {
    fn split_recursive(mut input: Chars, mut total: Vec<String>, current: String) -> Vec<String> {
        let Some(ch) = input.next() else { return total };
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
            split_recursive(line.chars(), Vec::new(), String::new())
                .iter()
                .enumerate()
                .map(|(i, word)| (line_no as i64, i as i64, word.to_string()))
                .collect::<Vec<(i64, i64, String)>>()
        })
        .collect();
}

fn parse_custom(line_no: i64, word_no: i64, input: &str) -> Token {
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
    let mut chars = input.chars();
    let Some(first) = chars.next() else {
        return Token::Invalid(line_no, word_no, input.to_string());
    };
    let mut current_token = get_token(first);
    if current_token == CustomToken::String {
        if chars.last() != Some('\"') {
            return Token::InvalidString(line_no, word_no, input.to_string());
        }
        return Token::String(input[1..input.len() - 1].to_string());
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
                return Token::Invalid(line_no, word_no, input.to_string());
            }
        }
    }
    match current_token {
        CustomToken::Type => Token::Type(input.to_string()),
        CustomToken::Name => Token::Name(input.to_string()),
        CustomToken::Int => Token::Int(
            input
                .parse::<i64>()
                .expect("Tried to parse invalid integer"),
        ),
        CustomToken::Float => {
            Token::Float(input.parse::<f64>().expect("Tried to parse invalid float"))
        }
        CustomToken::String => Token::String(input.to_string()),
    }
}

pub fn parse(input: &str) -> Vec<Token> {
    let words = split_words(input);
    let mut tokens = Vec::new();
    for (line_no, word_no, word) in words {
        let token = match word.as_str() {
            "type" => Token::TypeKeyword,
            "let" => Token::LetKeyword,
            "match" => Token::MatchKeyword,
            "doc" => Token::DocKeyword,
            "test" => Token::TestKeyword,
            "assert" => Token::AssertKeyword,
            "mock" => Token::MockKeyword,
            "import" => Token::ImportKeyword,
            "export" => Token::ExportKeyword,
            "trait" => Token::TraitKeyword,
            "=" => Token::EqualsSign,
            "==" => Token::DoubleEqualsSign,
            "!=" => Token::NotEqualsSign,
            "<" => Token::LessThanSign,
            ">" => Token::GreaterThanSign,
            "<=" => Token::LessThanEqualsSign,
            ">=" => Token::GreaterThanEqualsSign,
            ".." => Token::SpreadRangeOperator,
            "|" => Token::VerticalBar,
            "->" => Token::ArrowRight,
            "[" => Token::ArrayOpen,
            "]" => Token::ArrayClose,
            "(" => Token::ParenOpen,
            ")" => Token::ParenClose,
            "{" => Token::BraceOpen,
            "}" => Token::BraceClose,
            "," => Token::Comma,
            "." => Token::Dot,
            ":" => Token::Colon,
            ";" => Token::Semicolon,
            "+" => Token::Plus,
            "-" => Token::Minus,
            "*" => Token::Asterisk,
            "/" => Token::Slash,
            "%" => Token::Percent,
            "_" => Token::Underscore,
            "!" => Token::Bang,
            "?" => Token::QuestionMark,
            custom => parse_custom(line_no, word_no, custom),
        };
        tokens.push(token);
    }
    tokens
}
