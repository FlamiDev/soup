use parser_lib::ParseError;
use std::collections::HashMap;
use yansi::Paint;

pub fn show_errors(code: &str, errors: Vec<ParseError>) {
    let mut error_file = ErrorFile::new(code);
    error_file.insert_all(errors);
    error_file.print_errors();
}

struct Error {
    from: usize,
    to: usize,
    expected: Vec<String>,
    got: String,
}

struct ErrorFile<'l> {
    lines: Vec<&'l str>,
    errors: HashMap<usize, Vec<Error>>,
}

impl<'l> ErrorFile<'l> {
    fn new(code: &'l str) -> Self {
        Self {
            lines: code.lines().collect(),
            errors: HashMap::new(),
        }
    }

    fn print_errors(&self) {
        let mut errors: Vec<_> = self.errors.iter().collect();
        errors.sort_by(|(line_a, _), (line_b, _)| line_a.cmp(line_b));
        for (line, errors) in errors {
            for error in errors {
                let message = format!(
                    "Expected {} but got {}",
                    error.expected.join(" or "),
                    error.got
                );
                if *line == 0 {
                    println!("???? | {}", message);
                    continue;
                }
                println!("{:<4} | {}", line + 1, self.lines[*line]);
                println!(
                    "     | {:indent$}{:^<width$}{}",
                    "",
                    "^".bold().red(),
                    message.red(),
                    indent = error.from,
                    width = if error.to > error.from {
                        error.to - error.from
                    } else {
                        1
                    }
                );
            }
        }
    }

    fn insert(&mut self, new: ParseError) {
        let (line, from, to, got) = new
            .got
            .map(|w| (w.line, w.column_from, w.column_to, w.value.to_string()))
            .unwrap_or((0, 0, 0, "<<nothing>>".to_string()));
        let errors = self.errors.entry(line).or_default();
        if let Some(err) = errors
            .iter_mut()
            .find(|err| err.from == from && err.to == to && err.got == got)
        {
            if !err.expected.iter().any(|e| e == &new.expected) {
                err.expected.push(new.expected);
            }
        } else {
            errors.push(Error {
                from,
                to,
                expected: vec![new.expected],
                got,
            });
            errors.sort_by(|a, b| a.from.cmp(&b.from).reverse().then(a.to.cmp(&b.to)));
        }
    }

    fn insert_all(&mut self, errors: Vec<ParseError>) {
        for err in errors {
            self.insert(err);
        }
    }
}
