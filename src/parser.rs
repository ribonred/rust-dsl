use chumsky::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedFirstLine {
    Empty,
    Directive {
        name: String,
        pairs: Vec<(String, String)>,
    },
}

fn directive_parser() -> impl Parser<char, ParsedFirstLine, Error = Simple<char>> {
    let ident = text::ident();
    let value = none_of([' ', '\t', '\n'])
        .repeated()
        .at_least(1)
        .collect::<String>();
    let pair = ident.clone().then_ignore(just('=').padded()).then(value);
    just('@')
        .ignore_then(ident.map(|s: String| s))
        .then(pair.padded().repeated())
        .then_ignore(end())
        .map(|(name, pairs_vec)| ParsedFirstLine::Directive {
            name,
            pairs: pairs_vec,
        })
}

pub fn parse_first_line(src: &str) -> Result<ParsedFirstLine, String> {
    let mut first_non_empty = None;
    for line in src.lines() {
        if !line.trim().is_empty() {
            first_non_empty = Some(line.trim());
            break;
        }
    }
    let Some(line) = first_non_empty else {
        return Ok(ParsedFirstLine::Empty);
    };
    if !line.starts_with('@') {
        return Err("First non-empty line must start with @".into());
    }
    match directive_parser().parse(line) {
        Ok(ast) => Ok(ast),
        Err(errs) => {
            let msg = errs
                .into_iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<_>>()
                .join(", ");
            Err(format!("DSL parse error: {msg}"))
        }
    }
}

// Highlighting --------------------------------------------------------------

#[derive(Debug, Clone)]
struct TokenSpan {
    kind: &'static str,
    start: usize,
    end: usize,
    text: String,
}

fn lexer() -> impl Parser<char, Vec<TokenSpan>, Error = Simple<char>> {
    let at = just('@').map_with_span(|_, span: std::ops::Range<usize>| TokenSpan {
        kind: "At",
        start: span.start,
        end: span.end,
        text: "@".into(),
    });
    let ident = text::ident().map_with_span(|s: String, span: std::ops::Range<usize>| TokenSpan {
        kind: "Ident",
        start: span.start,
        end: span.end,
        text: s,
    });
    let eq = just('=').map_with_span(|_, span: std::ops::Range<usize>| TokenSpan {
        kind: "Equals",
        start: span.start,
        end: span.end,
        text: "=".into(),
    });
    let ws = one_of(" \t")
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map_with_span(|s, span: std::ops::Range<usize>| TokenSpan {
            kind: "Ws",
            start: span.start,
            end: span.end,
            text: s,
        });
    let value = none_of([' ', '\t', '\n', '@', '='])
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map_with_span(|s, span: std::ops::Range<usize>| TokenSpan {
            kind: "Value",
            start: span.start,
            end: span.end,
            text: s,
        });
    choice((at, eq, ws, ident, value))
        .repeated()
        .then_ignore(end())
}

pub fn highlight_first_line_json(src: &str) -> String {
    let mut first_non_empty = None;
    let mut offset_base = 0usize;
    for line in src.lines() {
        let trimmed_line = line.trim_end_matches(['\r']);
        if !trimmed_line.trim().is_empty() {
            first_non_empty = Some((trimmed_line, offset_base));
            break;
        }
        offset_base += line.len() + 1; // include newline
    }
    let Some((line, base)) = first_non_empty else {
        return "[]".into();
    };
    match lexer().parse(line) {
        Ok(tokens) => {
            let mut out = String::from("[");
            for (i, t) in tokens.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                let escaped = t.text.replace('"', "\\\"");
                out.push_str(&format!(
                    "{{\"kind\":\"{}\",\"start\":{},\"end\":{},\"text\":\"{}\"}}",
                    t.kind,
                    base + t.start,
                    base + t.end,
                    escaped
                ));
            }
            out.push(']');
            out
        }
        Err(_) => "[]".into(),
    }
}

// ---------------- Tests (native only) -----------------
#[cfg(test)]
mod tests {
    use super::*;

    fn show(label: &str, src: &str) {
        println!("-- {label} --");
        println!("input: {src:?}");
        println!("parse: {:?}", parse_first_line(src));
        println!("highlight: {}", highlight_first_line_json(src));
        println!();
    }

    #[test]
    fn basic_cases() {
        show("empty", "\n\n");
        show("simple", "@option\nrest");
        show("pairs", "@option key=value count=10\n");
        show("path", "@option path=/usr/local/bin format=json\n");
        show("missing_at", "option key=value\n");
        show("missing_value", "@option key=\n");
    }
}
