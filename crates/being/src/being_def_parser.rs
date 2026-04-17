use bevy::{platform::collections::HashMap, prelude::*};
use common::def_db::DefValue;

#[derive(Debug, Clone, PartialEq, Eq)]
enum DefTokenKind {
    Ident(String),
    String(String),
    Number(String),
    Bool(bool),
    Null,
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Eq,
    Colon,
    Comma,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DefToken {
    kind: DefTokenKind,
    line: usize,
    column: usize,
}

struct DefTokenizer<'a> {
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    source: &'a str,
    line: usize,
    column: usize,
}

impl<'a> DefTokenizer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.char_indices().peekable(),
            source,
            line: 1,
            column: 1,
        }
    }

    fn tokenize(mut self) -> Result<Vec<DefToken>, String> {
        let mut tokens = Vec::new();
        while let Some((_idx, ch)) = self.peek_char() {
            if ch.is_whitespace() {
                self.consume_char();
                continue;
            }
            if ch == '#' {
                self.consume_line_comment();
                continue;
            }
            if ch == '/' && self.peek_next_is('/') {
                self.consume_char();
                self.consume_char();
                self.consume_line_comment();
                continue;
            }

            let line = self.line;
            let column = self.column;
            let kind = match ch {
                '{' => {
                    self.consume_char();
                    DefTokenKind::LBrace
                }
                '}' => {
                    self.consume_char();
                    DefTokenKind::RBrace
                }
                '(' => {
                    self.consume_char();
                    DefTokenKind::LParen
                }
                ')' => {
                    self.consume_char();
                    DefTokenKind::RParen
                }
                '[' => {
                    self.consume_char();
                    DefTokenKind::LBracket
                }
                ']' => {
                    self.consume_char();
                    DefTokenKind::RBracket
                }
                '=' => {
                    self.consume_char();
                    DefTokenKind::Eq
                }
                ':' => {
                    self.consume_char();
                    DefTokenKind::Colon
                }
                ',' => {
                    self.consume_char();
                    DefTokenKind::Comma
                }
                '"' => DefTokenKind::String(self.parse_string()?),
                '-' | '+' | '0'..='9' => DefTokenKind::Number(self.parse_number()),
                _ if is_ident_start(ch) => {
                    let ident = self.parse_ident();
                    match ident.as_str() {
                        "true" => DefTokenKind::Bool(true),
                        "false" => DefTokenKind::Bool(false),
                        "null" => DefTokenKind::Null,
                        _ => DefTokenKind::Ident(ident),
                    }
                }
                _ => {
                    return Err(format!("Unexpected character '{}' at {}:{}", ch, line, column));
                }
            };
            tokens.push(DefToken { kind, line, column });
        }
        Ok(tokens)
    }

    fn peek_char(&mut self) -> Option<(usize, char)> {
        self.chars.peek().copied()
    }

    fn peek_next_is(&self, expected: char) -> bool {
        let mut iter = self.chars.clone();
        let Some((_idx, _curr)) = iter.next() else {
            return false;
        };
        let Some((_idx, next)) = iter.next() else {
            return false;
        };
        next == expected
    }

    fn consume_char(&mut self) -> Option<(usize, char)> {
        let next = self.chars.next();
        if let Some((_idx, ch)) = next {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        next
    }

    fn consume_line_comment(&mut self) {
        while let Some((_idx, ch)) = self.peek_char() {
            if ch == '\n' {
                break;
            }
            self.consume_char();
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        let Some((_idx, quote)) = self.consume_char() else {
            return Err("Unexpected EOF while parsing string".to_string());
        };
        if quote != '"' {
            return Err("Expected opening quote".to_string());
        }

        let mut output = String::new();
        while let Some((_idx, ch)) = self.consume_char() {
            match ch {
                '"' => return Ok(output),
                '\\' => {
                    let Some((_idx, escaped)) = self.consume_char() else {
                        return Err("Unexpected EOF in string escape".to_string());
                    };
                    let resolved = match escaped {
                        '"' => '"',
                        '\\' => '\\',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        other => other,
                    };
                    output.push(resolved);
                }
                _ => output.push(ch),
            }
        }
        Err("Unterminated string literal".to_string())
    }

    fn parse_number(&mut self) -> String {
        let Some((start_idx, _start_ch)) = self.peek_char() else {
            return String::new();
        };
        self.consume_char();
        while let Some((_idx, ch)) = self.peek_char() {
            if ch.is_ascii_digit() || matches!(ch, '.' | '_' | 'e' | 'E' | '+' | '-') {
                self.consume_char();
                continue;
            }
            break;
        }
        let end_idx = self
            .chars
            .peek()
            .map(|(idx, _)| *idx)
            .unwrap_or(self.source.len());
        self.source[start_idx..end_idx].to_string()
    }

    fn parse_ident(&mut self) -> String {
        let Some((start_idx, _start_ch)) = self.peek_char() else {
            return String::new();
        };
        self.consume_char();
        while let Some((_idx, ch)) = self.peek_char() {
            if is_ident_continue(ch) {
                self.consume_char();
                continue;
            }
            break;
        }
        let end_idx = self
            .chars
            .peek()
            .map(|(idx, _)| *idx)
            .unwrap_or(self.source.len());
        self.source[start_idx..end_idx].to_string()
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '$'
}

fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit() || ch == '-' || ch == '.'
}

struct DefParser {
    tokens: Vec<DefToken>,
    pos: usize,
}

impl DefParser {
    fn new(tokens: Vec<DefToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse_root(&mut self) -> Result<DefValue, String> {
        let header_id = self.try_parse_header_id()?;
        let mut value = self.parse_value()?;
        if let Some(header_id) = header_id {
            if let DefValue::Map(entries) = &mut value {
                let has_id = entries.iter().any(|(key, _)| matches!(key, DefValue::String(key) if key == "id"));
                if !has_id {
                    entries.push((DefValue::String("id".to_string()), DefValue::String(header_id)));
                }
            }
        }
        Ok(value)
    }

    fn try_parse_header_id(&mut self) -> Result<Option<String>, String> {
        let Some(token) = self.tokens.get(self.pos) else {
            return Ok(None);
        };
        let DefTokenKind::Ident(_header) = &token.kind else {
            return Ok(None);
        };

        let Some(id_token) = self.tokens.get(self.pos + 1) else {
            return Ok(None);
        };
        match &id_token.kind {
            DefTokenKind::Ident(id) | DefTokenKind::String(id) => {
                if !matches!(self.tokens.get(self.pos + 2).map(|token| &token.kind), Some(DefTokenKind::LBrace)) {
                    return Ok(None);
                }
                self.pos += 2;
                Ok(Some(id.clone()))
            }
            _ => Ok(None),
        }
    }

    fn parse_value(&mut self) -> Result<DefValue, String> {
        let Some(token) = self.tokens.get(self.pos) else {
            return Err("Unexpected EOF while parsing value".to_string());
        };
        match &token.kind {
            DefTokenKind::String(value) => {
                self.pos += 1;
                Ok(DefValue::String(value.clone()))
            }
            DefTokenKind::Ident(value) => {
                if self.peek_next_is_object_start() {
                    self.pos += 1;
                    self.parse_value()
                } else {
                    self.pos += 1;
                    Ok(DefValue::String(value.clone()))
                }
            }
            DefTokenKind::Bool(value) => {
                self.pos += 1;
                Ok(DefValue::Bool(*value))
            }
            DefTokenKind::Null => {
                self.pos += 1;
                Ok(DefValue::Option(None))
            }
            DefTokenKind::Number(value) => {
                self.pos += 1;
                let cleaned = value.replace('_', "");
                if cleaned.contains('.') || cleaned.contains('e') || cleaned.contains('E') {
                    let parsed = cleaned.parse::<f64>().map_err(|_| {
                        format!("Invalid float '{}' at {}:{}", value, token.line, token.column)
                    })?;
                    return Ok(DefValue::F64(parsed));
                }
                if cleaned.starts_with('-') {
                    let parsed = cleaned.parse::<i64>().map_err(|_| {
                        format!("Invalid integer '{}' at {}:{}", value, token.line, token.column)
                    })?;
                    return Ok(DefValue::I64(parsed));
                }
                if let Ok(parsed) = cleaned.parse::<u64>() {
                    return Ok(DefValue::U64(parsed));
                }
                let parsed = cleaned.parse::<i64>().map_err(|_| {
                    format!("Invalid integer '{}' at {}:{}", value, token.line, token.column)
                })?;
                Ok(DefValue::I64(parsed))
            }
            DefTokenKind::LBrace => self.parse_brace_map(),
            DefTokenKind::LParen => self.parse_paren_value(),
            DefTokenKind::LBracket => self.parse_list(),
            other => Err(format!(
                "Unexpected token {:?} while parsing value at {}:{}",
                other, token.line, token.column
            )),
        }
    }

    fn parse_brace_map(&mut self) -> Result<DefValue, String> {
        self.expect_kind(DefTokenKind::LBrace)?;
        let mut entries = Vec::new();
        while !self.peek_kind(&DefTokenKind::RBrace) {
            let key = self.expect_key()?;
            self.expect_separator()?;
            let value = self.parse_value()?;
            entries.push((DefValue::String(key), value));
            self.consume_if_kind(&DefTokenKind::Comma);
        }
        self.expect_kind(DefTokenKind::RBrace)?;
        Ok(DefValue::Map(entries))
    }

    fn parse_paren_value(&mut self) -> Result<DefValue, String> {
        self.expect_kind(DefTokenKind::LParen)?;
        if self.peek_kind(&DefTokenKind::RParen) {
            self.pos += 1;
            return Ok(DefValue::Seq(Vec::new()));
        }

        if self.peek_is_key_like() && self.peek_separator_after_key() {
            let mut entries = Vec::new();
            while !self.peek_kind(&DefTokenKind::RParen) {
                let key = self.expect_key()?;
                self.expect_separator()?;
                let value = self.parse_value()?;
                entries.push((DefValue::String(key), value));
                self.consume_if_kind(&DefTokenKind::Comma);
            }
            self.expect_kind(DefTokenKind::RParen)?;
            return Ok(DefValue::Map(entries));
        }

        let mut values = Vec::new();
        while !self.peek_kind(&DefTokenKind::RParen) {
            values.push(self.parse_value()?);
            self.consume_if_kind(&DefTokenKind::Comma);
        }
        self.expect_kind(DefTokenKind::RParen)?;
        Ok(DefValue::Seq(values))
    }

    fn parse_list(&mut self) -> Result<DefValue, String> {
        self.expect_kind(DefTokenKind::LBracket)?;
        let mut values = Vec::new();
        while !self.peek_kind(&DefTokenKind::RBracket) {
            values.push(self.parse_value()?);
            self.consume_if_kind(&DefTokenKind::Comma);
        }
        self.expect_kind(DefTokenKind::RBracket)?;
        Ok(DefValue::Seq(values))
    }

    fn expect_key(&mut self) -> Result<String, String> {
        let Some(token) = self.tokens.get(self.pos) else {
            return Err("Unexpected EOF while parsing key".to_string());
        };
        match &token.kind {
            DefTokenKind::Ident(value) | DefTokenKind::String(value) => {
                self.pos += 1;
                Ok(value.clone())
            }
            _ => Err(format!(
                "Expected identifier/string key at {}:{}, found {:?}",
                token.line, token.column, token.kind
            )),
        }
    }

    fn expect_separator(&mut self) -> Result<(), String> {
        if self.consume_if_kind(&DefTokenKind::Eq) || self.consume_if_kind(&DefTokenKind::Colon) {
            return Ok(());
        }
        let Some(token) = self.tokens.get(self.pos) else {
            return Err("Unexpected EOF while parsing separator".to_string());
        };
        Err(format!(
            "Expected ':' or '=' at {}:{}, found {:?}",
            token.line, token.column, token.kind
        ))
    }

    fn consume_if_kind(&mut self, expected: &DefTokenKind) -> bool {
        if self.peek_kind(expected) {
            self.pos += 1;
            return true;
        }
        false
    }

    fn peek_kind(&self, expected: &DefTokenKind) -> bool {
        let Some(token) = self.tokens.get(self.pos) else {
            return false;
        };
        std::mem::discriminant(&token.kind) == std::mem::discriminant(expected)
    }

    fn peek_is_key_like(&self) -> bool {
        matches!(self.tokens.get(self.pos).map(|token| &token.kind), Some(DefTokenKind::Ident(_)) | Some(DefTokenKind::String(_)))
    }

    fn peek_separator_after_key(&self) -> bool {
        matches!(
            self.tokens.get(self.pos + 1).map(|token| &token.kind),
            Some(DefTokenKind::Eq) | Some(DefTokenKind::Colon)
        )
    }

    fn peek_next_is_object_start(&self) -> bool {
        matches!(
            self.tokens.get(self.pos + 1).map(|token| &token.kind),
            Some(DefTokenKind::LBrace) | Some(DefTokenKind::LParen)
        )
    }

    fn expect_kind(&mut self, expected: DefTokenKind) -> Result<(), String> {
        if self.peek_kind(&expected) {
            self.pos += 1;
            return Ok(());
        }
        let Some(token) = self.tokens.get(self.pos) else {
            return Err("Unexpected EOF while parsing token".to_string());
        };
        Err(format!(
            "Expected {:?} at {}:{}, found {:?}",
            expected, token.line, token.column, token.kind
        ))
    }
}

fn parse_root_def_value(content: &str) -> Result<DefValue, String> {
    let tokens = DefTokenizer::new(content).tokenize()?;
    let mut parser = DefParser::new(tokens);
    parser.parse_root()
}

pub(crate) fn parse_def_value(content: &str) -> Result<DefValue, String> {
    parse_root_def_value(content)
}

pub(crate) fn def_value_to_map(def_value: DefValue) -> Result<HashMap<String, DefValue>, String> {
    let DefValue::Map(entries) = def_value else {
        return Err("Expected map/object at root".to_string());
    };
    let mut out = HashMap::with_capacity(entries.len());
    for (key, value) in entries {
        let DefValue::String(key) = key else {
            return Err("Expected string keys in object map".to_string());
        };
        out.insert(key, value);
    }
    Ok(out)
}

pub(crate) fn parse_typed_def<T: serde::de::DeserializeOwned>(content: &str) -> Result<T, String> {
    let def_value = parse_root_def_value(content)?;
    T::deserialize(def_value).map_err(|err| err.to_string())
}
