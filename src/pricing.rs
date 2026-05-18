use crate::domain::{TokenUsage, UsageEvent, UsageSpeed};
use std::collections::BTreeMap;
use std::process::Command;

const TIER_THRESHOLD_TOKENS: u64 = 200_000;
const DEFAULT_FAST_MULTIPLIER: f64 = 1.0;
pub const OPENAI_PRICING_URL: &str = "https://developers.openai.com/api/docs/pricing";
const DEFAULT_PROVIDER_PREFIXES: &[&str] = &[
    "anthropic",
    "openai",
    "openrouter",
    "vertex_ai",
    "bedrock",
    "azure",
    "gemini",
];
const OPENAI_LIVE_PRICE_MODELS: &[&str] = &[
    "gpt-5.5-pro",
    "gpt-5.5",
    "gpt-5.4-mini",
    "gpt-5.4-nano",
    "gpt-5.4-pro",
    "gpt-5.4",
    "gpt-realtime-2",
    "gpt-realtime-1.5",
    "gpt-realtime-mini",
    "gpt-image-2",
    "gpt-image-1.5",
    "gpt-image-1-mini",
    "gpt-4o-transcribe",
    "gpt-4o-mini-transcribe",
    "chat-latest",
    "gpt-5.3-codex",
    "o3-deep-research",
    "o4-mini-deep-research",
    "computer-use-preview",
];
const CLAUDE_4_OPUS_ALIASES: &[&str] = &[
    "claude-opus-4-20250514",
    "claude-opus-4",
    "claude-opus",
    "anthropic.claude-opus-4-20250514-v1:0",
];
const CLAUDE_4_5_OPUS_ALIASES: &[&str] = &["claude-opus-4-5"];
const CLAUDE_4_6_OPUS_ALIASES: &[&str] = &[
    "claude-opus-4-6",
    "claude-opus-4-6-20260205",
    "anthropic.claude-opus-4-6-v1",
];
const CLAUDE_4_SONNET_ALIASES: &[&str] = &[
    "claude-sonnet-4-20250514",
    "claude-sonnet-4",
    "anthropic.claude-sonnet-4-20250514-v1:0",
];
const CLAUDE_4_5_SONNET_ALIASES: &[&str] = &[
    "claude-sonnet-4-5",
    "claude-sonnet-4-5-20250929",
    "claude-sonnet-4-5-20250929-v1:0",
    "anthropic.claude-sonnet-4-5-20250929-v1:0",
];
const CLAUDE_4_6_SONNET_ALIASES: &[&str] = &["claude-sonnet-4-6", "anthropic.claude-sonnet-4-6"];
const CLAUDE_4_5_HAIKU_ALIASES: &[&str] = &[
    "claude-haiku-4-5",
    "claude-haiku-4-5-20251001",
    "anthropic.claude-haiku-4-5-20251001-v1:0",
];
const CLAUDE_3_7_SONNET_ALIASES: &[&str] = &[
    "claude-3-7-sonnet-20250219",
    "claude-3-7-sonnet",
    "claude-3.7-sonnet",
    "anthropic.claude-3-7-sonnet-20250219-v1:0",
];
const CLAUDE_3_5_SONNET_ALIASES: &[&str] = &[
    "claude-3-5-sonnet-20241022",
    "claude-3-5-sonnet",
    "claude-3.5-sonnet",
    "claude-sonnet",
    "anthropic.claude-3-5-sonnet-20241022-v2:0",
];
const CLAUDE_3_5_HAIKU_ALIASES: &[&str] = &[
    "claude-3-5-haiku-20241022",
    "claude-3-5-haiku",
    "claude-3.5-haiku",
    "anthropic.claude-3-5-haiku-20241022-v1:0",
];
const CLAUDE_3_OPUS_ALIASES: &[&str] = &[
    "claude-3-opus-20240229",
    "claude-3-opus",
    "anthropic.claude-3-opus-20240229-v1:0",
];
const CLAUDE_3_HAIKU_ALIASES: &[&str] = &[
    "claude-3-haiku-20240307",
    "claude-3-haiku",
    "claude-haiku",
    "anthropic.claude-3-haiku-20240307-v1:0",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CostMode {
    #[default]
    Auto,
    Calculate,
    Display,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostSource {
    Raw,
    Calculated,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedCost {
    pub cost_usd: f64,
    pub source: CostSource,
}

impl ResolvedCost {
    fn missing() -> Self {
        Self {
            cost_usd: 0.0,
            source: CostSource::Missing,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelPricing {
    pub input_cost_per_token: f64,
    pub output_cost_per_token: f64,
    pub cache_creation_input_cost_per_token: f64,
    pub cache_read_input_cost_per_token: f64,
    pub input_cost_per_token_above_200k: Option<f64>,
    pub output_cost_per_token_above_200k: Option<f64>,
    pub cache_creation_input_cost_per_token_above_200k: Option<f64>,
    pub cache_read_input_cost_per_token_above_200k: Option<f64>,
    pub fast_multiplier: f64,
}

impl ModelPricing {
    #[must_use]
    pub fn from_per_million(
        input_usd_per_million: f64,
        output_usd_per_million: f64,
        cache_creation_input_usd_per_million: f64,
        cache_read_input_usd_per_million: f64,
    ) -> Self {
        Self {
            input_cost_per_token: input_usd_per_million / 1_000_000.0,
            output_cost_per_token: output_usd_per_million / 1_000_000.0,
            cache_creation_input_cost_per_token: cache_creation_input_usd_per_million / 1_000_000.0,
            cache_read_input_cost_per_token: cache_read_input_usd_per_million / 1_000_000.0,
            input_cost_per_token_above_200k: None,
            output_cost_per_token_above_200k: None,
            cache_creation_input_cost_per_token_above_200k: None,
            cache_read_input_cost_per_token_above_200k: None,
            fast_multiplier: DEFAULT_FAST_MULTIPLIER,
        }
    }

    #[must_use]
    pub fn with_tiered_per_million(
        mut self,
        input_usd_per_million_above_200k: Option<f64>,
        output_usd_per_million_above_200k: Option<f64>,
        cache_creation_input_usd_per_million_above_200k: Option<f64>,
        cache_read_input_usd_per_million_above_200k: Option<f64>,
    ) -> Self {
        self.input_cost_per_token_above_200k =
            input_usd_per_million_above_200k.map(|v| v / 1_000_000.0);
        self.output_cost_per_token_above_200k =
            output_usd_per_million_above_200k.map(|v| v / 1_000_000.0);
        self.cache_creation_input_cost_per_token_above_200k =
            cache_creation_input_usd_per_million_above_200k.map(|v| v / 1_000_000.0);
        self.cache_read_input_cost_per_token_above_200k =
            cache_read_input_usd_per_million_above_200k.map(|v| v / 1_000_000.0);
        self
    }

    #[must_use]
    pub fn with_fast_multiplier(mut self, multiplier: f64) -> Self {
        self.fast_multiplier = if multiplier.is_finite() && multiplier > 0.0 {
            multiplier
        } else {
            DEFAULT_FAST_MULTIPLIER
        };
        self
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PricingCatalog {
    by_model: BTreeMap<String, ModelPricing>,
    provider_prefixes: Vec<String>,
}

impl PricingCatalog {
    #[must_use]
    pub fn new() -> Self {
        Self::default().with_default_provider_prefixes()
    }

    #[must_use]
    pub fn default_claude_catalog() -> Self {
        Self::new().with_default_claude_pricing()
    }

    #[must_use]
    pub fn default_catalog() -> Self {
        Self::default_claude_catalog().with_default_openai_pricing()
    }

    #[must_use]
    pub fn default_catalog_with_live_openai(allow_live_fetch: bool) -> Self {
        let mut catalog = Self::default_catalog();
        if allow_live_fetch && let Ok(live_catalog) = fetch_live_openai_pricing() {
            catalog.merge_from(&live_catalog);
        }
        catalog
    }

    #[must_use]
    pub fn with_default_provider_prefixes(mut self) -> Self {
        if self.provider_prefixes.is_empty() {
            self.provider_prefixes = DEFAULT_PROVIDER_PREFIXES
                .iter()
                .map(|prefix| (*prefix).to_owned())
                .collect();
        }
        self
    }

    #[must_use]
    pub fn with_default_claude_pricing(mut self) -> Self {
        let claude_4_sonnet_pricing = ModelPricing::from_per_million(3.0, 15.0, 3.75, 0.3)
            .with_tiered_per_million(Some(6.0), Some(22.5), Some(7.5), Some(0.6));
        let claude_3_sonnet_pricing = ModelPricing::from_per_million(3.0, 15.0, 3.75, 0.3);
        let claude_4_haiku_pricing = ModelPricing::from_per_million(1.0, 5.0, 1.25, 0.1);
        let claude_3_5_haiku_pricing = ModelPricing::from_per_million(0.8, 4.0, 1.0, 0.08);
        let claude_3_haiku_pricing = ModelPricing::from_per_million(0.25, 1.25, 0.3, 0.03);
        let claude_4_opus_pricing = ModelPricing::from_per_million(15.0, 75.0, 18.75, 1.5);
        let claude_4_5_opus_pricing = ModelPricing::from_per_million(5.0, 25.0, 6.25, 0.5);
        let claude_4_6_opus_pricing = ModelPricing::from_per_million(5.0, 25.0, 6.25, 0.5)
            .with_tiered_per_million(Some(10.0), Some(37.5), Some(12.5), Some(1.0));

        self.insert_aliases(CLAUDE_4_OPUS_ALIASES, &claude_4_opus_pricing);
        self.insert_aliases(CLAUDE_4_5_OPUS_ALIASES, &claude_4_5_opus_pricing);
        self.insert_aliases(CLAUDE_4_6_OPUS_ALIASES, &claude_4_6_opus_pricing);
        self.insert_aliases(CLAUDE_4_SONNET_ALIASES, &claude_4_sonnet_pricing);
        self.insert_aliases(CLAUDE_4_5_SONNET_ALIASES, &claude_4_sonnet_pricing);
        self.insert_aliases(CLAUDE_4_6_SONNET_ALIASES, &claude_4_sonnet_pricing);
        self.insert_aliases(CLAUDE_3_7_SONNET_ALIASES, &claude_3_sonnet_pricing);
        self.insert_aliases(CLAUDE_3_5_SONNET_ALIASES, &claude_3_sonnet_pricing);
        self.insert_aliases(CLAUDE_4_5_HAIKU_ALIASES, &claude_4_haiku_pricing);
        self.insert_aliases(CLAUDE_3_5_HAIKU_ALIASES, &claude_3_5_haiku_pricing);
        self.insert_aliases(CLAUDE_3_OPUS_ALIASES, &claude_4_opus_pricing);
        self.insert_aliases(CLAUDE_3_HAIKU_ALIASES, &claude_3_haiku_pricing);

        self
    }

    #[must_use]
    pub fn with_default_openai_pricing(mut self) -> Self {
        self.insert_openai_model("gpt-5.5", 5.0, Some(0.5), 30.0);
        self.insert_openai_model("gpt-5.5-2026-04-23", 5.0, Some(0.5), 30.0);
        self.insert_openai_model("gpt-5.5-pro", 30.0, None, 180.0);
        self.insert_openai_model("gpt-5.4", 2.5, Some(0.25), 15.0);
        self.insert_openai_model("gpt-5.4-mini", 0.75, Some(0.075), 4.5);
        self.insert_openai_model("gpt-5.4-nano", 0.20, Some(0.02), 1.25);
        self.insert_openai_model("gpt-5.4-pro", 30.0, None, 180.0);
        self.insert_openai_model("gpt-5.3-codex", 1.75, Some(0.175), 14.0);
        self.insert_openai_model("gpt-5.2", 1.75, Some(0.175), 14.0);
        self.insert_openai_model("gpt-5.2-2025-12-11", 1.75, Some(0.175), 14.0);
        self.insert_openai_model("gpt-5.2-chat-latest", 1.75, Some(0.175), 14.0);
        self.insert_openai_model("gpt-5.2-codex", 1.75, Some(0.175), 14.0);
        self.insert_openai_model("gpt-5.1", 1.25, Some(0.125), 10.0);
        self.insert_openai_model("gpt-5.1-chat-latest", 1.25, Some(0.125), 10.0);
        self.insert_openai_model("gpt-5", 1.25, Some(0.125), 10.0);
        self.insert_openai_model("gpt-5-2025-08-07", 1.25, Some(0.125), 10.0);
        self.insert_openai_model("gpt-5-chat-latest", 1.25, Some(0.125), 10.0);
        self.insert_openai_model("gpt-5-codex", 1.25, Some(0.125), 10.0);
        self.insert_openai_model("gpt-5-mini", 0.25, Some(0.025), 2.0);
        self.insert_openai_model("gpt-5-nano", 0.05, Some(0.005), 0.4);
        self.insert_openai_model("gpt-4.1", 2.0, Some(0.5), 8.0);
        self.insert_openai_model("gpt-4.1-2025-04-14", 2.0, Some(0.5), 8.0);
        self.insert_openai_model("gpt-4.1-mini", 0.4, Some(0.1), 1.6);
        self.insert_openai_model("gpt-4.1-nano", 0.1, Some(0.025), 0.4);
        self.insert_openai_model("gpt-4o", 2.5, Some(1.25), 10.0);
        self.insert_openai_model("gpt-4o-2024-08-06", 2.5, Some(1.25), 10.0);
        self.insert_openai_model("gpt-4o-mini", 0.15, Some(0.075), 0.6);
        self.insert_openai_model("gpt-4o-mini-2024-07-18", 0.15, Some(0.075), 0.6);
        self.insert_openai_model("gpt-4", 30.0, None, 60.0);
        self.insert_openai_model("gpt-4-turbo", 10.0, None, 30.0);
        self.insert_openai_model("gpt-3.5-turbo", 0.5, None, 1.5);
        self.insert_openai_model("o3", 2.0, Some(0.5), 8.0);
        self.insert_openai_model("o3-2025-04-16", 2.0, Some(0.5), 8.0);
        self.insert_openai_model("o3-mini", 1.1, Some(0.55), 4.4);
        self.insert_openai_model("o4-mini", 1.1, Some(0.275), 4.4);
        self.insert_openai_model("o4-mini-2025-04-16", 1.1, Some(0.275), 4.4);
        self.insert_openai_model("o3-deep-research", 5.0, None, 20.0);
        self.insert_openai_model("o4-mini-deep-research", 1.0, None, 4.0);
        self.insert_openai_model("computer-use-preview", 1.5, None, 6.0);
        self.insert_openai_model("gpt-realtime-2", 4.0, Some(0.4), 24.0);
        self.insert_openai_model("gpt-realtime-1.5", 4.0, Some(0.4), 16.0);
        self.insert_openai_model("gpt-realtime-mini", 0.6, Some(0.06), 2.4);
        self.insert_openai_model("gpt-4o-transcribe", 2.5, None, 10.0);
        self.insert_openai_model("gpt-4o-mini-transcribe", 1.25, None, 5.0);
        self.insert_openai_model("chat-latest", 5.0, Some(0.5), 30.0);
        self.insert_openai_model("gpt-image-2", 5.0, Some(1.25), 30.0);
        self.insert_openai_model("gpt-image-1.5", 5.0, Some(1.25), 10.0);
        self.insert_openai_model("gpt-image-1-mini", 2.0, Some(0.2), 8.0);

        self
    }

    pub fn insert(&mut self, model: impl Into<String>, pricing: ModelPricing) {
        let normalized = normalize_model_key(&model.into());
        if normalized.is_empty() {
            return;
        }
        self.by_model.insert(normalized, pricing);
    }

    pub fn merge_from(&mut self, other: &Self) {
        for (model, pricing) in &other.by_model {
            self.by_model.insert(model.clone(), pricing.clone());
        }
        for provider in &other.provider_prefixes {
            if !self.provider_prefixes.contains(provider) {
                self.provider_prefixes.push(provider.clone());
            }
        }
    }

    fn insert_openai_model(
        &mut self,
        model: &'static str,
        input_usd_per_million: f64,
        cached_input_usd_per_million: Option<f64>,
        output_usd_per_million: f64,
    ) {
        self.insert(
            model,
            openai_model_pricing(
                input_usd_per_million,
                cached_input_usd_per_million,
                output_usd_per_million,
            ),
        );
    }

    fn insert_aliases(&mut self, aliases: &[&str], pricing: &ModelPricing) {
        for alias in aliases {
            self.insert(*alias, pricing.clone());
        }
    }

    #[must_use]
    pub fn resolve(&self, model: &str) -> Option<&ModelPricing> {
        let model = normalize_model_key(model);
        if model.is_empty() {
            return None;
        }

        if let Some(pricing) = self.by_model.get(&model) {
            return Some(pricing);
        }

        if let Some(stripped) = self.strip_known_provider_prefix(&model)
            && let Some(pricing) = self.by_model.get(stripped)
        {
            return Some(pricing);
        }

        for provider in &self.provider_prefixes {
            let candidate = format!("{provider}/{model}");
            if let Some(pricing) = self.by_model.get(&candidate) {
                return Some(pricing);
            }
        }

        self.fuzzy_match(&model)
    }

    fn strip_known_provider_prefix<'a>(&self, model: &'a str) -> Option<&'a str> {
        let (provider, stripped) = model.split_once('/')?;
        if stripped.is_empty() {
            return None;
        }
        if self.provider_prefixes.iter().any(|known| known == provider) {
            return Some(stripped);
        }
        None
    }

    fn fuzzy_match(&self, model: &str) -> Option<&ModelPricing> {
        let mut best_key: Option<(&str, usize)> = None;

        for candidate in self.by_model.keys() {
            if !candidate.contains(model) && !model.contains(candidate) {
                continue;
            }

            let distance = candidate.len().abs_diff(model.len());
            match best_key {
                None => best_key = Some((candidate.as_str(), distance)),
                Some((best, best_distance)) => {
                    if distance < best_distance
                        || (distance == best_distance && candidate.as_str() < best)
                    {
                        best_key = Some((candidate.as_str(), distance));
                    }
                }
            }
        }

        best_key.and_then(|(key, _)| self.by_model.get(key))
    }
}

#[must_use]
pub fn parse_openai_pricing_page(contents: &str) -> PricingCatalog {
    let text = html_to_text(contents);
    let mut catalog = PricingCatalog::new();

    for model in OPENAI_LIVE_PRICE_MODELS {
        if let Some(pricing) = parse_openai_pricing_for_model(&text, model) {
            catalog.insert(*model, pricing);
        }
    }

    catalog
}

pub fn fetch_live_openai_pricing() -> Result<PricingCatalog, String> {
    let output = Command::new("curl")
        .args(["-fsSL", "--max-time", "8", OPENAI_PRICING_URL])
        .output()
        .map_err(|error| format!("failed to run curl for OpenAI pricing: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "failed to fetch OpenAI pricing from {OPENAI_PRICING_URL}: {stderr}"
        ));
    }

    let contents = String::from_utf8(output.stdout)
        .map_err(|error| format!("OpenAI pricing response was not valid UTF-8: {error}"))?;
    let catalog = parse_openai_pricing_page(&contents);
    if catalog.by_model.is_empty() {
        return Err("OpenAI pricing response did not contain parseable token rows".to_owned());
    }

    Ok(catalog)
}

fn openai_model_pricing(
    input_usd_per_million: f64,
    cached_input_usd_per_million: Option<f64>,
    output_usd_per_million: f64,
) -> ModelPricing {
    let cached = cached_input_usd_per_million.unwrap_or(input_usd_per_million);
    ModelPricing::from_per_million(
        input_usd_per_million,
        output_usd_per_million,
        input_usd_per_million,
        cached,
    )
}

fn parse_openai_pricing_for_model(text: &str, model: &str) -> Option<ModelPricing> {
    for line in text.lines() {
        let Some(after_model) = line_after_model(line, model) else {
            continue;
        };
        let cells = parse_price_cells(after_model);
        if let Some(pricing) = openai_pricing_from_cells(&cells) {
            return Some(pricing);
        }
    }

    let mut search_offset = 0;
    while let Some(relative_index) = text[search_offset..].find(model) {
        let start = search_offset + relative_index;
        let after_model_start = start + model.len();
        if !is_model_boundary(text, start, after_model_start) {
            search_offset = after_model_start;
            continue;
        }

        let end = text[after_model_start..]
            .find('\n')
            .map_or(text.len(), |relative_end| after_model_start + relative_end);
        let cells = parse_price_cells(&text[after_model_start..end]);
        if let Some(pricing) = openai_pricing_from_cells(&cells) {
            return Some(pricing);
        }
        search_offset = after_model_start;
    }

    None
}

fn line_after_model<'a>(line: &'a str, model: &str) -> Option<&'a str> {
    let index = line.find(model)?;
    let after_model_start = index + model.len();
    if !is_model_boundary(line, index, after_model_start) {
        return None;
    }
    Some(&line[after_model_start..])
}

fn is_model_boundary(text: &str, start: usize, end: usize) -> bool {
    let before_ok = text[..start]
        .chars()
        .next_back()
        .is_none_or(|ch| !is_model_char(ch));
    let after_ok = text[end..]
        .chars()
        .next()
        .is_none_or(|ch| !is_model_char(ch));
    before_ok && after_ok
}

fn is_model_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.')
}

fn openai_pricing_from_cells(cells: &[Option<f64>]) -> Option<ModelPricing> {
    let input = cells.first().copied().flatten()?;
    let output_index = if cells.get(1).is_some_and(Option::is_none) {
        2
    } else {
        2
    };
    let cached = cells.get(1).copied().flatten();
    let output = cells.get(output_index).copied().flatten()?;
    Some(openai_model_pricing(input, cached, output))
}

fn parse_price_cells(input: &str) -> Vec<Option<f64>> {
    let mut cells = Vec::new();
    let bytes = input.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'$' => {
                index += 1;
                let start = index;
                while index < bytes.len()
                    && (bytes[index].is_ascii_digit()
                        || bytes[index] == b'.'
                        || bytes[index] == b',')
                {
                    index += 1;
                }
                let raw = input[start..index].replace(',', "");
                if let Ok(value) = raw.parse::<f64>() {
                    cells.push(Some(value));
                }
            }
            b'-' => {
                cells.push(None);
                index += 1;
            }
            byte if byte.is_ascii_whitespace() => index += 1,
            _ if cells.is_empty() => index += 1,
            _ => break,
        }
    }
    cells
}

