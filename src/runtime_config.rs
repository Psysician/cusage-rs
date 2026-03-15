use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const LOCAL_CONFIG_RELATIVE_PATH: &str = ".ccusage/ccusage.json";
const USER_CONFIG_RELATIVE_PATH: &str = ".config/claude/ccusage.json";
const LEGACY_CONFIG_RELATIVE_PATH: &str = ".claude/ccusage.json";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandConfigLayer {
    pub since: Option<String>,
    pub until: Option<String>,
    pub json: Option<bool>,
    pub breakdown: Option<bool>,
    pub compact: Option<bool>,
    pub instances: Option<bool>,
    pub project: Option<String>,
    pub timezone: Option<String>,
    pub locale: Option<String>,
    pub offline: Option<bool>,
}

impl CommandConfigLayer {
    pub fn merge_from(&mut self, other: &Self) {
        if other.since.is_some() {
            self.since = other.since.clone();
        }
        if other.until.is_some() {
            self.until = other.until.clone();
        }
        if other.json.is_some() {
            self.json = other.json;
        }
        if other.breakdown.is_some() {
            self.breakdown = other.breakdown;
        }
        if other.compact.is_some() {
            self.compact = other.compact;
        }
        if other.instances.is_some() {
            self.instances = other.instances;
        }
        if other.project.is_some() {
            self.project = other.project.clone();
        }
        if other.timezone.is_some() {
            self.timezone = other.timezone.clone();
        }
        if other.locale.is_some() {
            self.locale = other.locale.clone();
        }
        if other.offline.is_some() {
            self.offline = other.offline;
        }
    }
}

pub fn load_auto_config_layer(
    command: &str,
    cwd: &Path,
    home_dir: Option<&Path>,
) -> Result<CommandConfigLayer, String> {
    let mut layer = CommandConfigLayer::default();

    if let Some(home_dir) = home_dir {
        if let Some(legacy_layer) =
            load_layer_from_path(command, &home_dir.join(LEGACY_CONFIG_RELATIVE_PATH), false)?
        {
            layer.merge_from(&legacy_layer);
        }
        if let Some(user_layer) =
            load_layer_from_path(command, &home_dir.join(USER_CONFIG_RELATIVE_PATH), false)?
        {
            layer.merge_from(&user_layer);
        }
    }

    if let Some(local_layer) =
        load_layer_from_path(command, &cwd.join(LOCAL_CONFIG_RELATIVE_PATH), false)?
    {
        layer.merge_from(&local_layer);
    }

    Ok(layer)
}

pub fn load_custom_config_layer(
    command: &str,
    config_path: &Path,
) -> Result<CommandConfigLayer, String> {
    let Some(layer) = load_layer_from_path(command, config_path, true)? else {
        return Err(format!(
            "config file '{}' does not exist",
            config_path.display()
        ));
    };

    Ok(layer)
}

fn load_layer_from_path(
    command: &str,
    path: &Path,
    required: bool,
) -> Result<Option<CommandConfigLayer>, String> {
    if !path.exists() {
        if required {
            return Err(format!("config file '{}' does not exist", path.display()));
        }
        return Ok(None);
    }

    if !path.is_file() {
        return Err(format!("config path '{}' is not a file", path.display()));
    }

    let contents = fs::read_to_string(path)
        .map_err(|error| format!("failed to read config '{}': {error}", path.display()))?;
    let layer = load_document_layer(command, &contents, path)?;
    Ok(Some(layer))
}

