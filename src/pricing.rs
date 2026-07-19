use crate::domain::{TokenUsage, UsageEvent, UsageSpeed};
use std::collections::BTreeMap;
use std::process::Command;

const TIER_THRESHOLD_TOKENS: u64 = 200_000;
const DEFAULT_FAST_MULTIPLIER: f64 = 1.0;
pub const OPENAI_PRICING_URL: &str = "https://developers.openai.com/api/docs/pricing";
pub const ANTHROPIC_PRICING_URL: &str =
    "https://platform.claude.com/docs/en/about-claude/pricing.md";
const DEFAULT_PROVIDER_PREFIXES: &[&str] = &[
    "anthropic",
    "openai",
    "openrouter",
    "vertex_ai",
    "bedrock",
    "azure",
    "gemini",
];
/// Models whose rows on the OpenAI pricing page are plain per-MTok token tables
/// (`Input | Cached input | [Cache writes |] Output`) and are therefore safe to
/// refresh live. Audio-per-minute (`*-transcribe`), realtime, and image models
/// are deliberately excluded: their rows carry per-minute or per-modality
/// figures that positional parsing would misread as token prices and merge over
/// the correct compiled defaults (e.g. a transcribe row's `$0.006 / minute`
/// would overwrite the $10/MTok text-output price). Keep this list token-only;
/// non-token models rely on their compiled defaults in `with_default_openai_pricing`.
const OPENAI_LIVE_PRICE_MODELS: &[&str] = &[
    "gpt-5.6-sol",
    "gpt-5.6-terra",
    "gpt-5.6-luna",
    "gpt-5.5-pro",
    "gpt-5.5",
    "gpt-5.4-mini",
    "gpt-5.4-nano",
    "gpt-5.4-pro",
    "gpt-5.4",
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
const CLAUDE_4_5_OPUS_ALIASES: &[&str] = &["claude-opus-4-5", "claude-opus-4-5-20251101"];
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
const CLAUDE_4_7_OPUS_ALIASES: &[&str] = &["claude-opus-4-7", "anthropic.claude-opus-4-7"];
const CLAUDE_4_8_OPUS_ALIASES: &[&str] = &["claude-opus-4-8", "anthropic.claude-opus-4-8"];
const CLAUDE_4_1_OPUS_ALIASES: &[&str] = &[
    "claude-opus-4-1",
    "claude-opus-4-1-20250805",
    "anthropic.claude-opus-4-1-20250805-v1:0",
];
const CLAUDE_5_SONNET_ALIASES: &[&str] = &["claude-sonnet-5", "anthropic.claude-sonnet-5"];
const CLAUDE_FABLE_5_ALIASES: &[&str] = &["claude-fable-5", "anthropic.claude-fable-5", "fable"];
const CLAUDE_MYTHOS_5_ALIASES: &[&str] = &["claude-mythos-5", "anthropic.claude-mythos-5"];
const CLAUDE_MYTHOS_PREVIEW_ALIASES: &[&str] = &["claude-mythos-preview"];
/// Display-name -> alias mapping used to refresh Claude prices from Anthropic's
/// official pricing page (`ANTHROPIC_PRICING_URL`). The page labels each row by
/// display name, so map those back to the ids Claude Code writes to its logs.
const CLAUDE_LIVE_PRICE_MODELS: &[(&str, &[&str])] = &[
    ("Claude Fable 5", CLAUDE_FABLE_5_ALIASES),
    ("Claude Mythos 5", CLAUDE_MYTHOS_5_ALIASES),
    ("Claude Opus 4.8", CLAUDE_4_8_OPUS_ALIASES),
    ("Claude Opus 4.7", CLAUDE_4_7_OPUS_ALIASES),
    ("Claude Opus 4.6", CLAUDE_4_6_OPUS_ALIASES),
    ("Claude Sonnet 5", CLAUDE_5_SONNET_ALIASES),
    ("Claude Sonnet 4.6", CLAUDE_4_6_SONNET_ALIASES),
    ("Claude Haiku 4.5", CLAUDE_4_5_HAIKU_ALIASES),
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
    pub fn default_catalog_with_live(allow_live_fetch: bool) -> Self {
        let mut catalog = Self::default_catalog();
        if allow_live_fetch {
            // Refresh Claude prices from Anthropic and OpenAI prices from OpenAI.
            // Run both fetches concurrently so total latency stays bounded by a
            // single curl timeout, not the sum; each is best-effort and falls
            // back to the compiled catalog on any error (offline, parse, panic).
            let anthropic = std::thread::spawn(fetch_live_anthropic_pricing);
            let openai = std::thread::spawn(fetch_live_openai_pricing);
            if let Ok(Ok(live_catalog)) = anthropic.join() {
                catalog.merge_from(&live_catalog);
            }
            if let Ok(Ok(live_catalog)) = openai.join() {
                catalog.merge_from(&live_catalog);
            }
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
        // Sonnet: 3.x is flat; 4 / 4.5 carry the historical >200k long-context
        // surcharge; 4.6 and 5 bill the full 1M window at a flat rate (Anthropic).
        let claude_4_sonnet_pricing = ModelPricing::from_per_million(3.0, 15.0, 3.75, 0.3)
            .with_tiered_per_million(Some(6.0), Some(22.5), Some(7.5), Some(0.6));
        let claude_4_6_sonnet_pricing = ModelPricing::from_per_million(3.0, 15.0, 3.75, 0.3);
        let claude_3_sonnet_pricing = ModelPricing::from_per_million(3.0, 15.0, 3.75, 0.3);
        // Sonnet 5 introductory pricing (in effect through 2026-08-31); Anthropic
        // steps it up to (3.0, 15.0, 3.75, 0.3) on 2026-09-01. The live fetch reads
        // whichever Sonnet 5 row the pricing page lists first, so it only tracks the
        // step-up once the page drops or reorders the introductory row; until then
        // this compiled default must be bumped in a release after the intro period.
        let claude_5_sonnet_pricing = ModelPricing::from_per_million(2.0, 10.0, 2.5, 0.2);
        let claude_4_haiku_pricing = ModelPricing::from_per_million(1.0, 5.0, 1.25, 0.1);
        let claude_3_5_haiku_pricing = ModelPricing::from_per_million(0.8, 4.0, 1.0, 0.08);
        let claude_3_haiku_pricing = ModelPricing::from_per_million(0.25, 1.25, 0.3, 0.03);
        let claude_4_opus_pricing = ModelPricing::from_per_million(15.0, 75.0, 18.75, 1.5);
        // Opus 4.5–4.8 share flat $5/$25 pricing (full 1M context at standard rate).
        let claude_opus_5_pricing = ModelPricing::from_per_million(5.0, 25.0, 6.25, 0.5);
        // Fable 5 / Mythos 5 / Mythos Preview: flat $10/$50 (1M context).
        let claude_fable_5_pricing = ModelPricing::from_per_million(10.0, 50.0, 12.5, 1.0);

        self.insert_aliases(CLAUDE_4_OPUS_ALIASES, &claude_4_opus_pricing);
        self.insert_aliases(CLAUDE_4_1_OPUS_ALIASES, &claude_4_opus_pricing);
        self.insert_aliases(CLAUDE_4_5_OPUS_ALIASES, &claude_opus_5_pricing);
        self.insert_aliases(CLAUDE_4_6_OPUS_ALIASES, &claude_opus_5_pricing);
        self.insert_aliases(CLAUDE_4_7_OPUS_ALIASES, &claude_opus_5_pricing);
        self.insert_aliases(CLAUDE_4_8_OPUS_ALIASES, &claude_opus_5_pricing);
        self.insert_aliases(CLAUDE_4_SONNET_ALIASES, &claude_4_sonnet_pricing);
        self.insert_aliases(CLAUDE_4_5_SONNET_ALIASES, &claude_4_sonnet_pricing);
        self.insert_aliases(CLAUDE_4_6_SONNET_ALIASES, &claude_4_6_sonnet_pricing);
        self.insert_aliases(CLAUDE_5_SONNET_ALIASES, &claude_5_sonnet_pricing);
        self.insert_aliases(CLAUDE_3_7_SONNET_ALIASES, &claude_3_sonnet_pricing);
        self.insert_aliases(CLAUDE_3_5_SONNET_ALIASES, &claude_3_sonnet_pricing);
        self.insert_aliases(CLAUDE_4_5_HAIKU_ALIASES, &claude_4_haiku_pricing);
        self.insert_aliases(CLAUDE_3_5_HAIKU_ALIASES, &claude_3_5_haiku_pricing);
        self.insert_aliases(CLAUDE_3_OPUS_ALIASES, &claude_4_opus_pricing);
        self.insert_aliases(CLAUDE_3_HAIKU_ALIASES, &claude_3_haiku_pricing);
        self.insert_aliases(CLAUDE_FABLE_5_ALIASES, &claude_fable_5_pricing);
        self.insert_aliases(CLAUDE_MYTHOS_5_ALIASES, &claude_fable_5_pricing);
        self.insert_aliases(CLAUDE_MYTHOS_PREVIEW_ALIASES, &claude_fable_5_pricing);

        self
    }

    #[must_use]
    pub fn with_default_openai_pricing(mut self) -> Self {
        self.insert_openai_model("gpt-5.6", 5.0, Some(0.5), 30.0);
        self.insert_openai_model("gpt-5.6-sol", 5.0, Some(0.5), 30.0);
        self.insert_openai_model("gpt-5.6-terra", 2.5, Some(0.25), 15.0);
        self.insert_openai_model("gpt-5.6-luna", 1.0, Some(0.1), 6.0);
        self.insert_openai_model("gpt-5.5", 5.0, Some(0.5), 30.0);
        self.insert_openai_model("gpt-5.5-2026-04-23", 5.0, Some(0.5), 30.0);
        self.insert_openai_model("gpt-5.5-pro", 30.0, None, 180.0);
        self.insert_openai_model("gpt-5.5-pro-2026-04-23", 30.0, None, 180.0);
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

fn fetch_pricing_page(url: &str) -> Result<String, String> {
    let output = Command::new("curl")
        .args(["-fsSL", "--max-time", "8", url])
        .output()
        .map_err(|error| format!("failed to run curl for {url}: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("failed to fetch pricing from {url}: {stderr}"));
    }

    String::from_utf8(output.stdout)
        .map_err(|error| format!("pricing response from {url} was not valid UTF-8: {error}"))
}

pub fn fetch_live_openai_pricing() -> Result<PricingCatalog, String> {
    let contents = fetch_pricing_page(OPENAI_PRICING_URL)?;
    let catalog = parse_openai_pricing_page(&contents);
    if catalog.by_model.is_empty() {
        return Err("OpenAI pricing response did not contain parseable token rows".to_owned());
    }
    Ok(catalog)
}

pub fn fetch_live_anthropic_pricing() -> Result<PricingCatalog, String> {
    let contents = fetch_pricing_page(ANTHROPIC_PRICING_URL)?;
    let catalog = parse_anthropic_pricing_page(&contents);
    if catalog.by_model.is_empty() {
        return Err("Anthropic pricing response did not contain parseable token rows".to_owned());
    }
    Ok(catalog)
}

#[must_use]
pub fn parse_anthropic_pricing_page(contents: &str) -> PricingCatalog {
    let mut catalog = PricingCatalog::new();

    for entry in CLAUDE_LIVE_PRICE_MODELS {
        let (display_name, aliases) = *entry;
        // The table lists Sonnet 5 twice (introductory + standard); the first
        // matching row is the currently-effective one, so take the first match.
        // Require the `MTok` price unit so prose mentions can't false-match.
        let Some(line) = contents
            .lines()
            .find(|line| line.contains(display_name) && line.contains("MTok"))
        else {
            continue;
        };
        // Row columns: Base Input | 5m Write | 1h Write | Cache Read | Output.
        let cells = parse_anthropic_price_cells(line);
        let (Some(&input), Some(&cache_write_5m), Some(&cache_read), Some(&output)) =
            (cells.first(), cells.get(1), cells.get(3), cells.get(4))
        else {
            continue;
        };
        let pricing = ModelPricing::from_per_million(input, output, cache_write_5m, cache_read);
        catalog.insert_aliases(aliases, &pricing);
    }

    catalog
}

/// Collect every `$<number>` value from a pricing-table row, in order. Unlike
/// `parse_price_cells`, this tolerates ` / MTok` unit text and `|` separators
/// between cells (Anthropic's Markdown pricing-table format).
fn parse_anthropic_price_cells(line: &str) -> Vec<f64> {
    let mut cells = Vec::new();
    let bytes = line.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'$' {
            index += 1;
            let start = index;
            while index < bytes.len()
                && (bytes[index].is_ascii_digit() || bytes[index] == b'.' || bytes[index] == b',')
            {
                index += 1;
            }
            if let Ok(value) = line[start..index].replace(',', "").parse::<f64>() {
                cells.push(value);
            }
        } else {
            index += 1;
        }
    }
    cells
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
    // OpenAI's flagship pricing table repeats its columns across two context
    // tiers (standard, then long context), each `Input | Cached input | Cache
    // writes | Output`, so those rows expose an even number of price cells and
    // the standard-tier Output is the last column of the first half. Specialized
    // and per-modality tables list a single `Input | Cached input | Output`
    // tier, where Output is the final cell. Deriving the tier width from the cell
    // count keeps Output correct whether or not a "Cache writes" column is
    // present -- "-" entries are recorded as None by parse_price_cells, so every
    // column keeps its position. A fixed index here mispriced the GPT-5.6 family
    // (whose Cache-writes column is populated) at roughly one-fifth of real cost.
    if cells.len() < 3 {
        return None;
    }
    let input = cells.first().copied().flatten()?;
    let cached = cells.get(1).copied().flatten();
    let output_index = if cells.len().is_multiple_of(2) {
        cells.len() / 2 - 1
    } else {
        cells.len() - 1
    };
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
    // Claude Code appends a `[1m]` marker (sometimes doubled) to 1M-context model
    // ids, e.g. `claude-opus-4-8[1m]` / `claude-opus-4-6[1m][1m]`. The suffix selects
    // the context window, not a differently-priced model, so strip it before lookup
    // so ids resolve by exact match instead of relying on fuzzy matching. Display
    // grouping is unaffected (report.rs keys off the raw model string).
    let mut key = model.trim().to_ascii_lowercase();
    while key.ends_with("[1m]") {
        key.truncate(key.len() - "[1m]".len());
    }
    key.trim().to_string()
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

        // Generation 5 + Opus 4.7/4.8 (previously missing) and the [1m] context suffix.
        assert!(catalog.resolve("claude-fable-5").is_some());
        assert!(catalog.resolve("claude-fable-5[1m]").is_some());
        assert!(catalog.resolve("fable").is_some());
        assert!(catalog.resolve("claude-mythos-5").is_some());
        assert!(catalog.resolve("claude-mythos-preview").is_some());
        assert!(catalog.resolve("claude-sonnet-5").is_some());
        assert!(catalog.resolve("claude-sonnet-5[1m]").is_some());
        assert!(catalog.resolve("claude-opus-4-7").is_some());
        assert!(catalog.resolve("claude-opus-4-8").is_some());
        assert!(catalog.resolve("claude-opus-4-8[1m]").is_some());
        assert!(catalog.resolve("claude-opus-4-8[1m][1m]").is_some());
        assert!(catalog.resolve("anthropic/claude-fable-5").is_some());
    }

    #[test]
    fn default_catalog_prices_generation_5_and_1m_suffix() {
        let catalog = PricingCatalog::default_claude_catalog();

        // Fable 5 / Mythos 5: flat $10 / $50 per 1M, 1.25x cache write, 0.1x read.
        let fable = catalog.resolve("claude-fable-5").expect("fable 5 priced");
        assert_close(fable.input_cost_per_token, 10.0 / 1_000_000.0);
        assert_close(fable.output_cost_per_token, 50.0 / 1_000_000.0);
        assert_close(
            fable.cache_creation_input_cost_per_token,
            12.5 / 1_000_000.0,
        );
        assert_close(fable.cache_read_input_cost_per_token, 1.0 / 1_000_000.0);
        assert_eq!(
            catalog.resolve("claude-mythos-5"),
            catalog.resolve("claude-fable-5"),
        );

        // Opus 4.8 is $5 / $25 — not the $15 / $75 Opus 4 rate it previously
        // fuzzy-matched to. The [1m] context suffix must resolve identically.
        let opus_48 = catalog.resolve("claude-opus-4-8").expect("opus 4.8 priced");
        assert_close(opus_48.input_cost_per_token, 5.0 / 1_000_000.0);
        assert_close(opus_48.output_cost_per_token, 25.0 / 1_000_000.0);
        assert_eq!(catalog.resolve("claude-opus-4-8[1m]"), Some(opus_48));
        assert_eq!(catalog.resolve("claude-opus-4-8[1m][1m]"), Some(opus_48));
        assert_ne!(
            catalog
                .resolve("claude-opus-4-8")
                .map(|p| p.input_cost_per_token),
            catalog
                .resolve("claude-opus-4")
                .map(|p| p.input_cost_per_token),
        );

        // Sonnet 5 introductory pricing ($2 / $10 per 1M).
        let sonnet_5 = catalog.resolve("claude-sonnet-5").expect("sonnet 5 priced");
        assert_close(sonnet_5.input_cost_per_token, 2.0 / 1_000_000.0);
        assert_close(sonnet_5.output_cost_per_token, 10.0 / 1_000_000.0);
    }

    #[test]
    fn parses_anthropic_pricing_page_rows_from_official_markdown_shape() {
        // Verbatim column/cell shape from
        // platform.claude.com/docs/en/about-claude/pricing.md.
        let page = "\
| Model | Base Input Tokens | 5m Cache Writes | 1h Cache Writes | Cache Hits & Refreshes | Output Tokens |
| --- | --- | --- | --- | --- | --- |
| Claude Fable 5 | $10 / MTok | $12.50 / MTok | $20 / MTok | $1 / MTok | $50 / MTok |
| Claude Opus 4.8 | $5 / MTok | $6.25 / MTok | $10 / MTok | $0.50 / MTok | $25 / MTok |
| Claude Sonnet 5 [through August 31, 2026](/x) | $2 / MTok | $2.50 / MTok | $4 / MTok | $0.20 / MTok | $10 / MTok |
| Claude Sonnet 5 starting September 1, 2026 | $3 / MTok | $3.75 / MTok | $6 / MTok | $0.30 / MTok | $15 / MTok |
| Claude Haiku 4.5 | $1 / MTok | $1.25 / MTok | $2 / MTok | $0.10 / MTok | $5 / MTok |
";
        let catalog = parse_anthropic_pricing_page(page);

        let fable = catalog.resolve("claude-fable-5").expect("fable parsed");
        assert_close(fable.input_cost_per_token, 10.0 / 1_000_000.0);
        assert_close(fable.output_cost_per_token, 50.0 / 1_000_000.0);
        assert_close(
            fable.cache_creation_input_cost_per_token,
            12.5 / 1_000_000.0,
        );
        assert_close(fable.cache_read_input_cost_per_token, 1.0 / 1_000_000.0);

        let opus = catalog.resolve("claude-opus-4-8").expect("opus 4.8 parsed");
        assert_close(opus.input_cost_per_token, 5.0 / 1_000_000.0);
        assert_close(opus.output_cost_per_token, 25.0 / 1_000_000.0);

        // Sonnet 5 must take the currently-effective introductory row ($2 / $10).
        let sonnet = catalog.resolve("claude-sonnet-5").expect("sonnet 5 parsed");
        assert_close(sonnet.input_cost_per_token, 2.0 / 1_000_000.0);
        assert_close(sonnet.output_cost_per_token, 10.0 / 1_000_000.0);
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

        // GPT-5.6 family (Sol / Terra / Luna) plus the bare alias.
        assert!(catalog.resolve("gpt-5.6").is_some());
        assert!(catalog.resolve("gpt-5.6-sol").is_some());
        assert!(catalog.resolve("gpt-5.6-terra").is_some());
        assert!(catalog.resolve("gpt-5.6-luna").is_some());
        assert!(catalog.resolve("openai/gpt-5.6-terra").is_some());
    }

    #[test]
    fn default_catalog_prices_gpt_5_6_family() {
        let catalog = PricingCatalog::default_catalog();

        // Sol: $5 / $30 in/out, $0.50 cached read; bare gpt-5.6 aliases to Sol.
        let sol = catalog.resolve("gpt-5.6-sol").expect("sol priced");
        assert_close(sol.input_cost_per_token, 5.0 / 1_000_000.0);
        assert_close(sol.output_cost_per_token, 30.0 / 1_000_000.0);
        assert_close(sol.cache_read_input_cost_per_token, 0.5 / 1_000_000.0);
        assert_eq!(catalog.resolve("gpt-5.6"), Some(sol));

        // Terra: $2.50 / $15, $0.25 cached read.
        let terra = catalog.resolve("gpt-5.6-terra").expect("terra priced");
        assert_close(terra.input_cost_per_token, 2.5 / 1_000_000.0);
        assert_close(terra.output_cost_per_token, 15.0 / 1_000_000.0);
        assert_close(terra.cache_read_input_cost_per_token, 0.25 / 1_000_000.0);

        // Luna: $1 / $6, $0.10 cached read.
        let luna = catalog.resolve("gpt-5.6-luna").expect("luna priced");
        assert_close(luna.input_cost_per_token, 1.0 / 1_000_000.0);
        assert_close(luna.output_cost_per_token, 6.0 / 1_000_000.0);
        assert_close(luna.cache_read_input_cost_per_token, 0.1 / 1_000_000.0);
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
        // Mirrors the real developers.openai.com/api/docs/pricing shape: the
        // flagship table lists two context tiers, each `Input | Cached input |
        // Cache writes | Output` (eight price cells per row, "-" where a cell is
        // absent); the specialized table lists a single `Input | Cached input |
        // Output` tier. The populated Cache-writes column is what previously
        // mispriced the GPT-5.6 rows.
        let catalog = parse_openai_pricing_page(
            r#"
            Standard
            Short context Long context
            Model Input Cached input Cache writes Output Input Cached input Cache writes Output
            gpt-5.6-sol$5.00$0.50$6.25$30.00$10.00$1.00$12.50$45.00
            gpt-5.6-terra$2.50$0.25$3.125$15.00$5.00$0.50$6.25$22.50
            gpt-5.6-luna$1.00$0.10$1.25$6.00$2.00$0.20$2.50$9.00
            gpt-5.5$5.00$0.50-$30.00$10.00$1.00-$45.00
            gpt-5.5-pro$30.00--$180.00$60.00--$270.00
            Specialized models
            Category Model Input Cached input Output
            Codex gpt-5.3-codex$1.75$0.175$14.00
            "#,
        );

        // GPT-5.6 flagship rows: Output is the 4th column ($30/$15/$6), not the
        // Cache-writes column ($6.25/$3.125/$1.25).
        let sol = catalog
            .resolve("gpt-5.6-sol")
            .expect("expected gpt-5.6-sol pricing");
        assert_close(sol.input_cost_per_token, 5.0 / 1_000_000.0);
        assert_close(sol.cache_read_input_cost_per_token, 0.5 / 1_000_000.0);
        assert_close(sol.output_cost_per_token, 30.0 / 1_000_000.0);

        let terra = catalog
            .resolve("gpt-5.6-terra")
            .expect("expected gpt-5.6-terra pricing");
        assert_close(terra.output_cost_per_token, 15.0 / 1_000_000.0);

        let luna = catalog
            .resolve("gpt-5.6-luna")
            .expect("expected gpt-5.6-luna pricing");
        assert_close(luna.output_cost_per_token, 6.0 / 1_000_000.0);

        // Dashed Cache-writes cells keep the remaining columns aligned.
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

        // Specialized single-tier row: Output is the final cell.
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

        // Guard the Cache-writes column mapping against the live page: the
        // GPT-5.6 flagship Output prices must survive the round trip, not the
        // Cache-writes values ($6.25 / $3.125 / $1.25).
        let sol = catalog
            .resolve("gpt-5.6-sol")
            .expect("expected live gpt-5.6-sol pricing");
        assert_close(sol.output_cost_per_token, 30.0 / 1_000_000.0);
        let terra = catalog
            .resolve("gpt-5.6-terra")
            .expect("expected live gpt-5.6-terra pricing");
        assert_close(terra.output_cost_per_token, 15.0 / 1_000_000.0);
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
