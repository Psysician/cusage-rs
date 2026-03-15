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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WeeklyReport {
    pub weeks: Vec<WeeklyReportWeek>,
    pub totals: WeeklyReportTotals,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WeeklyReportWeek {
    pub week_start: String,
    pub week_end: String,
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
pub struct WeeklyReportTotals {
    pub weeks: usize,
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
pub struct MonthlyReport {
    pub months: Vec<MonthlyReportMonth>,
    pub totals: MonthlyReportTotals,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MonthlyReportMonth {
    pub month: String,
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
pub struct MonthlyReportTotals {
    pub months: usize,
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
pub struct SessionReport {
    pub sessions: Vec<SessionReportSession>,
    pub totals: SessionReportTotals,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SessionReportSession {
    pub session_id: Option<String>,
    pub project: Option<String>,
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
pub struct SessionReportTotals {
    pub sessions: usize,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SessionGroupKey {
    bucket: u8,
    value: String,
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
pub fn build_weekly_report(
    events: &[UsageEvent],
    cost_mode: CostMode,
    pricing: &PricingCatalog,
) -> WeeklyReport {
    let mut grouped = BTreeMap::<i64, WeeklyReportWeek>::new();

    for event in events {
        let week_start_days = utc_week_start_days_from_unix_ms(event.occurred_at_unix_ms);
        let row = grouped
            .entry(week_start_days)
            .or_insert_with(|| WeeklyReportWeek {
                week_start: utc_day_label_from_days_since_epoch(week_start_days),
                week_end: utc_day_label_from_days_since_epoch(
                    utc_week_end_days_from_week_start_days(week_start_days),
                ),
                ..WeeklyReportWeek::default()
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

    let mut report = WeeklyReport {
        weeks: grouped.into_values().collect(),
        totals: WeeklyReportTotals::default(),
    };
    report.totals.weeks = report.weeks.len();

    for week in &report.weeks {
        report.totals.entries = report.totals.entries.saturating_add(week.entries);
        report.totals.input_tokens = report.totals.input_tokens.saturating_add(week.input_tokens);
        report.totals.output_tokens = report
            .totals
            .output_tokens
            .saturating_add(week.output_tokens);
        report.totals.cache_creation_input_tokens = report
            .totals
            .cache_creation_input_tokens
            .saturating_add(week.cache_creation_input_tokens);
        report.totals.cache_read_input_tokens = report
            .totals
            .cache_read_input_tokens
            .saturating_add(week.cache_read_input_tokens);
        report.totals.total_tokens = report.totals.total_tokens.saturating_add(week.total_tokens);
        report.totals.total_cost_usd += week.total_cost_usd;
        report.totals.entries_with_raw_cost = report
            .totals
            .entries_with_raw_cost
            .saturating_add(week.entries_with_raw_cost);
        report.totals.entries_with_calculated_cost = report
            .totals
            .entries_with_calculated_cost
            .saturating_add(week.entries_with_calculated_cost);
        report.totals.entries_with_missing_cost = report
            .totals
            .entries_with_missing_cost
            .saturating_add(week.entries_with_missing_cost);
    }

    report
}

#[must_use]
pub fn build_monthly_report(
    events: &[UsageEvent],
    cost_mode: CostMode,
    pricing: &PricingCatalog,
) -> MonthlyReport {
    let mut grouped = BTreeMap::<i64, MonthlyReportMonth>::new();

    for event in events {
        let month_key = utc_month_key_from_unix_ms(event.occurred_at_unix_ms);
        let row = grouped
            .entry(month_key)
            .or_insert_with(|| MonthlyReportMonth {
                month: utc_month_label_from_month_key(month_key),
                ..MonthlyReportMonth::default()
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

    let mut report = MonthlyReport {
        months: grouped.into_values().collect(),
        totals: MonthlyReportTotals::default(),
    };
    report.totals.months = report.months.len();

    for month in &report.months {
        report.totals.entries = report.totals.entries.saturating_add(month.entries);
        report.totals.input_tokens = report
            .totals
            .input_tokens
            .saturating_add(month.input_tokens);
        report.totals.output_tokens = report
            .totals
            .output_tokens
            .saturating_add(month.output_tokens);
        report.totals.cache_creation_input_tokens = report
            .totals
            .cache_creation_input_tokens
            .saturating_add(month.cache_creation_input_tokens);
        report.totals.cache_read_input_tokens = report
            .totals
            .cache_read_input_tokens
            .saturating_add(month.cache_read_input_tokens);
        report.totals.total_tokens = report
            .totals
            .total_tokens
            .saturating_add(month.total_tokens);
        report.totals.total_cost_usd += month.total_cost_usd;
        report.totals.entries_with_raw_cost = report
            .totals
            .entries_with_raw_cost
            .saturating_add(month.entries_with_raw_cost);
        report.totals.entries_with_calculated_cost = report
            .totals
            .entries_with_calculated_cost
            .saturating_add(month.entries_with_calculated_cost);
        report.totals.entries_with_missing_cost = report
            .totals
            .entries_with_missing_cost
            .saturating_add(month.entries_with_missing_cost);
    }

    report
}

#[must_use]
pub fn build_session_report(
    events: &[UsageEvent],
    cost_mode: CostMode,
    pricing: &PricingCatalog,
) -> SessionReport {
    let mut grouped = BTreeMap::<SessionGroupKey, SessionReportSession>::new();

    for event in events {
        let key = session_group_key(event);
        let row = grouped
            .entry(key)
            .or_insert_with(|| SessionReportSession {
                session_id: normalized_optional_string(event.session_id.as_deref()),
                project: normalized_optional_string(event.project.as_deref()),
                ..SessionReportSession::default()
            });

        if row.session_id.is_none() {
            row.session_id = normalized_optional_string(event.session_id.as_deref());
        }
        if row.project.is_none() {
            row.project = normalized_optional_string(event.project.as_deref());
        }

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

    let mut report = SessionReport {
        sessions: grouped.into_values().collect(),
        totals: SessionReportTotals::default(),
    };
    report.totals.sessions = report.sessions.len();

    for session in &report.sessions {
        report.totals.entries = report.totals.entries.saturating_add(session.entries);
        report.totals.input_tokens = report
            .totals
            .input_tokens
            .saturating_add(session.input_tokens);
        report.totals.output_tokens = report
            .totals
            .output_tokens
            .saturating_add(session.output_tokens);
        report.totals.cache_creation_input_tokens = report
            .totals
            .cache_creation_input_tokens
            .saturating_add(session.cache_creation_input_tokens);
        report.totals.cache_read_input_tokens = report
            .totals
            .cache_read_input_tokens
            .saturating_add(session.cache_read_input_tokens);
        report.totals.total_tokens = report
            .totals
            .total_tokens
            .saturating_add(session.total_tokens);
        report.totals.total_cost_usd += session.total_cost_usd;
        report.totals.entries_with_raw_cost = report
            .totals
            .entries_with_raw_cost
            .saturating_add(session.entries_with_raw_cost);
        report.totals.entries_with_calculated_cost = report
            .totals
            .entries_with_calculated_cost
            .saturating_add(session.entries_with_calculated_cost);
        report.totals.entries_with_missing_cost = report
            .totals
            .entries_with_missing_cost
            .saturating_add(session.entries_with_missing_cost);
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

#[must_use]
pub fn render_weekly_report_json(
    report: &WeeklyReport,
    discovery_warning_count: usize,
    parse_warning_count: usize,
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"mode\": \"weekly\",\n");

    if report.weeks.is_empty() {
        out.push_str("  \"weeks\": [],\n");
    } else {
        out.push_str("  \"weeks\": [\n");
        for (index, week) in report.weeks.iter().enumerate() {
            out.push_str("    {\n");
            out.push_str(&format!(
                "      \"week_start\": \"{}\",\n",
                json_escape(&week.week_start)
            ));
            out.push_str(&format!(
                "      \"week_end\": \"{}\",\n",
                json_escape(&week.week_end)
            ));
            out.push_str(&format!("      \"entries\": {},\n", week.entries));
            out.push_str("      \"tokens\": {\n");
            out.push_str(&format!("        \"input\": {},\n", week.input_tokens));
            out.push_str(&format!("        \"output\": {},\n", week.output_tokens));
            out.push_str(&format!(
                "        \"cache_creation_input\": {},\n",
                week.cache_creation_input_tokens
            ));
            out.push_str(&format!(
                "        \"cache_read_input\": {},\n",
                week.cache_read_input_tokens
            ));
            out.push_str(&format!("        \"total\": {}\n", week.total_tokens));
            out.push_str("      },\n");
            out.push_str("      \"cost\": {\n");
            out.push_str(&format!(
                "        \"usd\": {},\n",
                json_number(week.total_cost_usd)
            ));
            out.push_str(&format!(
                "        \"raw_entries\": {},\n",
                week.entries_with_raw_cost
            ));
            out.push_str(&format!(
                "        \"calculated_entries\": {},\n",
                week.entries_with_calculated_cost
            ));
            out.push_str(&format!(
                "        \"missing_entries\": {}\n",
                week.entries_with_missing_cost
            ));
            out.push_str("      }\n");
            out.push_str("    }");
            if index + 1 != report.weeks.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ],\n");
    }

    out.push_str("  \"totals\": {\n");
    out.push_str(&format!("    \"weeks\": {},\n", report.totals.weeks));
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
pub fn render_weekly_report_table(
    report: &WeeklyReport,
    discovery_warning_count: usize,
    parse_warning_count: usize,
) -> String {
    let mut lines = Vec::new();
    lines.push(
        "WEEK_START WEEK_END   ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD"
            .to_owned(),
    );

    for week in &report.weeks {
        lines.push(format!(
            "{} {} {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
            week.week_start,
            week.week_end,
            week.entries,
            week.input_tokens,
            week.output_tokens,
            week.cache_creation_input_tokens,
            week.cache_read_input_tokens,
            week.total_tokens,
            json_number(week.total_cost_usd)
        ));
    }

    lines.push(format!(
        "TOTAL                {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
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

#[must_use]
pub fn render_monthly_report_json(
    report: &MonthlyReport,
    discovery_warning_count: usize,
    parse_warning_count: usize,
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"mode\": \"monthly\",\n");

    if report.months.is_empty() {
        out.push_str("  \"months\": [],\n");
    } else {
        out.push_str("  \"months\": [\n");
        for (index, month) in report.months.iter().enumerate() {
            out.push_str("    {\n");
            out.push_str(&format!(
                "      \"month\": \"{}\",\n",
                json_escape(&month.month)
            ));
            out.push_str(&format!("      \"entries\": {},\n", month.entries));
            out.push_str("      \"tokens\": {\n");
            out.push_str(&format!("        \"input\": {},\n", month.input_tokens));
            out.push_str(&format!("        \"output\": {},\n", month.output_tokens));
            out.push_str(&format!(
                "        \"cache_creation_input\": {},\n",
                month.cache_creation_input_tokens
            ));
            out.push_str(&format!(
                "        \"cache_read_input\": {},\n",
                month.cache_read_input_tokens
            ));
            out.push_str(&format!("        \"total\": {}\n", month.total_tokens));
            out.push_str("      },\n");
            out.push_str("      \"cost\": {\n");
            out.push_str(&format!(
                "        \"usd\": {},\n",
                json_number(month.total_cost_usd)
            ));
            out.push_str(&format!(
                "        \"raw_entries\": {},\n",
                month.entries_with_raw_cost
            ));
            out.push_str(&format!(
                "        \"calculated_entries\": {},\n",
                month.entries_with_calculated_cost
            ));
            out.push_str(&format!(
                "        \"missing_entries\": {}\n",
                month.entries_with_missing_cost
            ));
            out.push_str("      }\n");
            out.push_str("    }");
            if index + 1 != report.months.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ],\n");
    }

    out.push_str("  \"totals\": {\n");
    out.push_str(&format!("    \"months\": {},\n", report.totals.months));
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
pub fn render_monthly_report_table(
    report: &MonthlyReport,
    discovery_warning_count: usize,
    parse_warning_count: usize,
) -> String {
    let mut lines = Vec::new();
    lines.push("MONTH    ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD".to_owned());

    for month in &report.months {
        lines.push(format!(
            "{} {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
            month.month,
            month.entries,
            month.input_tokens,
            month.output_tokens,
            month.cache_creation_input_tokens,
            month.cache_read_input_tokens,
            month.total_tokens,
            json_number(month.total_cost_usd)
        ));
    }

    lines.push(format!(
        "TOTAL    {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
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

#[must_use]
pub fn render_session_report_json(
    report: &SessionReport,
    discovery_warning_count: usize,
    parse_warning_count: usize,
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"mode\": \"session\",\n");

    if report.sessions.is_empty() {
        out.push_str("  \"sessions\": [],\n");
    } else {
        out.push_str("  \"sessions\": [\n");
        for (index, session) in report.sessions.iter().enumerate() {
            out.push_str("    {\n");
            out.push_str(&format!(
                "      \"session_id\": {},\n",
                json_optional_string(session.session_id.as_deref())
            ));
            out.push_str(&format!(
                "      \"project\": {},\n",
                json_optional_string(session.project.as_deref())
            ));
            out.push_str(&format!("      \"entries\": {},\n", session.entries));
            out.push_str("      \"tokens\": {\n");
            out.push_str(&format!("        \"input\": {},\n", session.input_tokens));
            out.push_str(&format!("        \"output\": {},\n", session.output_tokens));
            out.push_str(&format!(
                "        \"cache_creation_input\": {},\n",
                session.cache_creation_input_tokens
            ));
            out.push_str(&format!(
                "        \"cache_read_input\": {},\n",
                session.cache_read_input_tokens
            ));
            out.push_str(&format!("        \"total\": {}\n", session.total_tokens));
            out.push_str("      },\n");
            out.push_str("      \"cost\": {\n");
            out.push_str(&format!(
                "        \"usd\": {},\n",
                json_number(session.total_cost_usd)
            ));
            out.push_str(&format!(
                "        \"raw_entries\": {},\n",
                session.entries_with_raw_cost
            ));
            out.push_str(&format!(
                "        \"calculated_entries\": {},\n",
                session.entries_with_calculated_cost
            ));
            out.push_str(&format!(
                "        \"missing_entries\": {}\n",
                session.entries_with_missing_cost
            ));
            out.push_str("      }\n");
            out.push_str("    }");
            if index + 1 != report.sessions.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ],\n");
    }

    out.push_str("  \"totals\": {\n");
    out.push_str(&format!("    \"sessions\": {},\n", report.totals.sessions));
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
pub fn render_session_report_table(
    report: &SessionReport,
    discovery_warning_count: usize,
    parse_warning_count: usize,
) -> String {
    let mut lines = Vec::new();
    lines.push(
        "SESSION_ID          PROJECT             ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD"
            .to_owned(),
    );

    for session in &report.sessions {
        lines.push(format!(
            "{:<19} {:<19} {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
            session.session_id.as_deref().unwrap_or("-"),
            session.project.as_deref().unwrap_or("-"),
            session.entries,
            session.input_tokens,
            session.output_tokens,
            session.cache_creation_input_tokens,
            session.cache_read_input_tokens,
            session.total_tokens,
            json_number(session.total_cost_usd)
        ));
    }

    lines.push(format!(
        "TOTAL                                  {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
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
    utc_day_label_from_days_since_epoch(unix_ms_to_days_since_epoch(unix_ms))
}

fn utc_day_label_from_days_since_epoch(days_since_epoch: i64) -> String {
    let (year, month, day) = civil_from_days(days_since_epoch);
    format!("{year:04}-{month:02}-{day:02}")
}

fn unix_ms_to_days_since_epoch(unix_ms: i64) -> i64 {
    unix_ms.div_euclid(86_400_000)
}

fn utc_week_start_days_from_unix_ms(unix_ms: i64) -> i64 {
    let days_since_epoch = unix_ms_to_days_since_epoch(unix_ms);
    let weekday_monday_based = (days_since_epoch + 3).rem_euclid(7);
    days_since_epoch - weekday_monday_based
}

fn utc_week_end_days_from_week_start_days(week_start_days: i64) -> i64 {
    week_start_days.saturating_add(6)
}

fn utc_month_key_from_unix_ms(unix_ms: i64) -> i64 {
    let days_since_epoch = unix_ms_to_days_since_epoch(unix_ms);
    let (year, month, _) = civil_from_days(days_since_epoch);
    year.saturating_mul(12)
        .saturating_add(i64::from(month).saturating_sub(1))
}

fn utc_month_label_from_month_key(month_key: i64) -> String {
    let year = month_key.div_euclid(12);
    let month = month_key.rem_euclid(12) + 1;
    format!("{year:04}-{month:02}")
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

fn session_group_key(event: &UsageEvent) -> SessionGroupKey {
    let session_id = normalized_optional_string(event.session_id.as_deref());
    if let Some(value) = session_id {
        return SessionGroupKey { bucket: 0, value };
    }

    let project = normalized_optional_string(event.project.as_deref());
    if let Some(value) = project {
        return SessionGroupKey { bucket: 1, value };
    }

    SessionGroupKey {
        bucket: 2,
        value: String::new(),
    }
}

fn normalized_optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn json_optional_string(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    }
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

    #[test]
    fn builds_weekly_buckets_with_monday_start() {
        let mut pricing = PricingCatalog::new();
        pricing.insert(
            "claude-sonnet",
            ModelPricing::from_per_million(1.0, 1.0, 1.0, 1.0),
        );

        let events = vec![
            test_event(
                1_772_971_200_000,
                Some("claude-sonnet"),
                Some(0.2),
                10,
                0,
                0,
                0,
            ),
            test_event(
                1_773_057_600_000,
                Some("claude-sonnet"),
                None,
                1_000_000,
                0,
                0,
                0,
            ),
            test_event(1_773_057_600_001, Some("unknown-model"), None, 3, 4, 0, 0),
        ];

        let report = build_weekly_report(&events, CostMode::Auto, &pricing);

        assert_eq!(report.weeks.len(), 2);
        assert_eq!(report.weeks[0].week_start, "2026-03-02");
        assert_eq!(report.weeks[0].week_end, "2026-03-08");
        assert_eq!(report.weeks[0].entries, 1);
        assert_eq!(report.weeks[1].week_start, "2026-03-09");
        assert_eq!(report.weeks[1].week_end, "2026-03-15");
        assert_eq!(report.weeks[1].entries, 2);
        assert_eq!(report.weeks[1].entries_with_calculated_cost, 1);
        assert_eq!(report.weeks[1].entries_with_missing_cost, 1);
        assert_eq!(report.totals.entries, 3);
        assert_eq!(report.totals.weeks, 2);
        assert_eq!(report.totals.entries_with_raw_cost, 1);
        assert_eq!(report.totals.entries_with_calculated_cost, 1);
        assert_eq!(report.totals.entries_with_missing_cost, 1);
    }

    #[test]
    fn renders_weekly_json_shape_deterministically() {
        let report = WeeklyReport {
            weeks: vec![WeeklyReportWeek {
                week_start: "2026-03-09".to_owned(),
                week_end: "2026-03-15".to_owned(),
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
            totals: WeeklyReportTotals {
                weeks: 1,
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

        let rendered = render_weekly_report_json(&report, 1, 2);

        assert!(rendered.contains("\"mode\": \"weekly\""));
        assert!(rendered.contains("\"week_start\": \"2026-03-09\""));
        assert!(rendered.contains("\"week_end\": \"2026-03-15\""));
        assert!(rendered.contains("\"usd\": 0.123456"));
        assert!(rendered.contains("\"discovery\": 1"));
        assert!(rendered.contains("\"parse\": 2"));
    }

    #[test]
    fn builds_monthly_buckets_in_utc_order() {
        let mut pricing = PricingCatalog::new();
        pricing.insert(
            "claude-sonnet",
            ModelPricing::from_per_million(1.0, 1.0, 1.0, 1.0),
        );

        let events = vec![
            test_event(
                1_772_319_600_000,
                Some("claude-sonnet"),
                Some(0.2),
                10,
                0,
                0,
                0,
            ),
            test_event(
                1_772_323_200_000,
                Some("claude-sonnet"),
                None,
                1_000_000,
                0,
                0,
                0,
            ),
            test_event(1_775_001_600_000, Some("unknown-model"), None, 3, 4, 0, 0),
        ];

        let report = build_monthly_report(&events, CostMode::Auto, &pricing);

        assert_eq!(report.months.len(), 3);
        assert_eq!(report.months[0].month, "2026-02");
        assert_eq!(report.months[0].entries, 1);
        assert_eq!(report.months[1].month, "2026-03");
        assert_eq!(report.months[1].entries, 1);
        assert_eq!(report.months[1].entries_with_calculated_cost, 1);
        assert_eq!(report.months[2].month, "2026-04");
        assert_eq!(report.months[2].entries, 1);
        assert_eq!(report.months[2].entries_with_missing_cost, 1);
        assert_eq!(report.totals.entries, 3);
        assert_eq!(report.totals.months, 3);
        assert_eq!(report.totals.entries_with_raw_cost, 1);
        assert_eq!(report.totals.entries_with_calculated_cost, 1);
        assert_eq!(report.totals.entries_with_missing_cost, 1);
    }

    #[test]
    fn renders_monthly_json_shape_deterministically() {
        let report = MonthlyReport {
            months: vec![MonthlyReportMonth {
                month: "2026-03".to_owned(),
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
            totals: MonthlyReportTotals {
                months: 1,
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

        let rendered = render_monthly_report_json(&report, 1, 2);

        assert!(rendered.contains("\"mode\": \"monthly\""));
        assert!(rendered.contains("\"month\": \"2026-03\""));
        assert!(rendered.contains("\"usd\": 0.123456"));
        assert!(rendered.contains("\"discovery\": 1"));
        assert!(rendered.contains("\"parse\": 2"));
    }

    #[test]
    fn builds_session_buckets_in_stable_order() {
        let mut pricing = PricingCatalog::new();
        pricing.insert(
            "claude-sonnet",
            ModelPricing::from_per_million(1.0, 1.0, 1.0, 1.0),
        );

        let events = vec![
            test_event_with_identity(
                1_773_057_600_000,
                Some("session-b"),
                Some("beta"),
                Some("claude-sonnet"),
                None,
                1_000_000,
                0,
                0,
                0,
            ),
            test_event_with_identity(
                1_772_971_200_000,
                Some("session-a"),
                Some("alpha"),
                Some("claude-sonnet"),
                Some(0.2),
                10,
                0,
                0,
                0,
            ),
            test_event_with_identity(
                1_773_057_600_001,
                None,
                Some("alpha"),
                Some("unknown-model"),
                None,
                3,
                4,
                0,
                0,
            ),
            test_event_with_identity(
                1_773_057_600_002,
                Some("session-b"),
                None,
                Some("unknown-model"),
                None,
                5,
                6,
                0,
                0,
            ),
        ];

        let report = build_session_report(&events, CostMode::Auto, &pricing);

        assert_eq!(report.sessions.len(), 3);
        assert_eq!(report.sessions[0].session_id.as_deref(), Some("session-a"));
        assert_eq!(report.sessions[0].project.as_deref(), Some("alpha"));
        assert_eq!(report.sessions[0].entries, 1);
        assert_eq!(report.sessions[1].session_id.as_deref(), Some("session-b"));
        assert_eq!(report.sessions[1].project.as_deref(), Some("beta"));
        assert_eq!(report.sessions[1].entries, 2);
        assert_eq!(report.sessions[1].entries_with_calculated_cost, 1);
        assert_eq!(report.sessions[1].entries_with_missing_cost, 1);
        assert_eq!(report.sessions[2].session_id, None);
        assert_eq!(report.sessions[2].project.as_deref(), Some("alpha"));
        assert_eq!(report.sessions[2].entries, 1);
        assert_eq!(report.totals.entries, 4);
        assert_eq!(report.totals.sessions, 3);
        assert_eq!(report.totals.entries_with_raw_cost, 1);
        assert_eq!(report.totals.entries_with_calculated_cost, 1);
        assert_eq!(report.totals.entries_with_missing_cost, 2);
    }

    #[test]
    fn renders_session_json_shape_deterministically() {
        let report = SessionReport {
            sessions: vec![SessionReportSession {
                session_id: Some("session-a".to_owned()),
                project: Some("alpha".to_owned()),
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
            totals: SessionReportTotals {
                sessions: 1,
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

        let rendered = render_session_report_json(&report, 1, 2);

        assert!(rendered.contains("\"mode\": \"session\""));
        assert!(rendered.contains("\"session_id\": \"session-a\""));
        assert!(rendered.contains("\"project\": \"alpha\""));
        assert!(rendered.contains("\"usd\": 0.123456"));
        assert!(rendered.contains("\"discovery\": 1"));
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

    fn test_event_with_identity(
        occurred_at_unix_ms: i64,
        session_id: Option<&str>,
        project: Option<&str>,
        model: Option<&str>,
        raw_cost_usd: Option<f64>,
        input_tokens: u64,
        output_tokens: u64,
        cache_creation_input_tokens: u64,
        cache_read_input_tokens: u64,
    ) -> UsageEvent {
        let mut event = test_event(
            occurred_at_unix_ms,
            model,
            raw_cost_usd,
            input_tokens,
            output_tokens,
            cache_creation_input_tokens,
            cache_read_input_tokens,
        );
        event.session_id = session_id.map(str::to_owned);
        event.project = project.map(str::to_owned);
        event
    }
}
