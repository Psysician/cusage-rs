use crate::domain::UsageEvent;
use crate::pricing::{
    CostMode, CostSource, PricingCatalog, resolve_event_cost, total_tokens_for_usage,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DailyReport {
    pub days: Vec<DailyReportDay>,
    pub totals: DailyReportTotals,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DailyReportDay {
    pub date: String,
    pub entries: usize,
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DailyReportTotals {
    pub days: usize,
    pub entries: usize,
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
pub fn build_daily_report(
    events: &[UsageEvent],
    cost_mode: CostMode,
    pricing: &PricingCatalog,
) -> DailyReport {
    let mut grouped = BTreeMap::<String, DailyReportDay>::new();

    for event in events {
        let day_label = utc_day_label_from_unix_ms(event.occurred_at_unix_ms);
        let row = grouped
            .entry(day_label.clone())
            .or_insert_with(|| DailyReportDay {
                date: day_label,
                ..DailyReportDay::default()
            });

        row.entries = row.entries.saturating_add(1);
        row.input_tokens = row.input_tokens.saturating_add(event.usage.input_tokens);
        row.output_tokens = row.output_tokens.saturating_add(event.usage.output_tokens);
        row.cache_creation_input_tokens = row
            .cache_creation_input_tokens
            .saturating_add(event.usage.cache_creation_input_tokens);
        row.cache_read_input_tokens = row
            .cache_read_input_tokens
            .saturating_add(event.usage.cache_read_input_tokens);
        row.total_tokens = row
            .total_tokens
            .saturating_add(total_tokens_for_usage(&event.usage));

        let resolved = resolve_event_cost(event, cost_mode, pricing);
        row.total_cost_usd += resolved.cost_usd;
        match resolved.source {
            CostSource::Raw => {
                row.entries_with_raw_cost = row.entries_with_raw_cost.saturating_add(1)
            }
            CostSource::Calculated => {
                row.entries_with_calculated_cost =
                    row.entries_with_calculated_cost.saturating_add(1)
            }
            CostSource::Missing => {
                row.entries_with_missing_cost = row.entries_with_missing_cost.saturating_add(1)
            }
        }
    }

    let mut report = DailyReport {
        days: grouped.into_values().collect(),
        totals: DailyReportTotals::default(),
    };
    report.totals.days = report.days.len();

    for day in &report.days {
        report.totals.entries = report.totals.entries.saturating_add(day.entries);
        report.totals.input_tokens = report.totals.input_tokens.saturating_add(day.input_tokens);
        report.totals.output_tokens = report
            .totals
            .output_tokens
            .saturating_add(day.output_tokens);
        report.totals.cache_creation_input_tokens = report
            .totals
            .cache_creation_input_tokens
            .saturating_add(day.cache_creation_input_tokens);
        report.totals.cache_read_input_tokens = report
            .totals
            .cache_read_input_tokens
            .saturating_add(day.cache_read_input_tokens);
        report.totals.total_tokens = report.totals.total_tokens.saturating_add(day.total_tokens);
        report.totals.total_cost_usd += day.total_cost_usd;
        report.totals.entries_with_raw_cost = report
            .totals
            .entries_with_raw_cost
            .saturating_add(day.entries_with_raw_cost);
        report.totals.entries_with_calculated_cost = report
            .totals
            .entries_with_calculated_cost
            .saturating_add(day.entries_with_calculated_cost);
        report.totals.entries_with_missing_cost = report
            .totals
            .entries_with_missing_cost
            .saturating_add(day.entries_with_missing_cost);
    }

    report
}

#[must_use]
pub fn render_daily_report_json(
    report: &DailyReport,
    discovery_warning_count: usize,
    parse_warning_count: usize,
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"mode\": \"daily\",\n");

    if report.days.is_empty() {
        out.push_str("  \"days\": [],\n");
    } else {
        out.push_str("  \"days\": [\n");
        for (index, day) in report.days.iter().enumerate() {
            out.push_str("    {\n");
            out.push_str(&format!(
                "      \"date\": \"{}\",\n",
                json_escape(&day.date)
            ));
            out.push_str(&format!("      \"entries\": {},\n", day.entries));
            out.push_str("      \"tokens\": {\n");
            out.push_str(&format!("        \"input\": {},\n", day.input_tokens));
            out.push_str(&format!("        \"output\": {},\n", day.output_tokens));
            out.push_str(&format!(
                "        \"cache_creation_input\": {},\n",
                day.cache_creation_input_tokens
            ));
            out.push_str(&format!(
                "        \"cache_read_input\": {},\n",
                day.cache_read_input_tokens
            ));
            out.push_str(&format!("        \"total\": {}\n", day.total_tokens));
            out.push_str("      },\n");
            out.push_str("      \"cost\": {\n");
            out.push_str(&format!(
                "        \"usd\": {},\n",
                json_number(day.total_cost_usd)
            ));
            out.push_str(&format!(
                "        \"raw_entries\": {},\n",
                day.entries_with_raw_cost
            ));
            out.push_str(&format!(
                "        \"calculated_entries\": {},\n",
                day.entries_with_calculated_cost
            ));
            out.push_str(&format!(
                "        \"missing_entries\": {}\n",
                day.entries_with_missing_cost
            ));
            out.push_str("      }\n");
            out.push_str("    }");
            if index + 1 != report.days.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ],\n");
    }

    out.push_str("  \"totals\": {\n");
    out.push_str(&format!("    \"days\": {},\n", report.totals.days));
    out.push_str(&format!("    \"entries\": {},\n", report.totals.entries));
    out.push_str("    \"tokens\": {\n");
    out.push_str(&format!(
        "      \"input\": {},\n",
        report.totals.input_tokens
    ));
    out.push_str(&format!(
        "      \"output\": {},\n",
        report.totals.output_tokens
    ));
    out.push_str(&format!(
        "      \"cache_creation_input\": {},\n",
        report.totals.cache_creation_input_tokens
    ));
    out.push_str(&format!(
        "      \"cache_read_input\": {},\n",
        report.totals.cache_read_input_tokens
    ));
    out.push_str(&format!(
        "      \"total\": {}\n",
        report.totals.total_tokens
    ));
    out.push_str("    },\n");
    out.push_str("    \"cost\": {\n");
    out.push_str(&format!(
        "      \"usd\": {},\n",
        json_number(report.totals.total_cost_usd)
    ));
    out.push_str(&format!(
        "      \"raw_entries\": {},\n",
        report.totals.entries_with_raw_cost
    ));
    out.push_str(&format!(
        "      \"calculated_entries\": {},\n",
        report.totals.entries_with_calculated_cost
    ));
    out.push_str(&format!(
        "      \"missing_entries\": {}\n",
        report.totals.entries_with_missing_cost
    ));
    out.push_str("    }\n");
    out.push_str("  },\n");

    out.push_str("  \"warnings\": {\n");
    out.push_str(&format!(
        "    \"discovery\": {},\n",
        discovery_warning_count
    ));
    out.push_str(&format!("    \"parse\": {}\n", parse_warning_count));
    out.push_str("  }\n");
    out.push_str("}\n");

    out
}

#[must_use]
pub fn render_daily_report_table(
    report: &DailyReport,
    discovery_warning_count: usize,
    parse_warning_count: usize,
) -> String {
    let mut lines = Vec::new();
    lines.push("DATE       ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD".to_owned());

    for day in &report.days {
        lines.push(format!(
            "{} {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
            day.date,
            day.entries,
            day.input_tokens,
            day.output_tokens,
            day.cache_creation_input_tokens,
            day.cache_read_input_tokens,
            day.total_tokens,
            json_number(day.total_cost_usd)
        ));
    }

    lines.push(format!(
        "TOTAL      {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
        report.totals.entries,
        report.totals.input_tokens,
        report.totals.output_tokens,
        report.totals.cache_creation_input_tokens,
        report.totals.cache_read_input_tokens,
        report.totals.total_tokens,
        json_number(report.totals.total_cost_usd)
    ));
    lines.push(format!(
        "WARNINGS discovery={} parse={}",
        discovery_warning_count, parse_warning_count
    ));

    lines.join("\n") + "\n"
}

fn utc_day_label_from_unix_ms(unix_ms: i64) -> String {
    let days_since_epoch = unix_ms.div_euclid(86_400_000);
    let (year, month, day) = civil_from_days(days_since_epoch);
    format!("{year:04}-{month:02}-{day:02}")
}

fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };

    (year, month as u32, day as u32)
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => escaped.push_str(&format!("\\u{:04X}", c as u32)),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn json_number(value: f64) -> String {
    let mut normalized = if value.is_finite() { value } else { 0.0 };
    if normalized.abs() < 0.000_000_000_001 {
        normalized = 0.0;
    }

    let mut rendered = format!("{normalized:.6}");
    while rendered.contains('.') && rendered.ends_with('0') {
        rendered.pop();
    }
    if rendered.ends_with('.') {
        rendered.push('0');
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{EventKind, EventOrigin, TokenUsage, UsageSpeed};
    use crate::pricing::ModelPricing;
    use std::path::PathBuf;

    #[test]
    fn builds_daily_buckets_in_utc_order() {
        let mut pricing = PricingCatalog::new();
        pricing.insert(
            "claude-sonnet",
            ModelPricing::from_per_million(1.0, 1.0, 1.0, 1.0),
        );

        let events = vec![
            test_event(
                1_710_028_800_000,
                Some("claude-sonnet"),
                Some(0.2),
                10,
                5,
                0,
                0,
            ),
            test_event(1_709_942_400_000, Some("claude-sonnet"), None, 1, 2, 3, 4),
            test_event(1_709_950_400_000, Some("unknown-model"), None, 7, 8, 0, 0),
        ];

        let report = build_daily_report(&events, CostMode::Auto, &pricing);

        assert_eq!(report.days.len(), 2);
        assert_eq!(report.days[0].date, "2024-03-09");
        assert_eq!(report.days[0].entries, 2);
        assert_eq!(report.days[0].total_tokens, 25);
        assert_eq!(report.days[1].date, "2024-03-10");
        assert_eq!(report.days[1].entries, 1);
        assert_eq!(report.totals.entries, 3);
        assert_eq!(report.totals.days, 2);
        assert_eq!(report.totals.entries_with_raw_cost, 1);
        assert_eq!(report.totals.entries_with_calculated_cost, 1);
        assert_eq!(report.totals.entries_with_missing_cost, 1);
    }

    #[test]
    fn renders_deterministic_json_shape() {
        let report = DailyReport {
            days: vec![DailyReportDay {
                date: "2026-03-10".to_owned(),
                entries: 2,
                input_tokens: 5,
                output_tokens: 6,
                cache_creation_input_tokens: 7,
                cache_read_input_tokens: 8,
                total_tokens: 26,
                total_cost_usd: 0.123_456,
                entries_with_raw_cost: 1,
                entries_with_calculated_cost: 1,
                entries_with_missing_cost: 0,
            }],
            totals: DailyReportTotals {
                days: 1,
                entries: 2,
                input_tokens: 5,
                output_tokens: 6,
                cache_creation_input_tokens: 7,
                cache_read_input_tokens: 8,
                total_tokens: 26,
                total_cost_usd: 0.123_456,
                entries_with_raw_cost: 1,
                entries_with_calculated_cost: 1,
                entries_with_missing_cost: 0,
            },
        };

        let rendered = render_daily_report_json(&report, 0, 2);

        assert!(rendered.contains("\"mode\": \"daily\""));
        assert!(rendered.contains("\"date\": \"2026-03-10\""));
        assert!(rendered.contains("\"usd\": 0.123456"));
        assert!(rendered.contains("\"parse\": 2"));
    }

    fn test_event(
        occurred_at_unix_ms: i64,
        model: Option<&str>,
        raw_cost_usd: Option<f64>,
        input_tokens: u64,
        output_tokens: u64,
        cache_creation_input_tokens: u64,
        cache_read_input_tokens: u64,
    ) -> UsageEvent {
        UsageEvent {
            origin: EventOrigin {
                file: PathBuf::from("/tmp/session.jsonl"),
                line_number: 1,
            },
            occurred_at_unix_ms,
            event_kind: EventKind::Assistant,
            session_id: Some("s1".to_owned()),
            project: Some("demo".to_owned()),
            model: model.map(str::to_owned),
            speed: Some(UsageSpeed::Standard),
            usage: TokenUsage::new(
                input_tokens,
                output_tokens,
                cache_creation_input_tokens,
                cache_read_input_tokens,
                None,
            ),
            raw_cost_usd,
        }
    }
}