fn load_document_layer(
    command: &str,
    contents: &str,
    path: &Path,
) -> Result<CommandConfigLayer, String> {
    let document = parse_config_document(contents)
        .map_err(|error| format!("invalid config '{}': {error}", path.display()))?;
    let mut layer = document.defaults;
    if let Some(command_layer) = document.commands.get(&normalize_key(command)) {
        layer.merge_from(command_layer);
    }
    Ok(layer)
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ConfigDocument {
    defaults: CommandConfigLayer,
    commands: BTreeMap<String, CommandConfigLayer>,
}

fn parse_config_document(input: &str) -> Result<ConfigDocument, String> {
    let parsed = JsonParser::new(input).parse()?;
    let root = parsed
        .as_object()
        .ok_or_else(|| "root JSON value must be an object".to_owned())?;

    let mut defaults = CommandConfigLayer::default();
    let mut commands = BTreeMap::new();

    for (key, value) in root {
        match normalize_key(key).as_str() {
            "defaults" => {
                let object = value
                    .as_object()
                    .ok_or_else(|| "'defaults' must be a JSON object".to_owned())?;
                defaults = parse_command_layer(object, "defaults")?;
            }
            "commands" => {
                let object = value
                    .as_object()
                    .ok_or_else(|| "'commands' must be a JSON object".to_owned())?;
                for (command_name, command_value) in object {
                    let command_object = command_value.as_object().ok_or_else(|| {
                        format!("'commands.{command_name}' must be a JSON object")
                    })?;
                    commands.insert(
                        normalize_key(command_name),
                        parse_command_layer(command_object, &format!("commands.{command_name}"))?,
                    );
                }
            }
            _ => {}
        }
    }

    Ok(ConfigDocument { defaults, commands })
}

fn parse_command_layer(
    object: &BTreeMap<String, JsonValue>,
    context: &str,
) -> Result<CommandConfigLayer, String> {
    let mut layer = CommandConfigLayer::default();

    for (key, value) in object {
        match normalize_key(key).as_str() {
            "since" => layer.since = parse_optional_string(value, context, key)?,
            "until" => layer.until = parse_optional_string(value, context, key)?,
            "json" => layer.json = parse_optional_bool(value, context, key)?,
            "breakdown" => layer.breakdown = parse_optional_bool(value, context, key)?,
            "compact" => layer.compact = parse_optional_bool(value, context, key)?,
            "instances" => layer.instances = parse_optional_bool(value, context, key)?,
            "project" => layer.project = parse_optional_string(value, context, key)?,
            "timezone" => layer.timezone = parse_optional_string(value, context, key)?,
            "locale" => layer.locale = parse_optional_string(value, context, key)?,
            "offline" => layer.offline = parse_optional_bool(value, context, key)?,
            _ => {}
        }
    }

    Ok(layer)
}

fn parse_optional_string(
    value: &JsonValue,
    context: &str,
    key: &str,
) -> Result<Option<String>, String> {
    match value {
        JsonValue::Null => Ok(None),
        JsonValue::String(raw) => Ok(normalized_optional_string(Some(raw))),
        _ => Err(format!("'{context}.{key}' must be a string or null")),
    }
}

fn parse_optional_bool(
    value: &JsonValue,
    context: &str,
    key: &str,
) -> Result<Option<bool>, String> {
    match value {
        JsonValue::Null => Ok(None),
        JsonValue::Bool(raw) => Ok(Some(*raw)),
        _ => Err(format!("'{context}.{key}' must be a boolean or null")),
    }
}

fn normalized_optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn normalize_key(key: &str) -> String {
    key.chars()
        .filter(|ch| *ch != '_' && *ch != '-')
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

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
}

#[derive(Debug, Clone, PartialEq)]
enum JsonNumber {
    Integer(i128),
    Float(f64),
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
                                return Err(self.error("invalid unicode scalar value"));
                            };
                            out.push(ch);
                        }
                        _ => return Err(self.error("invalid string escape")),
                    }
                }
                ch if ch.is_control() => return Err(self.error("control character in string")),
                _ => out.push(ch),
            }
        }

        Ok(out)
    }

    fn parse_unicode_escape(&mut self) -> Result<u32, String> {
        let mut value: u32 = 0;
        for _ in 0..4 {
            let Some(ch) = self.next_char() else {
                return Err(self.error("unterminated unicode escape"));
            };
            let digit = ch
                .to_digit(16)
                .ok_or_else(|| self.error("invalid unicode escape"))?;
            value = value.saturating_mul(16).saturating_add(digit);
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
                self.consume_digits();
            }
            _ => return Err(self.error("invalid number")),
        }

        let mut is_float = false;
        if self.peek_char() == Some('.') {
            is_float = true;
            self.next_char();
            if !matches!(self.peek_char(), Some('0'..='9')) {
                return Err(self.error("invalid number fraction"));
            }
            self.consume_digits();
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
            self.consume_digits();
        }

        let number_text = &self.input[start..self.index];
        if is_float {
            let value = number_text
                .parse::<f64>()
                .map_err(|_| self.error("invalid floating-point number"))?;
            Ok(JsonNumber::Float(value))
        } else {
            let value = number_text
                .parse::<i128>()
                .map_err(|_| self.error("invalid integer number"))?;
            Ok(JsonNumber::Integer(value))
        }
    }

    fn consume_digits(&mut self) {
        while matches!(self.peek_char(), Some('0'..='9')) {
            self.next_char();
        }
    }

    fn consume_literal(&mut self, literal: &str) -> Result<(), String> {
        for expected in literal.chars() {
            match self.next_char() {
                Some(actual) if actual == expected => {}
                _ => return Err(self.error("invalid literal")),
            }
        }
        Ok(())
    }

    fn expect_char(&mut self, expected: char) -> Result<(), String> {
        match self.next_char() {
            Some(actual) if actual == expected => Ok(()),
            _ => Err(self.error(&format!("expected '{expected}'"))),
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_char(), Some(' ' | '\n' | '\r' | '\t')) {
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
        format!("{message} at byte {}", self.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, remove_dir_all, write};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn merges_defaults_and_command_specific_overrides() {
        let test_dir = TestDir::new();
        let custom_path = test_dir.path().join("custom.json");
        write(
            &custom_path,
            r#"{
                "defaults": { "json": true, "offline": true, "timezone": "UTC+01:00" },
                "commands": {
                    "daily": { "offline": false, "instances": true, "project": "demo" }
                }
            }"#,
        )
        .expect("failed to write custom config");

        let layer =
            load_custom_config_layer("daily", &custom_path).expect("expected custom config layer");
        assert_eq!(layer.json, Some(true));
        assert_eq!(layer.offline, Some(false));
        assert_eq!(layer.instances, Some(true));
        assert_eq!(layer.project.as_deref(), Some("demo"));
        assert_eq!(layer.timezone.as_deref(), Some("UTC+01:00"));
    }

    #[test]
    fn applies_auto_config_priority_legacy_user_local() {
        let test_dir = TestDir::new();
        let home_dir = test_dir.path().join("home");
        let cwd = test_dir.path().join("workspace");

        create_dir_all(home_dir.join(".claude")).expect("failed to create legacy dir");
        create_dir_all(home_dir.join(".config/claude")).expect("failed to create user dir");
        create_dir_all(cwd.join(".ccusage")).expect("failed to create local config dir");

        write(
            home_dir.join(".claude/ccusage.json"),
            r#"{"commands":{"daily":{"project":"legacy","offline":true}}}"#,
        )
        .expect("failed to write legacy config");
        write(
            home_dir.join(".config/claude/ccusage.json"),
            r#"{"commands":{"daily":{"project":"user"}}}"#,
        )
        .expect("failed to write user config");
        write(
            cwd.join(".ccusage/ccusage.json"),
            r#"{"defaults":{"json":true},"commands":{"daily":{"project":"local","offline":false}}}"#,
        )
        .expect("failed to write local config");

        let layer = load_auto_config_layer("daily", &cwd, Some(&home_dir))
            .expect("expected merged auto config");

        assert_eq!(layer.json, Some(true));
        assert_eq!(layer.project.as_deref(), Some("local"));
        assert_eq!(layer.offline, Some(false));
    }

    #[test]
    fn missing_custom_config_is_an_error() {
        let error =
            load_custom_config_layer("daily", Path::new("/tmp/does-not-exist-cusage-rs.json"))
                .expect_err("expected missing custom config to fail");
        assert!(error.contains("does not exist"));
    }

    #[test]
    fn invalid_types_are_rejected() {
        let test_dir = TestDir::new();
        let custom_path = test_dir.path().join("invalid.json");
        write(
            &custom_path,
            r#"{
                "defaults": { "offline": "yes" }
            }"#,
        )
        .expect("failed to write invalid config");

        let error = load_custom_config_layer("daily", &custom_path)
            .expect_err("expected invalid config to fail");
        assert!(error.contains("defaults.offline"));
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
                "cusage-rs-runtime-config-tests-{}-{timestamp}-{counter}",
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
