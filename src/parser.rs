use crate::domain::{EventKind, EventOrigin, TokenUsage, UsageEvent};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

const TIMESTAMP_PATHS: &[&[&str]] = &[
    &["timestamp"],
    &["created_at"],
    &["createdAt"],
    &["time"],
    &["event_time"],
    &["eventTime"],
    &["meta", "timestamp"],
    &["meta", "created_at"],
    &["meta", "createdAt"],
    &["message", "timestamp"],
    &["message", "created_at"],
    &["message", "createdAt"],
];

const EVENT_TYPE_PATHS: &[&[&str]] = &[
    &["type"],
    &["event"],
    &["event_type"],
    &["eventType"],
    &["message", "type"],
    &["message", "event"],
];

const SESSION_ID_PATHS: &[&[&str]] = &[
    &["session_id"],
    &["sessionId"],
    &["session", "id"],
    &["message", "session_id"],
    &["message", "sessionId"],
    &["metadata", "session_id"],
    &["metadata", "sessionId"],
];

const PROJECT_PATHS: &[&[&str]] = &[
    &["project"],
    &["project_name"],
    &["projectName"],
    &["project_path"],
    &["projectPath"],
    &["cwd"],
    &["session", "project"],
    &["session", "project_name"],
    &["metadata", "project"],
    &["metadata", "project_name"],
    &["metadata", "projectName"],
];

const MODEL_PATHS: &[&[&str]] = &[
    &["model"],
    &["model_name"],
    &["modelName"],
    &["message", "model"],
    &["message", "model_name"],
    &["usage", "model"],
    &["metadata", "model"],
];

const INPUT_TOKEN_PATHS: &[&[&str]] = &[
    &["input_tokens"],
    &["inputTokens"],
    &["prompt_tokens"],
    &["promptTokens"],
    &["usage", "input_tokens"],
    &["usage", "inputTokens"],
    &["usage", "prompt_tokens"],
    &["usage", "promptTokens"],
    &["message", "usage", "input_tokens"],
    &["message", "usage", "inputTokens"],
    &["message", "usage", "prompt_tokens"],
    &["response", "usage", "input_tokens"],
    &["response", "usage", "prompt_tokens"],
];

const OUTPUT_TOKEN_PATHS: &[&[&str]] = &[
    &["output_tokens"],
    &["outputTokens"],
    &["completion_tokens"],
    &["completionTokens"],
    &["usage", "output_tokens"],
    &["usage", "outputTokens"],
    &["usage", "completion_tokens"],
    &["usage", "completionTokens"],
    &["message", "usage", "output_tokens"],
    &["message", "usage", "outputTokens"],
    &["message", "usage", "completion_tokens"],
    &["response", "usage", "output_tokens"],
    &["response", "usage", "completion_tokens"],
];

const CACHE_CREATION_TOKEN_PATHS: &[&[&str]] = &[
    &["cache_creation_input_tokens"],
    &["cacheCreationInputTokens"],
    &["cache_creation_tokens"],
    &["usage", "cache_creation_input_tokens"],
    &["usage", "cacheCreationInputTokens"],
    &["usage", "cache_creation_tokens"],
    &["message", "usage", "cache_creation_input_tokens"],
    &["response", "usage", "cache_creation_input_tokens"],
];

const CACHE_READ_TOKEN_PATHS: &[&[&str]] = &[
    &["cache_read_input_tokens"],
    &["cacheReadInputTokens"],
    &["cache_read_tokens"],
    &["usage", "cache_read_input_tokens"],
    &["usage", "cacheReadInputTokens"],
    &["usage", "cache_read_tokens"],
    &["message", "usage", "cache_read_input_tokens"],
    &["response", "usage", "cache_read_input_tokens"],
];

const TOTAL_TOKEN_PATHS: &[&[&str]] = &[
    &["total_tokens"],
    &["totalTokens"],
    &["usage", "total_tokens"],
    &["usage", "totalTokens"],
    &["message", "usage", "total_tokens"],
    &["response", "usage", "total_tokens"],
];

