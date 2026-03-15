use crate::domain::{TokenUsage, UsageEvent, UsageSpeed};
use std::collections::BTreeMap;

const TIER_THRESHOLD_TOKENS: u64 = 200_000;
const DEFAULT_FAST_MULTIPLIER: f64 = 1.0;
const DEFAULT_PROVIDER_PREFIXES: &[&str] = &[
    "anthropic",
    "openrouter",
    "vertex_ai",
    "bedrock",
    "azure",
    "gemini",
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
    pub fn with_default_provider_prefixes(mut self) -> Self {
        if self.provider_prefixes.is_empty() {
            self.provider_prefixes = DEFAULT_PROVIDER_PREFIXES
                .iter()
                .map(|prefix| (*prefix).to_owned())
                .collect();
        }
        self
    }

    pub fn insert(&mut self, model: impl Into<String>, pricing: ModelPricing) {
        let normalized = normalize_model_key(&model.into());
        if normalized.is_empty() {
            return;
        }
        self.by_model.insert(normalized, pricing);
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

        for provider in &self.provider_prefixes {
            let candidate = format!("{provider}/{model}");
            if let Some(pricing) = self.by_model.get(&candidate) {
                return Some(pricing);
            }
        }

        self.fuzzy_match(&model)
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
        catalog.insert("anthropic/claude-3-5-sonnet-20241022", pricing.clone());

        assert!(catalog.resolve("claude-3-5-sonnet-20241022").is_some());
        assert!(catalog.resolve("claude-3-5-sonnet").is_some());
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
