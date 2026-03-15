use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventOrigin {
    pub file: PathBuf,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    Assistant,
    User,
    System,
    Tool,
    ToolResult,
    Unknown(String),
}

impl EventKind {
    #[must_use]
    pub fn from_raw(raw: Option<&str>) -> Self {
        let normalized = raw
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| {
                value
                    .chars()
                    .map(|ch| if ch == '-' || ch == ' ' { '_' } else { ch })
                    .collect::<String>()
                    .to_ascii_lowercase()
            });

        match normalized.as_deref() {
            Some("assistant") | Some("assistant_message") | Some("message") => Self::Assistant,
            Some("user") | Some("user_message") | Some("human") => Self::User,
            Some("system") | Some("system_message") => Self::System,
            Some("tool") | Some("tool_use") | Some("tool_call") => Self::Tool,
            Some("tool_result") | Some("tool_response") => Self::ToolResult,
            Some(other) => Self::Unknown(other.to_owned()),
            None => Self::Unknown("unknown".to_owned()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageSpeed {
    Standard,
    Fast,
}

impl UsageSpeed {
    #[must_use]
    pub fn from_raw(raw: Option<&str>) -> Option<Self> {
        let normalized = raw
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_ascii_lowercase);

        match normalized.as_deref() {
            Some("standard") => Some(Self::Standard),
            Some("fast") => Some(Self::Fast),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub total_tokens: u64,
}

impl TokenUsage {
    #[must_use]
    pub fn new(
        input_tokens: u64,
        output_tokens: u64,
        cache_creation_input_tokens: u64,
        cache_read_input_tokens: u64,
        total_tokens: Option<u64>,
    ) -> Self {
        let derived_total = input_tokens
            .saturating_add(output_tokens)
            .saturating_add(cache_creation_input_tokens)
            .saturating_add(cache_read_input_tokens);

        Self {
            input_tokens,
            output_tokens,
            cache_creation_input_tokens,
            cache_read_input_tokens,
            total_tokens: total_tokens.unwrap_or(derived_total),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageEvent {
    pub origin: EventOrigin,
    pub occurred_at_unix_ms: i64,
    pub event_kind: EventKind,
    pub session_id: Option<String>,
    pub project: Option<String>,
    pub model: Option<String>,
    pub speed: Option<UsageSpeed>,
    pub usage: TokenUsage,
    pub raw_cost_usd: Option<f64>,
}
