use crate::Word;

pub fn split_words(text: &str, bracket_chars: &'static str) -> Vec<Word> {
    let mut res = Vec::new();
    let mut current_word = Word {
        line_number: 0,
        column_number: 0,
        text: String::new(),
    };
    for (line_number, line) in text.lines().enumerate() {
        for (column_number, character) in line.chars().enumerate() {
            if current_word.text.starts_with('\"') {
                if character == '\"' && !current_word.text.ends_with('\\') {
                    current_word.text.push(character);
                    res.push(current_word);
                    current_word = Word {
                        line_number,
                        column_number,
                        text: String::new(),
                    };
                    continue;
                }
                current_word.text.push(character);
                continue;
            }
            if character.is_whitespace() {
                if !current_word.text.is_empty() {
                    res.push(current_word);
                    current_word = Word {
                        line_number,
                        column_number,
                        text: String::new(),
                    };
                }
                continue;
            }
            let Some(last) = current_word.text.chars().last() else {
                current_word.line_number = line_number;
                current_word.column_number = column_number;
                current_word.text.push(character);
                continue;
            };
            let is_or_was_bracket =
                bracket_chars.contains(last) || bracket_chars.contains(character);
            let is_same_word_type = (last.is_alphanumeric() == character.is_alphanumeric())
                || character == '_'
                || last == '_';
            let same_word = !is_or_was_bracket && is_same_word_type;
            if same_word {
                current_word.text.push(character);
            } else {
                res.push(current_word);
                current_word = Word {
                    line_number,
                    column_number,
                    text: character.to_string(),
                };
            }
        }
        if !current_word.text.is_empty() {
            res.push(current_word);
            current_word = Word {
                line_number,
                column_number: 0,
                text: String::new(),
            };
        }
    }
    res
}
