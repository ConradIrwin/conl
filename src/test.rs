use crate::{parse, Parser, SectionType, SyntaxError};

fn string_to_json(input: &str, output: &mut String) {
    output.push('"');
    for c in input.chars() {
        match c {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\x08' => output.push_str("\\b"),
            '\x0c' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            _ if c.is_ascii_control() => {
                output.push_str(&format!("\\u{:04x}", c as u32));
            }
            _ => output.push(c),
        }
    }
    output.push('"');
}

pub fn to_json(content: &[u8]) -> Result<String, SyntaxError> {
    let mut output = String::new();
    let mut parser = parse(content);
    section_to_json(&mut parser, &mut output, "")?;
    Ok(output)
}

fn section_to_json<'tok>(
    parser: &mut Parser<'tok>,
    output: &mut String,
    indent: &str,
) -> Result<(), SyntaxError> {
    use crate::Token::*;
    let mut sect_type: Option<SectionType> = None;
    while let Some(result) = parser.next() {
        match result? {
            Newline(..) | Comment(..) | MultilineIndicator(..) => {}
            Indent(..) => {
                section_to_json(parser, output, &(indent.to_string() + "  "))?;
            }
            Outdent(_) => {
                break;
            }
            ListItem(..) => match sect_type {
                None => {
                    output.push('[');
                    sect_type = Some(SectionType::List)
                }
                Some(SectionType::List) => {
                    output.push(',');
                }
                Some(SectionType::Map) => {
                    unreachable!()
                }
            },
            ref tok @ MapKey(..) => {
                match sect_type {
                    None => {
                        output.push('{');
                        sect_type = Some(SectionType::Map)
                    }
                    Some(SectionType::Map) => {
                        output.push(',');
                    }
                    Some(SectionType::List) => {
                        unreachable!()
                    }
                }
                string_to_json(&tok.unescape()?, output);
                output.push(':');
            }
            ref tok @ Value(..) | ref tok @ MultilineValue(..) => {
                string_to_json(&tok.unescape()?, output);
            }
        }
    }

    match sect_type {
        None => output.push_str("{}"),
        Some(SectionType::List) => output.push(']'),
        Some(SectionType::Map) => output.push('}'),
    }
    return Ok(());
}

#[test]
fn test_equivalence() {
    let examples = std::fs::read_to_string("test_data/examples.txt")
        .unwrap()
        .replace("␉", "\t")
        .replace("␊", "\r");

    for example in examples.split("\n===\n") {
        let (input, expected) = example.split_once("\n---\n").unwrap();

        match to_json(input.as_bytes()) {
            Ok(output) => {
                assert_eq!(output, expected.trim(), "input: {:?}", input);
            }
            Err(e) => {
                panic!("failed to parse: {}:\n{}", e, input)
            }
        }
    }
}

#[test]
fn test_errors() {
    let examples = std::fs::read_to_string("test_data/errors.txt")
        .unwrap()
        .replace("␉", "\t")
        .replace("␊", "\r");

    for example in examples.split("\n===\n") {
        dbg!("----------------------");
        let (input, expected) = example.split_once("\n---\n").unwrap();

        let input: Vec<u8> = input
            .as_bytes()
            .into_iter()
            .map(|c| if *c == b'?' { b'\xff' } else { *c })
            .collect();

        match to_json(&input) {
            Ok(output) => {
                panic!(
                    "expected to be unable to parse: {:?}, got: {:?}",
                    String::from_utf8_lossy(&input),
                    output
                )
            }
            Err(e) => {
                assert_eq!(
                    e.to_string(),
                    expected.trim().replace("␣", " "),
                    "input: {:?}",
                    input
                );
            }
        }
    }
}