const COST_PATHS: &[&[&str]] = &[
    &["cost_usd"],
    &["costUsd"],
    &["total_cost_usd"],
    &["totalCostUsd"],
    &["usage", "cost_usd"],
    &["usage", "total_cost_usd"],
    &["message", "usage", "cost_usd"],
    &["response", "usage", "cost_usd"],
];

const TIMESTAMP_KEYS: &[&str] = &[
    "timestamp",
    "created_at",
    "createdat",
    "time",
    "event_time",
    "eventtime",
];
const EVENT_TYPE_KEYS: &[&str] = &["type", "event", "event_type", "eventtype"];
const SESSION_ID_KEYS: &[&str] = &["session_id", "sessionid"];
const PROJECT_KEYS: &[&str] = &[
    "project",
    "project_name",
    "projectname",
    "project_path",
    "projectpath",
    "cwd",
];
const MODEL_KEYS: &[&str] = &["model", "model_name", "modelname"];

const INPUT_TOKEN_KEYS: &[&str] = &[
    "input_tokens",
    "inputtokens",
    "prompt_tokens",
    "prompttokens",
];
const OUTPUT_TOKEN_KEYS: &[&str] = &[
    "output_tokens",
    "outputtokens",
    "completion_tokens",
    "completiontokens",
];
const CACHE_CREATION_TOKEN_KEYS: &[&str] = &[
    "cache_creation_input_tokens",
    "cachecreationinputtokens",
    "cache_creation_tokens",
];
const CACHE_READ_TOKEN_KEYS: &[&str] = &[
    "cache_read_input_tokens",
    "cachereadinputtokens",
    "cache_read_tokens",
];
const TOTAL_TOKEN_KEYS: &[&str] = &["total_tokens", "totaltokens"];
const COST_KEYS: &[&str] = &[
    "cost_usd",
    "costusd",
    "total_cost_usd",
    "totalcostusd",
    "usd_cost",
];

#[derive(Debug, Clone, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(JsonNumber),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

impl JsonValue {
    fn as_object(&self) -> Option<&BTreeMap<String, JsonValue>> {
        match self {
            Self::Object(object) => Some(object),
            _ => None,
        }
    }

    fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum JsonNumber {
    Integer(i128),
    Float(f64),
}

impl JsonNumber {
    fn as_i128(&self) -> Option<i128> {
        match self {
            Self::Integer(value) => Some(*value),
            Self::Float(_) => None,
        }
    }

    fn as_f64(&self) -> f64 {
        match self {
            Self::Integer(value) => *value as f64,
            Self::Float(value) => *value,
        }
    }
}

impl std::fmt::Display for JsonNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{value}"),
            Self::Float(value) => write!(f, "{value}"),
        }
    }
}