fn html_to_text(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_tag = false;
    let mut last_was_space = false;

    for ch in input.chars() {
        match ch {
            '<' => {
                in_tag = true;
                if !last_was_space {
                    output.push(' ');
                    last_was_space = true;
                }
            }
            '>' => in_tag = false,
            _ if in_tag => {}
            '&' => {
                if !last_was_space {
                    output.push(' ');
                    last_was_space = true;
                }
            }
            ch if ch.is_whitespace() => {
                if ch == '\n' {
                    output.push('\n');
                    last_was_space = true;
                } else if !last_was_space {
                    output.push(' ');
                    last_was_space = true;
                }
            }
            ch => {
                output.push(ch);
                last_was_space = false;
            }
        }
    }

    output
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DerivedMetrics {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub entries_with_raw_cost: usize,
    pub entries_with_calculated_cost: usize,
    pub entries_with_missing_cost: usize,
}

#[must_use]
pub fn total_tokens_for_usage(usage: &TokenUsage) -> u64 {
    usage
        .input_tokens
        .saturating_add(usage.output_tokens)
        .saturating_add(usage.cache_creation_input_tokens)
        .saturating_add(usage.cache_read_input_tokens)
}

#[must_use]
pub fn calculate_cost_from_usage(
    usage: &TokenUsage,
    pricing: &ModelPricing,
    speed: Option<UsageSpeed>,
) -> f64 {
    let input_cost = tiered_cost(
        usage.input_tokens,
        pricing.input_cost_per_token,
        pricing.input_cost_per_token_above_200k,
    );
    let output_cost = tiered_cost(
        usage.output_tokens,
        pricing.output_cost_per_token,
        pricing.output_cost_per_token_above_200k,
    );
    let cache_creation_cost = tiered_cost(
        usage.cache_creation_input_tokens,
        pricing.cache_creation_input_cost_per_token,
        pricing.cache_creation_input_cost_per_token_above_200k,
    );
    let cache_read_cost = tiered_cost(
        usage.cache_read_input_tokens,
        pricing.cache_read_input_cost_per_token,
        pricing.cache_read_input_cost_per_token_above_200k,
    );

    let base = input_cost + output_cost + cache_creation_cost + cache_read_cost;
    if speed == Some(UsageSpeed::Fast) {
        base * pricing.fast_multiplier
    } else {
        base
    }
}

#[must_use]
pub fn resolve_event_cost(
    event: &UsageEvent,
    mode: CostMode,
    catalog: &PricingCatalog,
) -> ResolvedCost {
    match mode {
        CostMode::Display => match event.raw_cost_usd {
            Some(cost_usd) if cost_usd.is_finite() && cost_usd >= 0.0 => ResolvedCost {
                cost_usd,
                source: CostSource::Raw,
            },
            _ => ResolvedCost::missing(),
        },
        CostMode::Calculate => calculate_from_catalog(event, catalog),
        CostMode::Auto => {
            if let Some(cost_usd) = event.raw_cost_usd
                && cost_usd.is_finite()
                && cost_usd >= 0.0
            {
                return ResolvedCost {
                    cost_usd,
                    source: CostSource::Raw,
                };
            }
            calculate_from_catalog(event, catalog)
        }
    }
}

#[must_use]
pub fn derive_metrics(
    events: &[UsageEvent],
    mode: CostMode,
    catalog: &PricingCatalog,
) -> DerivedMetrics {
    let mut out = DerivedMetrics::default();

    for event in events {
        out.input_tokens = out.input_tokens.saturating_add(event.usage.input_tokens);
        out.output_tokens = out.output_tokens.saturating_add(event.usage.output_tokens);
        out.cache_creation_input_tokens = out
            .cache_creation_input_tokens
            .saturating_add(event.usage.cache_creation_input_tokens);
        out.cache_read_input_tokens = out
            .cache_read_input_tokens
            .saturating_add(event.usage.cache_read_input_tokens);
        out.total_tokens = out
            .total_tokens
            .saturating_add(total_tokens_for_usage(&event.usage));

        let resolved = resolve_event_cost(event, mode, catalog);
        out.total_cost_usd += resolved.cost_usd;

        match resolved.source {
            CostSource::Raw => out.entries_with_raw_cost += 1,
            CostSource::Calculated => out.entries_with_calculated_cost += 1,
            CostSource::Missing => out.entries_with_missing_cost += 1,
        }
    }

    out
}

fn tiered_cost(tokens: u64, normal_rate_per_token: f64, rate_above_threshold: Option<f64>) -> f64 {
    if tokens == 0 || !normal_rate_per_token.is_finite() || normal_rate_per_token <= 0.0 {
        return 0.0;
    }

    let normal_tokens = tokens.min(TIER_THRESHOLD_TOKENS);
    let overflow_tokens = tokens.saturating_sub(TIER_THRESHOLD_TOKENS);

    let mut total = normal_tokens as f64 * normal_rate_per_token;
    if overflow_tokens > 0 {
        let overflow_rate = rate_above_threshold.unwrap_or(normal_rate_per_token);
        if overflow_rate.is_finite() && overflow_rate > 0.0 {
            total += overflow_tokens as f64 * overflow_rate;
        }
    }
    total
}

fn calculate_from_catalog(event: &UsageEvent, catalog: &PricingCatalog) -> ResolvedCost {
    let Some(model) = event.model.as_deref() else {
        return ResolvedCost::missing();
    };
    let Some(pricing) = catalog.resolve(model) else {
        return ResolvedCost::missing();
    };

    ResolvedCost {
        cost_usd: calculate_cost_from_usage(&event.usage, pricing, event.speed),
        source: CostSource::Calculated,
    }
}

fn normalize_model_key(model: &str) -> String {
    model.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{EventKind, EventOrigin};
    use std::path::PathBuf;

    #[test]
    fn display_mode_prefers_raw_cost_and_defaults_to_zero() {
        let catalog = test_catalog();
        let with_raw = test_event(
            "claude-sonnet",
            Some(UsageSpeed::Standard),
            Some(1.25),
            10,
            5,
            0,
            0,
        );
        let without_raw = test_event(
            "claude-sonnet",
            Some(UsageSpeed::Standard),
            None,
            10,
            5,
            0,
            0,
        );

        let resolved_with_raw = resolve_event_cost(&with_raw, CostMode::Display, &catalog);
        let resolved_without_raw = resolve_event_cost(&without_raw, CostMode::Display, &catalog);

        assert_eq!(resolved_with_raw.source, CostSource::Raw);
        assert_close(resolved_with_raw.cost_usd, 1.25);
        assert_eq!(resolved_without_raw.source, CostSource::Missing);
        assert_close(resolved_without_raw.cost_usd, 0.0);
    }

    #[test]
    fn calculate_mode_ignores_raw_and_uses_catalog_pricing() {
        let catalog = test_catalog();
        let event = test_event(
            "claude-sonnet",
            Some(UsageSpeed::Standard),
            Some(9.99),
            500_000,
            100_000,
            0,
            0,
        );

        let resolved = resolve_event_cost(&event, CostMode::Calculate, &catalog);

        assert_eq!(resolved.source, CostSource::Calculated);
        assert_close(resolved.cost_usd, 0.8);
    }

    #[test]
    fn auto_mode_uses_raw_first_then_calculated() {
        let catalog = test_catalog();
        let raw_event = test_event(
            "claude-sonnet",
            Some(UsageSpeed::Standard),
            Some(0.75),
            1_000,
            1_000,
            0,
            0,
        );
        let calc_event = test_event(
            "claude-sonnet",
            Some(UsageSpeed::Standard),
            None,
            1_000,
            1_000,
            0,
            0,
        );

        let raw_resolved = resolve_event_cost(&raw_event, CostMode::Auto, &catalog);
        let calc_resolved = resolve_event_cost(&calc_event, CostMode::Auto, &catalog);

        assert_eq!(raw_resolved.source, CostSource::Raw);
        assert_close(raw_resolved.cost_usd, 0.75);

        assert_eq!(calc_resolved.source, CostSource::Calculated);
        assert_close(calc_resolved.cost_usd, 0.004);
    }

    #[test]
    fn tiered_pricing_kicks_in_above_threshold() {
        let pricing = ModelPricing::from_per_million(3.0, 15.0, 3.75, 0.3).with_tiered_per_million(
            Some(1.5),
            Some(7.5),
            Some(1.875),
            Some(0.15),
        );
        let usage = TokenUsage::new(300_000, 300_000, 300_000, 300_000, None);

        let cost = calculate_cost_from_usage(&usage, &pricing, Some(UsageSpeed::Standard));

        assert_close(cost, 5.5125);
    }

    #[test]
    fn fast_speed_applies_multiplier() {
        let pricing =
            ModelPricing::from_per_million(3.0, 15.0, 3.75, 0.3).with_fast_multiplier(1.5);
        let usage = TokenUsage::new(100_000, 50_000, 0, 0, None);

        let standard = calculate_cost_from_usage(&usage, &pricing, Some(UsageSpeed::Standard));
        let fast = calculate_cost_from_usage(&usage, &pricing, Some(UsageSpeed::Fast));

        assert_close(standard, 1.05);
        assert_close(fast, 1.575);
    }

    #[test]
    fn model_resolution_supports_provider_prefix_and_fuzzy_lookup() {
        let mut catalog = PricingCatalog::new();
        let pricing = ModelPricing::from_per_million(3.0, 15.0, 3.75, 0.3);
        catalog.insert("claude-3-5-sonnet-20241022", pricing.clone());

        assert!(catalog.resolve("claude-3-5-sonnet-20241022").is_some());
        assert!(
            catalog
                .resolve("anthropic/claude-3-5-sonnet-20241022")
                .is_some()
        );
        assert!(catalog.resolve("claude-3-5-sonnet").is_some());
    }

    #[test]
    fn default_catalog_resolves_claude_aliases_and_provider_prefixed_variants() {
        let catalog = PricingCatalog::default_claude_catalog();

        assert!(catalog.resolve("claude-sonnet").is_some());
        assert!(catalog.resolve("anthropic/claude-sonnet").is_some());
        assert!(catalog.resolve("openrouter/claude-3.5-sonnet").is_some());
        assert!(
            catalog
                .resolve("bedrock/anthropic.claude-3-5-sonnet-20241022-v2:0")
                .is_some()
        );
        assert!(catalog.resolve("claude-opus").is_some());
        assert!(catalog.resolve("claude-haiku").is_some());
    }

    #[test]
    fn default_catalog_calculates_known_model_without_raw_cost() {
        let catalog = PricingCatalog::default_claude_catalog();
        let event = test_event(
            "claude-sonnet",
            Some(UsageSpeed::Standard),
            None,
            1_000,
            500,
            0,
            0,
        );

        let resolved = resolve_event_cost(&event, CostMode::Auto, &catalog);

        assert_eq!(resolved.source, CostSource::Calculated);
        assert_close(resolved.cost_usd, 0.0105);
    }

    #[test]
    fn default_catalog_resolves_openai_models_and_provider_prefixed_variants() {
        let catalog = PricingCatalog::default_catalog();

        assert!(catalog.resolve("gpt-5.5").is_some());
        assert!(catalog.resolve("openai/gpt-5.5").is_some());
        assert!(catalog.resolve("gpt-4o-mini").is_some());
        assert!(catalog.resolve("o4-mini").is_some());
    }

    #[test]
    fn default_catalog_calculates_openai_cost_without_raw_cost() {
        let catalog = PricingCatalog::default_catalog();
        let event = test_event(
            "openai/gpt-5.5",
            Some(UsageSpeed::Standard),
            None,
            1_000_000,
            1_000_000,
            0,
            100_000,
        );

        let resolved = resolve_event_cost(&event, CostMode::Auto, &catalog);

        assert_eq!(resolved.source, CostSource::Calculated);
        assert_close(resolved.cost_usd, 35.05);
    }

    #[test]
    fn parses_openai_pricing_page_rows_from_official_text_shape() {
        let catalog = parse_openai_pricing_page(
            r#"
            Flagship models
            Model Input Cached input Output Input Cached input Output
            gpt-5.5$5.00$0.50$30.00$10.00$1.00$45.00
            gpt-5.5-pro$30.00-$180.00$60.00-$270.00
            gpt-5.4-mini$0.75$0.075$4.50---
            Specialized models
            Category Model Input Cached input Output
            Codex gpt-5.3-codex$1.75$0.175$14.00
            "#,
        );

        let gpt_55 = catalog
            .resolve("gpt-5.5")
            .expect("expected gpt-5.5 pricing");
        assert_close(gpt_55.input_cost_per_token, 5.0 / 1_000_000.0);
        assert_close(gpt_55.cache_read_input_cost_per_token, 0.5 / 1_000_000.0);
        assert_close(gpt_55.output_cost_per_token, 30.0 / 1_000_000.0);

        let pro = catalog
            .resolve("gpt-5.5-pro")
            .expect("expected gpt-5.5-pro pricing");
        assert_close(pro.cache_read_input_cost_per_token, 30.0 / 1_000_000.0);
        assert_close(pro.output_cost_per_token, 180.0 / 1_000_000.0);

        let codex = catalog
            .resolve("gpt-5.3-codex")
            .expect("expected gpt-5.3-codex pricing");
        assert_close(codex.output_cost_per_token, 14.0 / 1_000_000.0);
    }

    #[test]
    #[ignore = "network smoke test for the official OpenAI pricing page"]
    fn live_openai_pricing_fetch_smoke() {
        let catalog = fetch_live_openai_pricing().expect("expected live OpenAI pricing fetch");

        assert!(catalog.resolve("gpt-5.5").is_some());
        assert!(catalog.resolve("gpt-5.4").is_some());
        assert!(catalog.resolve("gpt-5.3-codex").is_some());
    }

    #[test]
    fn derives_totals_and_cost_sources() {
        let catalog = test_catalog();
        let events = vec![
            test_event(
                "claude-sonnet",
                Some(UsageSpeed::Standard),
                Some(0.5),
                10,
                5,
                1,
                2,
            ),
            test_event(
                "claude-sonnet",
                Some(UsageSpeed::Fast),
                None,
                1_000,
                1_000,
                0,
                0,
            ),
            test_event(
                "unknown-model",
                Some(UsageSpeed::Standard),
                None,
                4,
                5,
                6,
                7,
            ),
        ];

        let metrics = derive_metrics(&events, CostMode::Auto, &catalog);

        assert_eq!(metrics.input_tokens, 1_014);
        assert_eq!(metrics.output_tokens, 1_010);
        assert_eq!(metrics.cache_creation_input_tokens, 7);
        assert_eq!(metrics.cache_read_input_tokens, 9);
        assert_eq!(metrics.total_tokens, 2_040);
        assert_close(metrics.total_cost_usd, 0.506);
        assert_eq!(metrics.entries_with_raw_cost, 1);
        assert_eq!(metrics.entries_with_calculated_cost, 1);
        assert_eq!(metrics.entries_with_missing_cost, 1);
    }

    fn test_catalog() -> PricingCatalog {
        let mut catalog = PricingCatalog::new();
        catalog.insert(
            "claude-sonnet",
            ModelPricing::from_per_million(1.0, 3.0, 1.25, 0.1).with_fast_multiplier(1.5),
        );
        catalog
    }

    fn test_event(
        model: &str,
        speed: Option<UsageSpeed>,
        raw_cost_usd: Option<f64>,
        input: u64,
        output: u64,
        cache_create: u64,
        cache_read: u64,
    ) -> UsageEvent {
        UsageEvent {
            origin: EventOrigin {
                file: PathBuf::from("/tmp/session.jsonl"),
                line_number: 1,
            },
            occurred_at_unix_ms: 0,
            event_kind: EventKind::Assistant,
            session_id: None,
            project: None,
            model: Some(model.to_owned()),
            speed,
            usage: TokenUsage::new(input, output, cache_create, cache_read, None),
            raw_cost_usd,
        }
    }

    fn assert_close(left: f64, right: f64) {
        let delta = (left - right).abs();
        assert!(
            delta <= 1e-12,
            "expected {left} to be close to {right}, delta={delta}"
        );
    }
}
