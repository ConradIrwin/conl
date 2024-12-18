use std::borrow::Cow;

#[cfg(test)]
mod test;

/// A Token is a single token in the input with a line number attached.
/// They are generated by [parse] and [tokenize]. Use [Token::unescape] to get the actual value.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Token<'tok> {
    /// Newline is \r \n or \r\n (you can likely skip this token unless building a formatter)
    Newline(usize),
    /// Comment (you can likely skip this token unless building a formatter)
    Comment(usize, &'tok str),
    /// Indent marks the beginning of a new section.
    /// Once you receive the first [Token::MapKey] or [Token::ListItem] you know if it's a map or a list
    Indent(usize),
    /// Outdent marks the end of a section. You will receive one [Token::Outdent] per [Token::Indent]
    /// except in case of errors.
    Outdent(usize),
    /// ListItem indicates a new list item. Its value will be the next [Token::Value], [Token::MultilineValue] or [Token::Indent] you receive.
    ListItem(usize),
    /// Key indicates a new map key. Its value will be the next [Token::Value], [Token::MultilineValue] or [Token::Indent] you receive.
    MapKey(usize, &'tok str),
    /// Value contains a single-line value
    Value(usize, &'tok str),
    /// MultilineHint contains the language tag for a multiline value (you can likely skip this token unless building a formatter)
    MultilineHint(usize, &'tok str),
    /// MultilineValue contains a multiline value
    MultilineValue(usize, &'tok str, &'tok str),
    /// NoValue indicates that a key or item had no value.
    NoValue(usize),
}

impl<'tok> Token<'tok> {
    /// returns the line on which the token starts
    pub fn line_number(&self) -> usize {
        match self {
            Token::Newline(lno) => *lno,
            Token::Comment(lno, _) => *lno,
            Token::Indent(lno) => *lno,
            Token::Outdent(lno) => *lno,
            Token::ListItem(lno) => *lno,
            Token::MapKey(lno, _) => *lno,
            Token::Value(lno, _) => *lno,
            Token::MultilineHint(lno, _) => *lno,
            Token::MultilineValue(lno, _, _) => *lno,
            Token::NoValue(lno) => *lno,
        }
    }

    /// returns the line on which the token starts (to put in error messages like: "expected a map key got a X")
    pub fn name(&self) -> &'static str {
        match self {
            Token::Newline(..) => "newline",
            Token::Comment(..) => "comment",
            Token::Indent(..) => "indent",
            Token::Outdent(..) => "outdent",
            Token::ListItem(..) => "list item",
            Token::MapKey(..) => "map key",
            Token::Value(..) => "value",
            Token::NoValue(..) => "no value",
            Token::MultilineHint(..) => "multiline hint",
            Token::MultilineValue(..) => "multiline value",
        }
    }

    /// returns the actual value of a token (removing quotes if present)
    /// This is most useful for [Token::MapKey], [Token::Value] and [Token::MultilineValue]; but also
    /// returns the contents of a [Token::Comment] or [Token::MultilineHint] for formatters.
    /// Other tokens always return Ok(Cow::Borrowed(""))
    pub fn unescape(&self) -> Result<Cow<'tok, str>, SyntaxError> {
        use Token::*;
        match self {
            MapKey(lno, val) | Value(lno, val) => {
                if !val.starts_with('"') {
                    return Ok(Cow::Borrowed(val));
                }
                if val.starts_with('"') && val.ends_with('"') && val.len() > 1 {
                    let possible = &val[1..val.len() - 1];
                    if !possible.contains(['"', '\\']) {
                        return Ok(Cow::Borrowed(possible));
                    }
                }

                let mut output = String::new();
                let mut chars = val.chars().skip(1);
                let mut escaped = false;
                let mut closed = false;
                'outer: while let Some(c) = chars.next() {
                    if !escaped {
                        if c == '\\' {
                            escaped = true
                        } else if c == '"' {
                            closed = true;
                            break 'outer;
                        } else {
                            output.push(c)
                        }
                        continue;
                    }
                    match c {
                        '"' => output.push(c),
                        '\\' => output.push('\\'),
                        'n' => output.push('\n'),
                        'r' => output.push('\r'),
                        't' => output.push('\t'),
                        '{' => {
                            let mut found = String::new();
                            loop {
                                match chars.next() {
                                    None => break 'outer,
                                    Some('}') => break,
                                    Some(c) => found.push(c),
                                }
                            }
                            let Some(ch) = u32::from_str_radix(&found, 16)
                                .ok()
                                .filter(|_| found.len() <= 8)
                                .and_then(|num| num.try_into().ok())
                            else {
                                return Err(SyntaxError {
                                    lno: *lno,
                                    msg: format!("invalid escape code: \\{{{}}}", found),
                                });
                            };
                            output.push(ch)
                        }
                        _ => {
                            return Err(SyntaxError {
                                lno: *lno,
                                msg: format!("invalid escape code: \\{}", c),
                            })
                        }
                    }
                    escaped = false;
                }
                if escaped {
                    return Err(SyntaxError {
                        lno: *lno,
                        msg: "invalid escape code: end of string".to_string(),
                    });
                }
                if chars.next().is_some() {
                    return Err(SyntaxError {
                        lno: *lno,
                        msg: "extra characters after quotes".to_string(),
                    });
                }
                if !closed {
                    return Err(SyntaxError {
                        lno: *lno,
                        msg: "unclosed quotes".to_string(),
                    });
                }
                Ok(Cow::Owned(output))
            }
            MultilineValue(_, indent, val) => {
                if !val.chars().any(is_newline_char) {
                    return Ok(Cow::Borrowed(val));
                }
                let content = val
                    .lines()
                    .flat_map(|line| line.split('\r'))
                    .enumerate()
                    .flat_map(|(i, line)| {
                        let newline = if i > 0 { "\n" } else { "" };
                        if let Some(content) = line.strip_prefix(indent) {
                            vec![newline, content]
                        } else if i == 0 {
                            vec![newline, line]
                        } else {
                            vec![newline]
                        }
                    })
                    .collect::<String>();
                Ok(Cow::Owned(content))
            }
            Comment(.., comment) => Ok(Cow::Borrowed(comment)),
            MultilineHint(.., hint) => Ok(Cow::Borrowed(hint)),
            _ => Ok(Cow::Borrowed("")),
        }
    }
}

