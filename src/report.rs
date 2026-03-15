use crate::domain::UsageEvent;
use crate::pricing::{
    CostMode, CostSource, PricingCatalog, resolve_event_cost, total_tokens_for_usage,
};
use std::collections::BTreeMap;

const DEFAULT_BLOCK_WINDOW_MS: i64 = 5 * 60 * 60 * 1000;

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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlocksReport {
    pub blocks: Vec<BlocksReportBlock>,
    pub totals: BlocksReportTotals,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlocksReportBlock {
    pub block_start: String,
    pub block_end: String,
    pub first_event_at: String,
    pub last_event_at: String,
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
pub struct BlocksReportTotals {
    pub blocks: usize,
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
pub struct StatuslineReport {
    pub model: Option<String>,
    pub session_cost_usd: f64,
    pub today_cost_usd: f64,
    pub session_input_tokens: u64,
    pub active_block: Option<StatuslineReportBlock>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StatuslineReportBlock {
    pub block_start: String,
    pub block_end: String,
    pub cost_usd: f64,
    pub elapsed_ms: i64,
    pub remaining_ms: i64,
    pub burn_rate_usd_per_hour: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SessionGroupKey {
    bucket: u8,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum StatuslineSessionMarker {
    SessionId(String),
    Project(String),
    Fallback,
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
        let row = grouped.entry(key).or_insert_with(|| SessionReportSession {
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
pub fn build_blocks_report(
    events: &[UsageEvent],
    cost_mode: CostMode,
    pricing: &PricingCatalog,
) -> BlocksReport {
    let mut sorted = events.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| {
        left.occurred_at_unix_ms
            .cmp(&right.occurred_at_unix_ms)
            .then_with(|| left.origin.file.cmp(&right.origin.file))
            .then_with(|| left.origin.line_number.cmp(&right.origin.line_number))
    });

    let mut rows = Vec::<BlocksReportBlock>::new();
    let mut current_end_ms: Option<i64> = None;

    for event in sorted {
        let should_start_new = match current_end_ms {
            Some(end_ms) => event.occurred_at_unix_ms >= end_ms,
            None => true,
        };

        if should_start_new {
            let start_ms = event.occurred_at_unix_ms;
            let end_ms = start_ms.saturating_add(DEFAULT_BLOCK_WINDOW_MS);
            rows.push(BlocksReportBlock {
                block_start: utc_timestamp_label_from_unix_ms(start_ms),
                block_end: utc_timestamp_label_from_unix_ms(end_ms),
                first_event_at: utc_timestamp_label_from_unix_ms(start_ms),
                last_event_at: utc_timestamp_label_from_unix_ms(start_ms),
                ..BlocksReportBlock::default()
            });
            current_end_ms = Some(end_ms);
        }

        let row = rows
            .last_mut()
            .expect("rows always contains current block after initialization");
        row.entries = row.entries.saturating_add(1);
        row.last_event_at = utc_timestamp_label_from_unix_ms(event.occurred_at_unix_ms);
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

    let mut report = BlocksReport {
        blocks: rows,
        totals: BlocksReportTotals::default(),
    };
    report.totals.blocks = report.blocks.len();

    for block in &report.blocks {
        report.totals.entries = report.totals.entries.saturating_add(block.entries);
        report.totals.input_tokens = report
            .totals
            .input_tokens
            .saturating_add(block.input_tokens);
        report.totals.output_tokens = report
            .totals
            .output_tokens
            .saturating_add(block.output_tokens);
        report.totals.cache_creation_input_tokens = report
            .totals
            .cache_creation_input_tokens
            .saturating_add(block.cache_creation_input_tokens);
        report.totals.cache_read_input_tokens = report
            .totals
            .cache_read_input_tokens
            .saturating_add(block.cache_read_input_tokens);
        report.totals.total_tokens = report
            .totals
            .total_tokens
            .saturating_add(block.total_tokens);
        report.totals.total_cost_usd += block.total_cost_usd;
        report.totals.entries_with_raw_cost = report
            .totals
            .entries_with_raw_cost
            .saturating_add(block.entries_with_raw_cost);
        report.totals.entries_with_calculated_cost = report
            .totals
            .entries_with_calculated_cost
            .saturating_add(block.entries_with_calculated_cost);
        report.totals.entries_with_missing_cost = report
            .totals
            .entries_with_missing_cost
            .saturating_add(block.entries_with_missing_cost);
    }

    report
}

#[must_use]
pub fn build_statusline_report(
    events: &[UsageEvent],
    cost_mode: CostMode,
    pricing: &PricingCatalog,
) -> StatuslineReport {
    let mut sorted = events.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| {
        left.occurred_at_unix_ms
            .cmp(&right.occurred_at_unix_ms)
            .then_with(|| left.origin.file.cmp(&right.origin.file))
            .then_with(|| left.origin.line_number.cmp(&right.origin.line_number))
    });

    let Some(latest_event) = sorted.last().copied() else {
        return StatuslineReport::default();
    };

    let marker = statusline_marker_from_event(latest_event);
    let latest_day = utc_day_label_from_unix_ms(latest_event.occurred_at_unix_ms);

    let mut report = StatuslineReport::default();
    let mut latest_active_ms: Option<i64> = None;
    let mut block_start_ms: Option<i64> = None;
    let mut block_cost_usd = 0.0;

    for event in &sorted {
        let resolved = resolve_event_cost(event, cost_mode, pricing);
        let event_day = utc_day_label_from_unix_ms(event.occurred_at_unix_ms);
        if event_day == latest_day {
            report.today_cost_usd += resolved.cost_usd;
        }

        if !event_matches_statusline_marker(event, &marker) {
            continue;
        }

        report.session_cost_usd += resolved.cost_usd;
        report.session_input_tokens = report
            .session_input_tokens
            .saturating_add(event.usage.input_tokens);
        latest_active_ms = Some(event.occurred_at_unix_ms);
        if let Some(model) = normalized_optional_string(event.model.as_deref()) {
            report.model = Some(model);
        }

        let should_start_new = match block_start_ms {
            Some(start_ms) => {
                let end_ms = start_ms.saturating_add(DEFAULT_BLOCK_WINDOW_MS);
                event.occurred_at_unix_ms >= end_ms
            }
            None => true,
        };

        if should_start_new {
            block_start_ms = Some(event.occurred_at_unix_ms);
            block_cost_usd = 0.0;
        }
        block_cost_usd += resolved.cost_usd;
    }

    if report.model.is_none() {
        report.model = sorted
            .iter()
            .rev()
            .find_map(|event| normalized_optional_string(event.model.as_deref()));
    }

    if let (Some(start_ms), Some(last_ms)) = (block_start_ms, latest_active_ms) {
        let block_end_ms = start_ms.saturating_add(DEFAULT_BLOCK_WINDOW_MS);
        let elapsed_ms = last_ms.saturating_sub(start_ms);
        let remaining_ms = block_end_ms.saturating_sub(last_ms).max(0);
        let burn_rate_usd_per_hour = if elapsed_ms <= 0 {
            0.0
        } else {
            block_cost_usd * 3_600_000.0 / (elapsed_ms as f64)
        };

        report.active_block = Some(StatuslineReportBlock {
            block_start: utc_timestamp_label_from_unix_ms(start_ms),
            block_end: utc_timestamp_label_from_unix_ms(block_end_ms),
            cost_usd: block_cost_usd,
            elapsed_ms,
            remaining_ms,
            burn_rate_usd_per_hour,
        });
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

#[must_use]
pub fn render_blocks_report_json(
    report: &BlocksReport,
    discovery_warning_count: usize,
    parse_warning_count: usize,
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"mode\": \"blocks\",\n");

    if report.blocks.is_empty() {
        out.push_str("  \"blocks\": [],\n");
    } else {
        out.push_str("  \"blocks\": [\n");
        for (index, block) in report.blocks.iter().enumerate() {
            out.push_str("    {\n");
            out.push_str(&format!(
                "      \"block_start\": \"{}\",\n",
                json_escape(&block.block_start)
            ));
            out.push_str(&format!(
                "      \"block_end\": \"{}\",\n",
                json_escape(&block.block_end)
            ));
            out.push_str(&format!(
                "      \"first_event_at\": \"{}\",\n",
                json_escape(&block.first_event_at)
            ));
            out.push_str(&format!(
                "      \"last_event_at\": \"{}\",\n",
                json_escape(&block.last_event_at)
            ));
            out.push_str(&format!("      \"entries\": {},\n", block.entries));
            out.push_str("      \"tokens\": {\n");
            out.push_str(&format!("        \"input\": {},\n", block.input_tokens));
            out.push_str(&format!("        \"output\": {},\n", block.output_tokens));
            out.push_str(&format!(
                "        \"cache_creation_input\": {},\n",
                block.cache_creation_input_tokens
            ));
            out.push_str(&format!(
                "        \"cache_read_input\": {},\n",
                block.cache_read_input_tokens
            ));
            out.push_str(&format!("        \"total\": {}\n", block.total_tokens));
            out.push_str("      },\n");
            out.push_str("      \"cost\": {\n");
            out.push_str(&format!(
                "        \"usd\": {},\n",
                json_number(block.total_cost_usd)
            ));
            out.push_str(&format!(
                "        \"raw_entries\": {},\n",
                block.entries_with_raw_cost
            ));
            out.push_str(&format!(
                "        \"calculated_entries\": {},\n",
                block.entries_with_calculated_cost
            ));
            out.push_str(&format!(
                "        \"missing_entries\": {}\n",
                block.entries_with_missing_cost
            ));
            out.push_str("      }\n");
            out.push_str("    }");
            if index + 1 != report.blocks.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ],\n");
    }

    out.push_str("  \"totals\": {\n");
    out.push_str(&format!("    \"blocks\": {},\n", report.totals.blocks));
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
pub fn render_blocks_report_table(
    report: &BlocksReport,
    discovery_warning_count: usize,
    parse_warning_count: usize,
) -> String {
    let mut lines = Vec::new();
    lines.push(
        "BLOCK_START           BLOCK_END             ENTRIES INPUT OUTPUT CACHE_CREATE CACHE_READ TOTAL COST_USD"
            .to_owned(),
    );

    for block in &report.blocks {
        lines.push(format!(
            "{} {} {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
            block.block_start,
            block.block_end,
            block.entries,
            block.input_tokens,
            block.output_tokens,
            block.cache_creation_input_tokens,
            block.cache_read_input_tokens,
            block.total_tokens,
            json_number(block.total_cost_usd)
        ));
    }

    lines.push(format!(
        "TOTAL                                    {:>7} {:>5} {:>6} {:>12} {:>10} {:>5} {:>8}",
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
pub fn render_statusline_report_json(
    report: &StatuslineReport,
    discovery_warning_count: usize,
    parse_warning_count: usize,
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"mode\": \"statusline\",\n");
    out.push_str("  \"statusline\": {\n");
    out.push_str(&format!(
        "    \"model\": {},\n",
        json_optional_string(report.model.as_deref())
    ));
    out.push_str(&format!(
        "    \"session_cost_usd\": {},\n",
        json_number(report.session_cost_usd)
    ));
    out.push_str(&format!(
        "    \"today_cost_usd\": {},\n",
        json_number(report.today_cost_usd)
    ));
    out.push_str(&format!(
        "    \"session_input_tokens\": {},\n",
        report.session_input_tokens
    ));
    out.push_str("    \"active_block\": ");
    if let Some(block) = &report.active_block {
        out.push_str("{\n");
        out.push_str(&format!(
            "      \"block_start\": \"{}\",\n",
            json_escape(&block.block_start)
        ));
        out.push_str(&format!(
            "      \"block_end\": \"{}\",\n",
            json_escape(&block.block_end)
        ));
        out.push_str(&format!(
            "      \"cost_usd\": {},\n",
            json_number(block.cost_usd)
        ));
        out.push_str(&format!("      \"elapsed_ms\": {},\n", block.elapsed_ms));
        out.push_str(&format!(
            "      \"remaining_ms\": {},\n",
            block.remaining_ms
        ));
        out.push_str(&format!(
            "      \"remaining\": \"{}\",\n",
            json_escape(&compact_duration_from_ms(block.remaining_ms))
        ));
        out.push_str(&format!(
            "      \"burn_rate_usd_per_hour\": {}\n",
            json_number(block.burn_rate_usd_per_hour)
        ));
        out.push_str("    }\n");
    } else {
        out.push_str("null\n");
    }
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
pub fn render_statusline_report_line(report: &StatuslineReport) -> String {
    let model = report.model.as_deref().unwrap_or("unknown");
    let (block_usd, block_remaining, burn_usd_per_hour) = match &report.active_block {
        Some(block) => (
            format_money_fixed_2(block.cost_usd),
            compact_duration_from_ms(block.remaining_ms),
            format_money_fixed_2(block.burn_rate_usd_per_hour),
        ),
        None => (
            format_money_fixed_2(0.0),
            compact_duration_from_ms(0),
            format_money_fixed_2(0.0),
        ),
    };

    format!(
        "model={model} session_usd={} today_usd={} block_usd={block_usd} block_remaining={block_remaining} burn_usd_per_hour={burn_usd_per_hour} input_tokens={}\n",
        format_money_fixed_2(report.session_cost_usd),
        format_money_fixed_2(report.today_cost_usd),
        report.session_input_tokens
    )
}

fn utc_day_label_from_unix_ms(unix_ms: i64) -> String {
    utc_day_label_from_days_since_epoch(unix_ms_to_days_since_epoch(unix_ms))
}

fn utc_timestamp_label_from_unix_ms(unix_ms: i64) -> String {
    let unix_seconds = unix_ms.div_euclid(1_000);
    let days_since_epoch = unix_seconds.div_euclid(86_400);
    let seconds_of_day = unix_seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days_since_epoch);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
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

fn statusline_marker_from_event(event: &UsageEvent) -> StatuslineSessionMarker {
    if let Some(session_id) = normalized_optional_string(event.session_id.as_deref()) {
        return StatuslineSessionMarker::SessionId(session_id);
    }
    if let Some(project) = normalized_optional_string(event.project.as_deref()) {
        return StatuslineSessionMarker::Project(project);
    }
    StatuslineSessionMarker::Fallback
}

fn event_matches_statusline_marker(event: &UsageEvent, marker: &StatuslineSessionMarker) -> bool {
    match marker {
        StatuslineSessionMarker::SessionId(expected) => {
            normalized_optional_string(event.session_id.as_deref()).as_deref() == Some(expected)
        }
        StatuslineSessionMarker::Project(expected) => {
            normalized_optional_string(event.project.as_deref()).as_deref() == Some(expected)
        }
        StatuslineSessionMarker::Fallback => true,
    }
}

fn normalized_optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn compact_duration_from_ms(duration_ms: i64) -> String {
    let total_minutes = duration_ms.max(0).div_euclid(60_000);
    let hours = total_minutes.div_euclid(60);
    let minutes = total_minutes.rem_euclid(60);
    format!("{hours}h{minutes:02}m")
}

fn format_money_fixed_2(value: f64) -> String {
    let normalized = if value.is_finite() { value } else { 0.0 };
    format!("{normalized:.2}")
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

    #[test]
    fn builds_blocks_with_rolling_five_hour_windows() {
        let events = vec![
            test_event(
                1_773_158_400_000,
                Some("claude-sonnet"),
                Some(0.09),
                60,
                30,
                0,
                10,
            ),
            test_event(
                1_773_145_800_000,
                Some("claude-sonnet"),
                None,
                25,
                10,
                5,
                15,
            ),
            test_event(
                1_773_136_800_000,
                Some("claude-sonnet"),
                Some(0.12),
                100,
                50,
                0,
                0,
            ),
            test_event(
                1_773_195_000_000,
                Some("claude-sonnet"),
                Some(0.05),
                30,
                10,
                0,
                0,
            ),
            test_event(1_773_174_600_000, Some("unknown-model"), None, 15, 5, 0, 0),
            test_event(
                1_773_154_740_000,
                Some("claude-sonnet"),
                Some(0.06),
                40,
                20,
                0,
                0,
            ),
        ];

        let report = build_blocks_report(&events, CostMode::Auto, &PricingCatalog::new());

        assert_eq!(report.blocks.len(), 3);

        assert_eq!(report.blocks[0].block_start, "2026-03-10T10:00:00Z");
        assert_eq!(report.blocks[0].block_end, "2026-03-10T15:00:00Z");
        assert_eq!(report.blocks[0].entries, 3);
        assert_eq!(report.blocks[0].total_tokens, 265);
        assert_eq!(report.blocks[0].entries_with_raw_cost, 2);
        assert_eq!(report.blocks[0].entries_with_missing_cost, 1);
        assert_eq!(report.blocks[0].total_cost_usd, 0.18);

        assert_eq!(report.blocks[1].block_start, "2026-03-10T16:00:00Z");
        assert_eq!(report.blocks[1].block_end, "2026-03-10T21:00:00Z");
        assert_eq!(report.blocks[1].entries, 2);
        assert_eq!(report.blocks[1].total_tokens, 120);
        assert_eq!(report.blocks[1].entries_with_raw_cost, 1);
        assert_eq!(report.blocks[1].entries_with_missing_cost, 1);
        assert_eq!(report.blocks[1].total_cost_usd, 0.09);

        assert_eq!(report.blocks[2].block_start, "2026-03-11T02:10:00Z");
        assert_eq!(report.blocks[2].block_end, "2026-03-11T07:10:00Z");
        assert_eq!(report.blocks[2].entries, 1);
        assert_eq!(report.blocks[2].total_tokens, 40);
        assert_eq!(report.blocks[2].entries_with_raw_cost, 1);
        assert_eq!(report.blocks[2].entries_with_missing_cost, 0);
        assert_eq!(report.blocks[2].total_cost_usd, 0.05);

        assert_eq!(report.totals.blocks, 3);
        assert_eq!(report.totals.entries, 6);
        assert_eq!(report.totals.total_tokens, 425);
        assert_eq!(report.totals.entries_with_raw_cost, 4);
        assert_eq!(report.totals.entries_with_missing_cost, 2);
        assert_eq!(report.totals.total_cost_usd, 0.32);
    }

    #[test]
    fn renders_blocks_json_shape_deterministically() {
        let report = BlocksReport {
            blocks: vec![BlocksReportBlock {
                block_start: "2026-03-10T10:00:00Z".to_owned(),
                block_end: "2026-03-10T15:00:00Z".to_owned(),
                first_event_at: "2026-03-10T10:00:00Z".to_owned(),
                last_event_at: "2026-03-10T12:30:00Z".to_owned(),
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
            totals: BlocksReportTotals {
                blocks: 1,
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

        let rendered = render_blocks_report_json(&report, 1, 2);

        assert!(rendered.contains("\"mode\": \"blocks\""));
        assert!(rendered.contains("\"block_start\": \"2026-03-10T10:00:00Z\""));
        assert!(rendered.contains("\"block_end\": \"2026-03-10T15:00:00Z\""));
        assert!(rendered.contains("\"first_event_at\": \"2026-03-10T10:00:00Z\""));
        assert!(rendered.contains("\"last_event_at\": \"2026-03-10T12:30:00Z\""));
        assert!(rendered.contains("\"usd\": 0.123456"));
        assert!(rendered.contains("\"discovery\": 1"));
        assert!(rendered.contains("\"parse\": 2"));
    }

    #[test]
    fn builds_statusline_for_latest_session_marker() {
        let events = vec![
            test_event_with_identity(
                1_773_136_800_000,
                Some("session-a"),
                Some("team-alpha"),
                Some("claude-sonnet"),
                Some(0.12),
                100,
                50,
                0,
                0,
            ),
            test_event_with_identity(
                1_773_154_740_000,
                Some("session-a"),
                Some("team-alpha"),
                None,
                None,
                25,
                10,
                5,
                15,
            ),
            test_event_with_identity(
                1_773_158_340_000,
                Some("session-b"),
                Some("team-beta"),
                Some("unknown-model"),
                Some(0.06),
                40,
                20,
                0,
                0,
            ),
            test_event_with_identity(
                1_773_195_000_000,
                Some("session-a"),
                Some("team-alpha"),
                None,
                Some(0.05),
                30,
                10,
                0,
                0,
            ),
        ];

        let report = build_statusline_report(&events, CostMode::Auto, &PricingCatalog::new());

        assert_eq!(report.model.as_deref(), Some("claude-sonnet"));
        assert!((report.session_cost_usd - 0.17).abs() < 0.000_000_001);
        assert!((report.today_cost_usd - 0.05).abs() < 0.000_000_001);
        assert_eq!(report.session_input_tokens, 155);

        let block = report
            .active_block
            .as_ref()
            .expect("expected active block for latest session");
        assert_eq!(block.block_start, "2026-03-11T02:10:00Z");
        assert_eq!(block.block_end, "2026-03-11T07:10:00Z");
        assert!((block.cost_usd - 0.05).abs() < 0.000_000_001);
        assert_eq!(block.elapsed_ms, 0);
        assert_eq!(block.remaining_ms, 18_000_000);
        assert!((block.burn_rate_usd_per_hour - 0.0).abs() < 0.000_000_001);
    }

    #[test]
    fn renders_statusline_as_single_compact_line() {
        let report = StatuslineReport {
            model: Some("claude-sonnet".to_owned()),
            session_cost_usd: 0.17,
            today_cost_usd: 0.05,
            session_input_tokens: 155,
            active_block: Some(StatuslineReportBlock {
                block_start: "2026-03-11T02:10:00Z".to_owned(),
                block_end: "2026-03-11T07:10:00Z".to_owned(),
                cost_usd: 0.05,
                elapsed_ms: 0,
                remaining_ms: 18_000_000,
                burn_rate_usd_per_hour: 0.0,
            }),
        };

        let rendered = render_statusline_report_line(&report);
        let trimmed = rendered.trim_end_matches('\n');
        assert!(!trimmed.contains('\n'));
        assert_eq!(
            trimmed,
            "model=claude-sonnet session_usd=0.17 today_usd=0.05 block_usd=0.05 block_remaining=5h00m burn_usd_per_hour=0.00 input_tokens=155"
        );

        let as_json = render_statusline_report_json(&report, 1, 2);
        assert!(as_json.contains("\"mode\": \"statusline\""));
        assert!(as_json.contains("\"session_cost_usd\": 0.17"));
        assert!(as_json.contains("\"today_cost_usd\": 0.05"));
        assert!(as_json.contains("\"remaining\": \"5h00m\""));
        assert!(as_json.contains("\"discovery\": 1"));
        assert!(as_json.contains("\"parse\": 2"));
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