struct JsonParser<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, index: 0 }
    }

    fn parse(mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let value = self.parse_value()?;
        self.skip_whitespace();
        if self.peek_char().is_some() {
            return Err(self.error("unexpected trailing characters"));
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let Some(ch) = self.peek_char() else {
            return Err(self.error("unexpected end of input"));
        };

        match ch {
            '{' => self.parse_object(),
            '[' => self.parse_array(),
            '"' => self.parse_string().map(JsonValue::String),
            't' => {
                self.consume_literal("true")?;
                Ok(JsonValue::Bool(true))
            }
            'f' => {
                self.consume_literal("false")?;
                Ok(JsonValue::Bool(false))
            }
            'n' => {
                self.consume_literal("null")?;
                Ok(JsonValue::Null)
            }
            '-' | '0'..='9' => self.parse_number().map(JsonValue::Number),
            _ => Err(self.error("unexpected value token")),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.expect_char('{')?;
        self.skip_whitespace();

        let mut object = BTreeMap::new();
        if self.peek_char() == Some('}') {
            self.next_char();
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.expect_char(':')?;
            self.skip_whitespace();
            let value = self.parse_value()?;
            object.insert(key, value);

            self.skip_whitespace();
            match self.next_char() {
                Some(',') => continue,
                Some('}') => break,
                _ => return Err(self.error("expected ',' or '}' in object")),
            }
        }

        Ok(JsonValue::Object(object))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.expect_char('[')?;
        self.skip_whitespace();

        let mut values = Vec::new();
        if self.peek_char() == Some(']') {
            self.next_char();
            return Ok(JsonValue::Array(values));
        }

        loop {
            values.push(self.parse_value()?);
            self.skip_whitespace();
            match self.next_char() {
                Some(',') => continue,
                Some(']') => break,
                _ => return Err(self.error("expected ',' or ']' in array")),
            }
        }

        Ok(JsonValue::Array(values))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect_char('"')?;
        let mut out = String::new();

        loop {
            let Some(ch) = self.next_char() else {
                return Err(self.error("unterminated string"));
            };

            match ch {
                '"' => break,
                '\\' => {
                    let Some(escaped) = self.next_char() else {
                        return Err(self.error("unterminated escape sequence"));
                    };
                    match escaped {
                        '"' | '\\' | '/' => out.push(escaped),
                        'b' => out.push('\u{0008}'),
                        'f' => out.push('\u{000C}'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => {
                            let codepoint = self.parse_unicode_escape()?;
                            let Some(ch) = char::from_u32(codepoint) else {
                                return Err(self.error("invalid unicode escape"));
                            };
                            out.push(ch);
                        }
                        _ => return Err(self.error("invalid escape sequence")),
                    }
                }
                c if c <= '\u{001F}' => return Err(self.error("control character in string")),
                _ => out.push(ch),
            }
        }

        Ok(out)
    }

    fn parse_unicode_escape(&mut self) -> Result<u32, String> {
        let high = self.parse_hex_u16()?;
        if !(0xD800..=0xDBFF).contains(&high) {
            return Ok(high as u32);
        }

        let saved = self.index;
        if self.next_char() != Some('\\') || self.next_char() != Some('u') {
            self.index = saved;
            return Err(self.error("invalid surrogate pair"));
        }

        let low = self.parse_hex_u16()?;
        if !(0xDC00..=0xDFFF).contains(&low) {
            return Err(self.error("invalid surrogate pair"));
        }

        let high_ten = (high as u32) - 0xD800;
        let low_ten = (low as u32) - 0xDC00;
        Ok(0x10000 + ((high_ten << 10) | low_ten))
    }

    fn parse_hex_u16(&mut self) -> Result<u16, String> {
        let mut value: u16 = 0;
        for _ in 0..4 {
            let Some(ch) = self.next_char() else {
                return Err(self.error("unexpected end of unicode escape"));
            };
            let digit = ch
                .to_digit(16)
                .ok_or_else(|| self.error("invalid unicode escape"))?;
            value = value
                .checked_mul(16)
                .and_then(|acc| acc.checked_add(digit as u16))
                .ok_or_else(|| self.error("unicode escape overflow"))?;
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<JsonNumber, String> {
        let start = self.index;

        if self.peek_char() == Some('-') {
            self.next_char();
        }

        match self.peek_char() {
            Some('0') => {
                self.next_char();
            }
            Some('1'..='9') => {
                self.next_char();
                while matches!(self.peek_char(), Some('0'..='9')) {
                    self.next_char();
                }
            }
            _ => return Err(self.error("invalid number")),
        }

        let mut is_float = false;
        if self.peek_char() == Some('.') {
            is_float = true;
            self.next_char();
            if !matches!(self.peek_char(), Some('0'..='9')) {
                return Err(self.error("invalid number"));
            }
            while matches!(self.peek_char(), Some('0'..='9')) {
                self.next_char();
            }
        }

        if matches!(self.peek_char(), Some('e' | 'E')) {
            is_float = true;
            self.next_char();
            if matches!(self.peek_char(), Some('+' | '-')) {
                self.next_char();
            }
            if !matches!(self.peek_char(), Some('0'..='9')) {
                return Err(self.error("invalid number exponent"));
            }
            while matches!(self.peek_char(), Some('0'..='9')) {
                self.next_char();
            }
        }

        let slice = &self.input[start..self.index];
        if is_float {
            let value = slice
                .parse::<f64>()
                .map_err(|_| self.error("invalid floating-point number"))?;
            Ok(JsonNumber::Float(value))
        } else {
            let value = slice
                .parse::<i128>()
                .map_err(|_| self.error("invalid integer number"))?;
            Ok(JsonNumber::Integer(value))
        }
    }

    fn consume_literal(&mut self, literal: &str) -> Result<(), String> {
        if self.input[self.index..].starts_with(literal) {
            self.index += literal.len();
            Ok(())
        } else {
            Err(self.error("invalid literal"))
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), String> {
        match self.next_char() {
            Some(ch) if ch == expected => Ok(()),
            _ => Err(self.error(&format!("expected '{expected}'"))),
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_char(), Some(ch) if ch.is_ascii_whitespace()) {
            self.next_char();
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.index..].chars().next()
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.index += ch.len_utf8();
        Some(ch)
    }

    fn error(&self, message: &str) -> String {
        format!("{message} at column {}", self.index + 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseWarning {
    pub file: PathBuf,
    pub line_number: Option<usize>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParseResult {
    pub events: Vec<UsageEvent>,
    pub warnings: Vec<ParseWarning>,
}

impl ParseResult {
    fn extend(&mut self, mut other: Self) {
        self.events.append(&mut other.events);
        self.warnings.append(&mut other.warnings);
    }
}

#[must_use]
pub fn parse_jsonl_files(files: &[PathBuf]) -> ParseResult {
    let unique_files = dedupe_paths(files.iter().cloned());
    let mut output = ParseResult::default();

    for file in unique_files {
        output.extend(parse_jsonl_file(&file));
    }

    output
}

#[must_use]
pub fn parse_jsonl_file(path: &Path) -> ParseResult {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            return ParseResult {
                events: Vec::new(),
                warnings: vec![ParseWarning {
                    file: path.to_path_buf(),
                    line_number: None,
                    message: format!("failed to open file: {error}"),
                }],
            };
        }
    };

    parse_jsonl_reader(BufReader::new(file), path)
}

fn parse_jsonl_reader<R: BufRead>(reader: R, path: &Path) -> ParseResult {
    let mut output = ParseResult::default();

    for (index, line_result) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = match line_result {
            Ok(line) => line,
            Err(error) => {
                output.warnings.push(ParseWarning {
                    file: path.to_path_buf(),
                    line_number: Some(line_number),
                    message: format!("failed to read line: {error}"),
                });
                continue;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let raw_value = match JsonParser::new(trimmed).parse() {
            Ok(value) => value,
            Err(error) => {
                output.warnings.push(ParseWarning {
                    file: path.to_path_buf(),
                    line_number: Some(line_number),
                    message: format!("invalid JSON line: {error}"),
                });
                continue;
            }
        };

        if !raw_value.is_object() {
            output.warnings.push(ParseWarning {
                file: path.to_path_buf(),
                line_number: Some(line_number),
                message: "root JSON value must be an object".to_owned(),
            });
            continue;
        }

        match normalize_event(&raw_value, path, line_number) {
            Ok(event) => output.events.push(event),
            Err(message) => output.warnings.push(ParseWarning {
                file: path.to_path_buf(),
                line_number: Some(line_number),
                message,
            }),
        }
    }

    output
}

fn normalize_event(
    value: &JsonValue,
    file: &Path,
    line_number: usize,
) -> Result<UsageEvent, String> {
    let Some(occurred_at_unix_ms) = extract_timestamp_ms(value) else {
        return Err("missing parseable timestamp".to_owned());
    };

    let event_kind =
        EventKind::from_raw(extract_string(value, EVENT_TYPE_PATHS, EVENT_TYPE_KEYS).as_deref());

    let input_tokens = extract_u64(value, INPUT_TOKEN_PATHS, INPUT_TOKEN_KEYS).unwrap_or(0);
    let output_tokens = extract_u64(value, OUTPUT_TOKEN_PATHS, OUTPUT_TOKEN_KEYS).unwrap_or(0);
    let cache_creation_input_tokens =
        extract_u64(value, CACHE_CREATION_TOKEN_PATHS, CACHE_CREATION_TOKEN_KEYS).unwrap_or(0);
    let cache_read_input_tokens =
        extract_u64(value, CACHE_READ_TOKEN_PATHS, CACHE_READ_TOKEN_KEYS).unwrap_or(0);
    let total_tokens = extract_u64(value, TOTAL_TOKEN_PATHS, TOTAL_TOKEN_KEYS);

    let usage = TokenUsage::new(
        input_tokens,
        output_tokens,
        cache_creation_input_tokens,
        cache_read_input_tokens,
        total_tokens,
    );

    Ok(UsageEvent {
        origin: EventOrigin {
            file: file.to_path_buf(),
            line_number,
        },
        occurred_at_unix_ms,
        event_kind,
        session_id: extract_string(value, SESSION_ID_PATHS, SESSION_ID_KEYS),
        project: extract_string(value, PROJECT_PATHS, PROJECT_KEYS),
        model: extract_string(value, MODEL_PATHS, MODEL_KEYS),
        usage,
        raw_cost_usd: extract_f64(value, COST_PATHS, COST_KEYS),
    })
}

fn extract_timestamp_ms(value: &JsonValue) -> Option<i64> {
    for path in TIMESTAMP_PATHS {
        if let Some(raw) = value_at_path(value, path)
            && let Some(parsed) = parse_timestamp_value(raw)
        {
            return Some(parsed);
        }
    }

    find_first_value_by_key(value, TIMESTAMP_KEYS).and_then(parse_timestamp_value)
}

fn parse_timestamp_value(value: &JsonValue) -> Option<i64> {
    match value {
        JsonValue::Number(number) => {
            if let Some(int) = number.as_i128() {
                return normalize_epoch(int);
            }
            normalize_epoch_float(number.as_f64())
        }
        JsonValue::String(text) => parse_timestamp_text(text),
        _ => None,
    }
}

fn parse_timestamp_text(text: &str) -> Option<i64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(parsed) = trimmed.parse::<i128>() {
        return normalize_epoch(parsed);
    }

    if let Ok(parsed) = trimmed.parse::<f64>()
        && let Some(normalized) = normalize_epoch_float(parsed)
    {
        return Some(normalized);
    }

    parse_rfc3339_to_millis(trimmed)
}

fn normalize_epoch(raw: i128) -> Option<i64> {
    let abs = raw.abs();
    let normalized = if abs >= 1_000_000_000_000_000_000 {
        raw / 1_000_000
    } else if abs >= 1_000_000_000_000_000 {
        raw / 1_000
    } else if abs >= 1_000_000_000_000 {
        raw
    } else {
        raw.saturating_mul(1_000)
    };

    i64::try_from(normalized).ok()
}

fn normalize_epoch_float(raw: f64) -> Option<i64> {
    if !raw.is_finite() {
        return None;
    }

    let abs = raw.abs();
    let normalized = if abs >= 1e18 {
        raw / 1_000_000.0
    } else if abs >= 1e15 {
        raw / 1_000.0
    } else if abs >= 1e12 {
        raw
    } else {
        raw * 1_000.0
    };

    if normalized < i64::MIN as f64 || normalized > i64::MAX as f64 {
        return None;
    }

    Some(normalized.round() as i64)
}

fn parse_rfc3339_to_millis(text: &str) -> Option<i64> {
    if text.len() < 20 {
        return None;
    }

    let year = parse_int(text.get(0..4)?)?;
    ensure_char(text, 4, '-')?;
    let month = parse_int(text.get(5..7)?)?;
    ensure_char(text, 7, '-')?;
    let day = parse_int(text.get(8..10)?)?;

    let separator = text.as_bytes().get(10).copied()? as char;
    if separator != 'T' && separator != 't' && separator != ' ' {
        return None;
    }

    let hour = parse_int(text.get(11..13)?)?;
    ensure_char(text, 13, ':')?;
    let minute = parse_int(text.get(14..16)?)?;
    ensure_char(text, 16, ':')?;
    let second = parse_int(text.get(17..19)?)?;

    if !(1..=12).contains(&month) {
        return None;
    }
    let max_day = days_in_month(year, month)?;
    if !(1..=max_day).contains(&day) {
        return None;
    }
    if hour > 23 || minute > 59 || second > 60 {
        return None;
    }

    let mut index = 19;
    let bytes = text.as_bytes();
    let mut millis = 0i64;

    if bytes.get(index).copied() == Some(b'.') {
        index += 1;
        let fraction_start = index;
        while matches!(bytes.get(index), Some(b'0'..=b'9')) {
            index += 1;
        }

        if index == fraction_start {
            return None;
        }

        let fraction = &text[fraction_start..index];
        let mut fraction_millis = 0i64;
        for (i, ch) in fraction.chars().enumerate() {
            if i >= 3 {
                break;
            }
            let digit = (ch as u8 - b'0') as i64;
            fraction_millis = fraction_millis.saturating_mul(10).saturating_add(digit);
        }
        let digits = fraction.len().min(3);
        for _ in digits..3 {
            fraction_millis *= 10;
        }
        millis = fraction_millis;
    }

    let offset_seconds = match bytes.get(index).copied().map(char::from) {
        Some('Z') | Some('z') => {
            index += 1;
            0i64
        }
        Some('+') | Some('-') => {
            let sign = if bytes[index] == b'+' { 1i64 } else { -1i64 };
            index += 1;
            let offset_hour = parse_int(text.get(index..index + 2)?)?;
            index += 2;
            ensure_char(text, index, ':')?;
            index += 1;
            let offset_minute = parse_int(text.get(index..index + 2)?)?;
            index += 2;

            if offset_hour > 23 || offset_minute > 59 {
                return None;
            }

            sign * (offset_hour as i64 * 3600 + offset_minute as i64 * 60)
        }
        _ => return None,
    };

    if index != text.len() {
        return None;
    }

    let days = days_from_civil(year as i64, month as i64, day as i64);
    let seconds = days
        .saturating_mul(86_400)
        .saturating_add(hour as i64 * 3_600)
        .saturating_add(minute as i64 * 60)
        .saturating_add(second as i64)
        .saturating_sub(offset_seconds);

    seconds
        .checked_mul(1_000)
        .and_then(|base| base.checked_add(millis))
}

fn parse_int(value: &str) -> Option<u32> {
    value.parse::<u32>().ok()
}

fn ensure_char(text: &str, index: usize, expected: char) -> Option<()> {
    (text.as_bytes().get(index).copied().map(char::from) == Some(expected)).then_some(())
}

fn days_in_month(year: u32, month: u32) -> Option<u32> {
    let leap = is_leap_year(year as i64);
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 if leap => Some(29),
        2 => Some(28),
        _ => None,
    }
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let mp = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn extract_string(value: &JsonValue, paths: &[&[&str]], keys: &[&str]) -> Option<String> {
    for path in paths {
        if let Some(candidate) = value_at_path(value, path)
            && let Some(normalized) = value_to_non_empty_string(candidate)
        {
            return Some(normalized);
        }
    }

    find_first_value_by_key(value, keys).and_then(value_to_non_empty_string)
}

fn extract_u64(value: &JsonValue, paths: &[&[&str]], keys: &[&str]) -> Option<u64> {
    for path in paths {
        if let Some(candidate) = value_at_path(value, path)
            && let Some(normalized) = value_to_u64(candidate)
        {
            return Some(normalized);
        }
    }

    find_first_value_by_key(value, keys).and_then(value_to_u64)
}

fn extract_f64(value: &JsonValue, paths: &[&[&str]], keys: &[&str]) -> Option<f64> {
    for path in paths {
        if let Some(candidate) = value_at_path(value, path)
            && let Some(normalized) = value_to_f64(candidate)
        {
            return Some(normalized);
        }
    }

    find_first_value_by_key(value, keys).and_then(value_to_f64)
}

fn value_to_non_empty_string(value: &JsonValue) -> Option<String> {
    let text = match value {
        JsonValue::String(text) => text.trim().to_owned(),
        JsonValue::Number(number) => number.to_string(),
        JsonValue::Bool(flag) => flag.to_string(),
        _ => return None,
    };

    if text.is_empty() { None } else { Some(text) }
}

fn value_to_u64(value: &JsonValue) -> Option<u64> {
    match value {
        JsonValue::Number(number) => {
            if let Some(raw) = number.as_i128() {
                return u64::try_from(raw).ok();
            }

            let raw = number.as_f64();
            if raw.is_finite() && raw >= 0.0 {
                return Some(raw.round() as u64);
            }
            None
        }
        JsonValue::String(text) => text.trim().parse::<u64>().ok(),
        _ => None,
    }
}

fn value_to_f64(value: &JsonValue) -> Option<f64> {
    let parsed = match value {
        JsonValue::Number(number) => Some(number.as_f64()),
        JsonValue::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    };

    parsed.filter(|raw| raw.is_finite() && *raw >= 0.0)
}

fn value_at_path<'a>(value: &'a JsonValue, path: &[&str]) -> Option<&'a JsonValue> {
    let mut current = value;
    for segment in path {
        current = current.as_object()?.get(*segment)?;
    }
    Some(current)
}

fn find_first_value_by_key<'a>(value: &'a JsonValue, keys: &[&str]) -> Option<&'a JsonValue> {
    match value {
        JsonValue::Object(object) => {
            for (key, child) in object {
                if key_matches(key, keys) && !child.is_null() {
                    return Some(child);
                }
            }

            for child in object.values() {
                if let Some(found) = find_first_value_by_key(child, keys) {
                    return Some(found);
                }
            }

            None
        }
        JsonValue::Array(items) => {
            for child in items {
                if let Some(found) = find_first_value_by_key(child, keys) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

fn key_matches(key: &str, keys: &[&str]) -> bool {
    keys.iter()
        .any(|candidate| key.eq_ignore_ascii_case(candidate))
}

fn dedupe_paths(paths: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    let mut deduped = BTreeSet::new();
    for path in paths {
        if path.as_os_str().is_empty() {
            continue;
        }
        deduped.insert(path);
    }
    deduped.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::EventKind;
    use std::fs::{create_dir_all, remove_dir_all, write};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn normalizes_token_usage_and_common_fields_from_mixed_shapes() {
        let test_dir = TestDir::new();
        let file = test_dir.path().join("mixed.jsonl");
        let content = concat!(
            "{\"timestamp\":1700000000,\"type\":\"assistant_message\",\"session_id\":\"s1\",\"project\":\"alpha\",\"model\":\"claude-a\",\"usage\":{\"input_tokens\":10,\"output_tokens\":5,\"cache_creation_input_tokens\":2,\"cache_read_input_tokens\":3}}\n",
            "{\"createdAt\":\"1970-01-01T00:00:02Z\",\"event\":\"tool_result\",\"session\":{\"id\":\"s2\"},\"project_name\":\"beta\",\"usage\":{\"prompt_tokens\":7,\"completion_tokens\":11,\"total_tokens\":99}}\n",
            "{\"timestamp\":1700000000123456,\"type\":\"custom_event\",\"message\":{\"usage\":{\"inputTokens\":\"4\",\"outputTokens\":\"6\"},\"model\":\"claude-b\"},\"total_cost_usd\":\"0.123\"}\n"
        );

        write(&file, content).expect("failed to write fixture file");

        let parsed = parse_jsonl_file(&file);

        assert!(parsed.warnings.is_empty());
        assert_eq!(parsed.events.len(), 3);

        assert_eq!(parsed.events[0].occurred_at_unix_ms, 1_700_000_000_000);
        assert_eq!(parsed.events[0].event_kind, EventKind::Assistant);
        assert_eq!(parsed.events[0].session_id.as_deref(), Some("s1"));
        assert_eq!(parsed.events[0].project.as_deref(), Some("alpha"));
        assert_eq!(parsed.events[0].model.as_deref(), Some("claude-a"));
        assert_eq!(parsed.events[0].usage.input_tokens, 10);
        assert_eq!(parsed.events[0].usage.output_tokens, 5);
        assert_eq!(parsed.events[0].usage.cache_creation_input_tokens, 2);
        assert_eq!(parsed.events[0].usage.cache_read_input_tokens, 3);
        assert_eq!(parsed.events[0].usage.total_tokens, 20);

        assert_eq!(parsed.events[1].occurred_at_unix_ms, 2_000);
        assert_eq!(parsed.events[1].event_kind, EventKind::ToolResult);
        assert_eq!(parsed.events[1].session_id.as_deref(), Some("s2"));
        assert_eq!(parsed.events[1].project.as_deref(), Some("beta"));
        assert_eq!(parsed.events[1].usage.input_tokens, 7);
        assert_eq!(parsed.events[1].usage.output_tokens, 11);
        assert_eq!(parsed.events[1].usage.total_tokens, 99);

        assert_eq!(parsed.events[2].occurred_at_unix_ms, 1_700_000_000_123);
        assert_eq!(
            parsed.events[2].event_kind,
            EventKind::Unknown("custom_event".to_owned())
        );
        assert_eq!(parsed.events[2].model.as_deref(), Some("claude-b"));
        assert_eq!(parsed.events[2].usage.input_tokens, 4);
        assert_eq!(parsed.events[2].usage.output_tokens, 6);
        assert_eq!(parsed.events[2].usage.total_tokens, 10);
        assert_eq!(parsed.events[2].raw_cost_usd, Some(0.123));
    }

    #[test]
    fn records_line_warnings_and_keeps_parsing() {
        let test_dir = TestDir::new();
        let file = test_dir.path().join("warnings.jsonl");
        let content = concat!(
            "{\"timestamp\":1700000000,\"type\":\"assistant\",\"usage\":{\"input_tokens\":1,\"output_tokens\":2}}\n",
            "{\"timestamp\":\n",
            "[1,2,3]\n",
            "{\"type\":\"assistant\",\"usage\":{\"input_tokens\":1}}\n"
        );

        write(&file, content).expect("failed to write fixture file");

        let parsed = parse_jsonl_file(&file);

        assert_eq!(parsed.events.len(), 1);
        assert_eq!(parsed.warnings.len(), 3);
        assert!(
            parsed.warnings[0].message.contains("invalid JSON line"),
            "unexpected warning message: {}",
            parsed.warnings[0].message
        );
        assert_eq!(
            parsed.warnings[1].message,
            "root JSON value must be an object"
        );
        assert_eq!(parsed.warnings[2].message, "missing parseable timestamp");
        assert_eq!(parsed.warnings[0].line_number, Some(2));
        assert_eq!(parsed.warnings[1].line_number, Some(3));
        assert_eq!(parsed.warnings[2].line_number, Some(4));
    }

    #[test]
    fn parses_multiple_files_with_dedup_and_open_failures() {
        let test_dir = TestDir::new();
        let a = test_dir.path().join("a.jsonl");
        let b = test_dir.path().join("b.jsonl");
        let missing = test_dir.path().join("missing.jsonl");

        write(
            &a,
            "{\"timestamp\":1700000000,\"type\":\"assistant\",\"usage\":{\"input_tokens\":1}}\n",
        )
        .expect("failed to write file a");
        write(
            &b,
            "{\"timestamp\":1700000001,\"type\":\"user\",\"usage\":{\"input_tokens\":2}}\n",
        )
        .expect("failed to write file b");

        let parsed = parse_jsonl_files(&[missing.clone(), b.clone(), a.clone(), a.clone()]);

        assert_eq!(parsed.events.len(), 2);
        assert_eq!(parsed.warnings.len(), 1);
        assert_eq!(parsed.warnings[0].file, missing);
        assert!(parsed.warnings[0].line_number.is_none());
        assert!(parsed.warnings[0].message.contains("failed to open file"));

        assert_eq!(parsed.events[0].origin.file, a);
        assert_eq!(parsed.events[1].origin.file, b);
    }

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let mut path = std::env::temp_dir();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos();
            let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
            path.push(format!(
                "cusage-rs-parser-tests-{}-{timestamp}-{counter}",
                std::process::id()
            ));

            create_dir_all(&path).expect("failed to create test directory");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = remove_dir_all(&self.path);
        }
    }
}
