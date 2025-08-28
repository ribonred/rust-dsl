use chumsky::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedFirstLine {
    Empty,
    Directive { name: String, pairs: Vec<(String, String)> },
}

// Build a parser for a directive line: @name key=value key2=value2
fn directive_parser() -> impl Parser<char, ParsedFirstLine, Error = Simple<char>> {
    // identifier: letters, digits, underscore
    let ident = text::ident();
    let value = none_of([' ', '\t', '\n']).repeated().at_least(1).collect::<String>();
    let pair = ident.clone().then_ignore(just('=').padded()).then(value);

    just('@')
        .ignore_then(ident.map(|s: String| s))
        .then(pair.padded().repeated())
        .then_ignore(end())
        .map(|(name, pairs_vec)| ParsedFirstLine::Directive {
            name,
            pairs: pairs_vec
                .into_iter()
                .map(|(k, v)| (k, v))
                .collect(),
        })
}

pub fn parse_first_line(src: &str) -> Result<ParsedFirstLine, String> {
    // Find first non-empty line
    let mut first_non_empty = None;
    for line in src.lines() {
        if !line.trim().is_empty() {
            first_non_empty = Some(line.trim());
            break;
        }
    }
    let Some(line) = first_non_empty else { return Ok(ParsedFirstLine::Empty); };

    if !line.starts_with('@') {
        return Err("First non-empty line must start with @".to_string());
    }

    let parser = directive_parser();
    match parser.parse(line) {
        Ok(ast) => Ok(ast),
        Err(errs) => {
            let msg = errs
                .into_iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<_>>()
                .join(", ");
            Err(format!("error: {msg}"))
        }
    }
}