#[derive(Debug)]
/// SyntaxError is returned when the input is invalid.
pub struct SyntaxError {
    pub lno: usize,
    pub msg: String,
}

impl SyntaxError {
    fn new(lno: usize, msg: impl Into<String>) -> Self {
        Self {
            lno,
            msg: msg.into(),
        }
    }
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.lno, self.msg)
    }
}

fn is_whitespace(&c: &u8) -> bool {
    c == b' ' || c == b'\t'
}
fn is_whitespace_char(c: char) -> bool {
    c == ' ' || c == '\t'
}
fn is_newline(&c: &u8) -> bool {
    c == b'\r' || c == b'\n'
}
fn is_newline_char(c: char) -> bool {
    c == '\r' || c == '\n'
}
fn newline_size(s: &[u8]) -> usize {
    if s.first() == Some(&b'\r') && s.get(1) == Some(&b'\n') {
        2
    } else {
        1
    }
}

/// tokenize iterates over the CONL tokens in the input. It does not
/// validate the structure of the file, so is suitable for using if you
/// need error-tolerant parsing (e.g. for a linter).
/// It continues after yielding errors.
/// See [parse] for a stricter interface.
pub fn tokenize(input: &[u8]) -> Tokenizer<'_> {
    Tokenizer {
        input,
        expect_indent: true,
        expect_value: false,
        expect_multiline: false,
        current_indent: None,
        indent_stack: vec![&[]],
        lno: 1,
    }
}

/// See [tokenize]
pub struct Tokenizer<'tok> {
    input: &'tok [u8],
    indent_stack: Vec<&'tok [u8]>,
    current_indent: Option<&'tok [u8]>,
    expect_indent: bool,
    expect_value: bool,
    expect_multiline: bool,
    lno: usize,
}

impl<'tok> Tokenizer<'tok> {
    fn consume_whitespace(&mut self) -> (&'tok [u8], &'tok [u8]) {
        let i = self.input.iter().position(|c| !is_whitespace(c));
        self.input.split_at(i.unwrap_or(self.input.len()))
    }

    fn consume_comment(&mut self, rest: &'tok [u8]) -> Result<Token<'tok>, SyntaxError> {
        let i = rest.iter().position(is_newline).unwrap_or(rest.len());
        let (comment, rest) = rest.split_at(i);
        self.input = rest;
        let str = std::str::from_utf8(comment)
            .map_err(|_| SyntaxError::new(self.lno, "invalid UTF-8"))?;
        Ok(Token::Comment(
            self.lno,
            str.trim_matches(is_whitespace_char),
        ))
    }

    fn consume_value(&mut self, rest: &'tok [u8]) -> Result<Token<'tok>, SyntaxError> {
        if let Some(hint) = rest.strip_prefix(&[b'"', b'"', b'"']) {
            return self.consume_multiline_hint(hint);
        }

        let mut quoted = rest.first() == Some(&b'"');
        let mut end = rest.len();
        let mut was_escape = false;
        for (i, c) in rest.iter().enumerate() {
            if is_newline(c) || (c == &b';' && !quoted) {
                end = i;
                break;
            }
            if i > 0 && !was_escape && c == &b'"' {
                quoted = false;
            }
            was_escape = c == &b'\\'
        }

        let (value, rest) = rest.split_at(end);
        self.input = rest;
        let str =
            std::str::from_utf8(value).map_err(|_| SyntaxError::new(self.lno, "invalid UTF-8"))?;
        let value = str.trim_matches(is_whitespace_char);
        Ok(Token::Value(self.lno, value))
    }

    fn consume_multiline_hint(&mut self, rest: &'tok [u8]) -> Result<Token<'tok>, SyntaxError> {
        let mut end = rest.len();
        for (i, c) in rest.iter().enumerate() {
            if is_newline(c) || c == &b';' {
                end = i;
                break;
            }
        }
        let (value, rest) = rest.split_at(end);
        self.input = rest;

        let str =
            std::str::from_utf8(value).map_err(|_| SyntaxError::new(self.lno, "invalid UTF-8"))?;
        let value = str.trim_matches(is_whitespace_char);

        self.expect_multiline = true;
        Ok(Token::MultilineHint(self.lno, value))
    }

    fn consume_key(&mut self, rest: &'tok [u8]) -> Result<Token<'tok>, SyntaxError> {
        let mut end = rest.len();
        let mut was_escape = false;
        let mut quoted = rest.first() == Some(&b'"');

        for (i, c) in rest.iter().enumerate() {
            if is_newline(c) || (c == &b';' && !quoted) || (c == &b'=' && !quoted) {
                end = i;
                break;
            }
            if i > 0 && !was_escape && c == &b'"' {
                quoted = false;
            }
            was_escape = c == &b'\\'
        }

        let (key, rest) = rest.split_at(end);
        self.expect_value = true;
        self.input = rest;
        if self.input.first() == Some(&b'=') {
            self.input = &self.input[1..];
        }

        let str =
            std::str::from_utf8(key).map_err(|_| SyntaxError::new(self.lno, "invalid UTF-8"))?;
        Ok(Token::MapKey(
            self.lno,
            str.trim_matches(is_whitespace_char),
        ))
    }

    fn consume_multiline(&mut self, indent: &'tok [u8]) -> Result<Token<'tok>, SyntaxError> {
        let mut end = 0;
        let lno = self.lno;
        let mut was_cr = false;

        for line in self.input.split_inclusive(is_newline) {
            if line.starts_with(indent) || line.iter().all(|c| is_whitespace(c) || is_newline(c)) {
                if !(was_cr && line == [b'\n']) {
                    self.lno += 1;
                }
                was_cr = line.last() == Some(&b'\r');
                end += line.len();
            } else {
                break;
            }
        }
        let (value, rest) = self.input.split_at(end);
        self.input = rest;

        let str = std::str::from_utf8(value).map_err(|_| SyntaxError::new(lno, "invalid UTF-8"))?;
        Ok(Token::MultilineValue(
            lno,
            std::str::from_utf8(indent).unwrap(),
            str.trim_matches(|c| is_newline_char(c) || is_whitespace_char(c)),
        ))
    }
}

impl<'tok> Iterator for Tokenizer<'tok> {
    type Item = Result<Token<'tok>, SyntaxError>;

    fn next(&mut self) -> Option<Self::Item> {
        let (indent, rest) = if let Some(current_indent) = self.current_indent.take() {
            (current_indent, self.input)
        } else {
            self.consume_whitespace()
        };
        if rest.first().is_some_and(is_newline) {
            self.input = &rest[newline_size(rest)..];
            self.lno += 1;
            self.expect_indent = true;
            self.expect_value = false;
            return Some(Ok(Token::Newline(self.lno - 1)));
        }

        let Some(first) = rest.first() else {
            if self.indent_stack.len() > 1 {
                self.indent_stack.pop();
                return Some(Ok(Token::Outdent(self.lno)));
            }
            return None;
        };

        if *first == b';' && !(self.expect_indent && self.expect_multiline) {
            return Some(self.consume_comment(&rest[1..]));
        }

        if self.expect_indent {
            self.expect_indent = false;
            let &current = self.indent_stack.last().unwrap();
            if self.expect_multiline {
                self.expect_multiline = false;
                if indent.len() > current.len() && indent.starts_with(current) {
                    return Some(self.consume_multiline(indent));
                }
            }
            if indent != current {
                if indent.len() > current.len() && indent.starts_with(current) {
                    self.indent_stack.push(indent);
                    self.input = rest;
                    return Some(Ok(Token::Indent(self.lno)));
                } else {
                    self.indent_stack.pop();
                    self.current_indent = Some(indent);
                    self.expect_indent = true;
                    return Some(Ok(Token::Outdent(self.lno)));
                }
            }
        }

        match first {
            b'=' if !self.expect_value => {
                self.expect_value = true;
                self.input = &rest[1..];
                Some(Ok(Token::ListItem(self.lno)))
            }
            _ if self.expect_value => {
                self.expect_value = false;
                Some(self.consume_value(rest))
            }
            _ => Some(self.consume_key(rest)),
        }
    }
}

#[derive(PartialEq)]
enum SectionType {
    List,
    Map,
}

/// parse iterates over a CONL file, returning [Token]s. In the case of an error it will
/// yield a [SyntaxError] and then stop returning more tokens.
/// You can likely ignore [Token::Newline], [Token::Comment] and [Token::MultilineHint].
/// The structure of the file is validated, so you can be sure that you'll see pairs of:
/// * one of [Token::MapKey] or [Token::ListItem]
/// * one of [Token::Value], [Token::MultilineValue] or [Token::Indent] ... [Token::Outdent]
///
/// Within a given Indent/Outdent section you'll always see either only [Token::MapKey] or [Token::ListItem].
/// For an error-tolerant version see [tokenize].
pub fn parse(input: &[u8]) -> Parser<'_> {
    Parser::new(input)
}

/// See [parse]
pub struct Parser<'tok> {
    tokenizer: Tokenizer<'tok>,
    peek: Option<Option<Token<'tok>>>,
    multiline_hint: Option<usize>,
    needs_value: Option<usize>,
    errored: bool,
    stack: Vec<Option<SectionType>>,
}

impl<'tok> Parser<'tok> {
    fn new(input: &'tok [u8]) -> Self {
        Parser {
            tokenizer: tokenize(input),
            multiline_hint: None,
            needs_value: None,
            errored: false,
            stack: vec![None],
            peek: None,
        }
    }
}

impl<'tok> Iterator for Parser<'tok> {
    type Item = Result<Token<'tok>, SyntaxError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.errored {
            return None;
        }
        use Token::*;

        let next = if let Some(peek) = self.peek.take() {
            peek
        } else {
            match self.tokenizer.next() {
                Some(Err(e)) => {
                    self.errored = true;
                    return Some(Err(e));
                }
                None => None,
                Some(Ok(next)) => Some(next),
            }
        };

        match next {
            Some(Newline(..)) | Some(Comment(..)) => return Ok(next).transpose(),
            _ => {}
        };

        let token = if let Some(lno) = self.multiline_hint.take() {
            match next {
                Some(MultilineValue(..)) => next,
                _ => {
                    self.errored = true;
                    return Some(Err(SyntaxError::new(lno, "missing value")));
                }
            }
        } else if let Some(lno) = self.needs_value.take() {
            match next {
                Some(MultilineHint(..)) => {
                    self.multiline_hint = Some(lno);
                    next
                }
                Some(Value(..)) => next,
                Some(Indent(..)) => {
                    self.stack.push(None);
                    next
                }
                _ => {
                    self.peek = Some(next);
                    Some(Token::NoValue(lno))
                }
            }
        } else {
            match next {
                Some(MapKey(lno, value)) => {
                    let last = self.stack.last_mut().unwrap();
                    if last.get_or_insert(SectionType::Map) == &SectionType::List {
                        self.errored = true;
                        return Some(Err(SyntaxError::new(lno, "expected list item")));
                    }
                    self.needs_value = Some(lno);
                    Some(MapKey(lno, value))
                }
                Some(ListItem(lno)) => {
                    let last = self.stack.last_mut().unwrap();
                    if last.get_or_insert(SectionType::List) == &SectionType::Map {
                        self.errored = true;
                        return Some(Err(SyntaxError::new(lno, "expected map key")));
                    }
                    self.needs_value = Some(lno);
                    Some(ListItem(lno))
                }
                None | Some(Outdent(_)) => {
                    self.stack.pop();
                    next
                }
                Some(Indent(lno)) => {
                    self.errored = true;
                    return Some(Err(SyntaxError::new(lno, "unexpected indent")));
                }
                _ => {
                    unreachable!()
                }
            }
        };
        Ok(token).transpose()
    }
}
