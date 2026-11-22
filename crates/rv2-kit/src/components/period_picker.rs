//! Period picker: a trigger that opens a panel of relative presets and two month grids for a
//! start and an end day, with an optional comparison span.
//!
//! Design choice: "today" comes from `anchor_today` when given, otherwise from the browser after
//! mount (`Date`), otherwise 1970-01-01. Calendar maths, the week start and key movement reuse
//! `rv2_affect::date_picker`. The comparison span is computed here, not by the host.
//!
//! Browser checks: the candidate band follows the pointer after the first click; focus returns to
//! the trigger on close; below the phone breakpoint presets scroll and the two grids stack, so both month controls stay reachable.

use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonVariant};
use rv2_affect::date_picker::{Day, GridKey, english_long, english_title, key_target, month_grid};

use crate::internal::{KIT_CSS, class_list, next_id, sleep};

/// Focuses the element after the next render; the grid may have just moved to another month.
fn focus_later(id: String) {
    spawn(async move {
        sleep(16).await;
        document::eval(&format!("document.getElementById({id:?})?.focus();"));
    });
}

const CSS: Asset = asset!("/assets/period-picker.css");
const FALLBACK_TODAY: Day = Day {
    year: 1970,
    month: 1,
    day: 1,
};
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const WEEKDAYS: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
/// The preset that stands for a hand-picked span; it marks itself when no other preset matches.
const CUSTOM_PRESET: &str = "custom";

/// An inclusive span of days. `from <= to` once built with [`DateRange::ordered`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateRange {
    pub from: Day,
    pub to: Day,
}

impl DateRange {
    /// The two days in order, whichever was given first.
    pub fn ordered(a: Day, b: Day) -> Self {
        if a <= b {
            Self { from: a, to: b }
        } else {
            Self { from: b, to: a }
        }
    }

    /// Number of days, both ends included.
    pub fn days(self) -> i64 {
        self.to.to_ordinal() - self.from.to_ordinal() + 1
    }

    pub fn contains(self, day: Day) -> bool {
        self.from <= day && day <= self.to
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ComparisonMode {
    None,
    #[default]
    PreviousPeriod,
    PreviousYear,
}

impl ComparisonMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::PreviousPeriod => "previous_period",
            Self::PreviousYear => "previous_year",
        }
    }
}

/// `yyyy-mm-dd` or `None`.
pub fn format_iso(day: Day) -> String {
    day.iso()
}

/// `Mar 3, 2026`.
pub fn format_day(day: Day) -> String {
    format!(
        "{} {}, {}",
        MONTHS[day.month as usize - 1],
        day.day,
        day.year
    )
}

/// `Mar 3, 2026 – Mar 9, 2026`.
pub fn format_range(range: DateRange) -> String {
    format!("{} – {}", format_day(range.from), format_day(range.to))
}

/// `1 day`, `12 days`.
pub fn days_line(count: i64) -> String {
    if count == 1 {
        "1 day".to_string()
    } else {
        format!("{count} days")
    }
}

/// The comparison span: an equal-length span ending the day before `from`, or the same dates a year earlier.
pub fn comparison_range(range: DateRange, mode: ComparisonMode) -> Option<DateRange> {
    match mode {
        ComparisonMode::None => None,
        ComparisonMode::PreviousPeriod => {
            let to = range.from.add_days(-1);
            let from = to.add_days(-(range.days() - 1));
            Some(DateRange { from, to })
        }
        ComparisonMode::PreviousYear => Some(DateRange {
            from: range.from.add_months(-12),
            to: range.to.add_months(-12),
        }),
    }
}

/// The span a built-in preset id stands for, resolved against `today`.
pub fn resolve_builtin(preset_id: &str, today: Day) -> DateRange {
    let month_start = today.start_of_month();
    let month_end = month_start.add_months(1).add_days(-1);
    match preset_id {
        "yesterday" => DateRange {
            from: today.add_days(-1),
            to: today.add_days(-1),
        },
        "last_7" => DateRange {
            from: today.add_days(-6),
            to: today,
        },
        "last_30" => DateRange {
            from: today.add_days(-29),
            to: today,
        },
        "last_90" => DateRange {
            from: today.add_days(-89),
            to: today,
        },
        "this_month" => DateRange {
            from: month_start,
            to: month_end,
        },
        "last_month" => {
            let from = month_start.add_months(-1);
            DateRange {
                from,
                to: from.add_months(1).add_days(-1),
            }
        }
        "this_quarter" => {
            let quarter_start = Day {
                month: ((today.month - 1) / 3) * 3 + 1,
                day: 1,
                ..today
            };
            DateRange {
                from: quarter_start,
                to: quarter_start.add_months(3).add_days(-1),
            }
        }
        "year_to_date" => DateRange {
            from: Day {
                month: 1,
                day: 1,
                ..today
            },
            to: today,
        },
        _ => DateRange {
            from: today,
            to: today,
        },
    }
}

/// A selectable preset. `resolve` is `None` for the built-in ids, which resolve through [`resolve_builtin`].
#[derive(Clone, PartialEq)]
pub struct Preset {
    pub preset_id: String,
    pub preset_label: String,
    pub resolve: Option<Callback<Day, DateRange>>,
}

impl Preset {
    pub fn builtin(preset_id: &str, preset_label: &str) -> Self {
        Self {
            preset_id: preset_id.to_string(),
            preset_label: preset_label.to_string(),
            resolve: None,
        }
    }

    fn range_for(&self, today: Day) -> DateRange {
        match self.resolve {
            Some(resolve) => resolve.call(today),
            None => resolve_builtin(&self.preset_id, today),
        }
    }
}

pub fn default_presets() -> Vec<Preset> {
    [
        ("today", "Today"),
        ("yesterday", "Yesterday"),
        ("last_7", "Last 7 days"),
        ("last_30", "Last 30 days"),
        ("last_90", "Last 90 days"),
        ("this_month", "This month"),
        ("last_month", "Last month"),
        ("this_quarter", "This quarter"),
        ("year_to_date", "Year to date"),
        ("custom", "Custom"),
    ]
    .iter()
    .map(|(id, label)| Preset::builtin(id, label))
    .collect()
}

/// Whether `day` may be chosen.
pub fn in_bounds(day: Day, earliest: Option<Day>, latest: Option<Day>) -> bool {
    earliest.is_none_or(|lo| day >= lo) && latest.is_none_or(|hi| day <= hi)
}

/// Click handling for the calendar. The first click sets the start; the second sets the end and yields the range.
pub fn click_day(pending: Option<Day>, day: Day) -> (Option<Day>, Option<DateRange>) {
    match pending {
        None => (Some(day), None),
        Some(start) => (None, Some(DateRange::ordered(start, day))),
    }
}

/// The candidate span shown while a start is pending and the pointer hovers `hover`.
pub fn candidate(pending: Option<Day>, hover: Option<Day>) -> Option<DateRange> {
    Some(DateRange::ordered(pending?, hover?))
}

#[component]
pub fn PeriodPicker(
    #[props(default)] selected_range: Option<DateRange>,
    #[props(default)] starting_range: Option<DateRange>,
    #[props(default)] on_apply: EventHandler<(DateRange, Option<DateRange>, ComparisonMode)>,
    #[props(default)] preset_list: Option<Vec<Preset>>,
    #[props(default = true)] show_comparison: bool,
    #[props(default)] starting_comparison: ComparisonMode,
    #[props(default = 1)] first_weekday: u8,
    #[props(default)] earliest_day: Option<Day>,
    #[props(default)] latest_day: Option<Day>,
    #[props(default)] start_expanded: bool,
    #[props(default)] anchor_today: Option<Day>,
    #[props(default, into)] extra_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-per-{}", next_id()));
    let mut browser_today = use_signal(|| None::<Day>);
    use_effect(move || {
        spawn(async move {
            if let Ok(value) = document::eval(
                "const d = new Date(); return [d.getFullYear(), d.getMonth() + 1, d.getDate()];",
            )
            .await
            {
                let parts = (value[0].as_i64(), value[1].as_i64(), value[2].as_i64());
                if let (Some(y), Some(m), Some(d)) = parts {
                    browser_today.set(Day::new(y as i32, m as u8, d as u8));
                }
            }
        });
    });

    let today = anchor_today.or(browser_today()).unwrap_or(FALLBACK_TODAY);
    let latest_bound = latest_day.or(Some(today));
    let key_day_prefix = format!("{id}-day-");
    let presets = preset_list.clone().unwrap_or_else(default_presets);
    let initial = selected_range
        .or(starting_range)
        .unwrap_or_else(|| resolve_builtin("last_30", today));

    let mut open = use_signal(|| start_expanded);
    let mut draft = use_signal(|| initial);
    let mut committed = use_signal(|| initial);
    let mut mode = use_signal(|| starting_comparison);
    let mut pending = use_signal(|| None::<Day>);
    let mut hover = use_signal(|| None::<Day>);
    let mut view = use_signal(|| initial.from.start_of_month());
    let mut focus = use_signal(|| initial.from);

    let week_start = first_weekday % 7;
    let range_now = if open() {
        draft()
    } else {
        selected_range.unwrap_or(committed())
    };
    let comparison = comparison_range(range_now, mode());
    let trigger_text = match comparison {
        Some(_) => format!(
            "{} · {}",
            format_range(range_now),
            mode().as_str().replace('_', " ")
        ),
        None => format_range(range_now),
    };
    let band = candidate(pending(), hover());
    let left = view().start_of_month();
    let right = left.add_months(1);
    let shown = draft();

    let mut cell_handler = move |day: Day| {
        if !in_bounds(day, earliest_day, latest_bound) {
            return;
        }
        let (next_pending, picked) = click_day(pending(), day);
        pending.set(next_pending);
        focus.set(day);
        if let Some(range) = picked {
            draft.set(range);
        }
    };
    let is_open = open();
    let selected_mode = mode();
    let custom_selected = !presets
        .iter()
        .filter(|preset| preset.preset_id != CUSTOM_PRESET)
        .any(|preset| preset.range_for(today) == shown);
    let summary = match pending() {
        Some(start) => format!("Starts {}, pick an end day", format_day(start)),
        None => days_line(shown.days()),
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class: class_list("kit-per", &extra_class), "data-open": open().then_some("true"),
            button {
                id: "{id}-trigger",
                r#type: "button",
                class: "kit-per__trigger",
                "aria-haspopup": "dialog",
                "aria-expanded": "{is_open}",
                "aria-controls": "{id}-panel",
                onclick: move |_| {
                    if !open() {
                        draft.set(selected_range.unwrap_or(committed()));
                        view.set(selected_range.unwrap_or(committed()).from.start_of_month());
                        pending.set(None);
                    }
                    open.set(!open());
                },
                span { class: "kit-per__value", "{trigger_text}" }
            }
            if open() {
                div {
                    id: "{id}-panel",
                    class: "kit-per__panel",
                    role: "dialog",
                    "aria-label": "Choose a period",
                    onkeydown: move |event: KeyboardEvent| {
                        let key = match event.key() {
                            Key::ArrowLeft => Some(GridKey::Left),
                            Key::ArrowRight => Some(GridKey::Right),
                            Key::ArrowUp => Some(GridKey::Up),
                            Key::ArrowDown => Some(GridKey::Down),
                            Key::PageUp => Some(GridKey::PageUp),
                            Key::PageDown => Some(GridKey::PageDown),
                            Key::Home => Some(GridKey::Home),
                            Key::End => Some(GridKey::End),
                            _ => None,
                        };
                        if let Some(key) = key {
                            event.prevent_default();
                            let next = key_target(focus(), key, event.modifiers().shift(), week_start, false, earliest_day, latest_bound);
                            focus.set(next);
                            if next.month != view().month || next.year != view().year {
                                view.set(next.start_of_month());
                            }
                            focus_later(format!("{key_day_prefix}{}", next.iso()));
                        } else if event.key() == Key::Enter {
                            event.prevent_default();
                            cell_handler(focus());
                        } else if event.key() == Key::Escape {
                            open.set(false);
                            pending.set(None);
                        }
                    },
                    div { class: "kit-per__body",
                        div { class: "kit-per__presets", role: "group", "aria-label": "Presets",
                            for preset in presets.iter() {
                                {
                                    let preset_id = preset.preset_id.clone();
                                    let preset_label = preset.preset_label.clone();
                                    let is_custom = preset_id == CUSTOM_PRESET;
                                    let resolved = preset.range_for(today);
                                    let is_current = if is_custom { custom_selected } else { shown == resolved };
                                    rsx! {
                                        button {
                                            key: "{preset_id}",
                                            r#type: "button",
                                            class: "kit-per__preset",
                                            "aria-pressed": "{is_current}",
                                            onclick: move |_| {
                                                if !is_custom {
                                                    draft.set(resolved);
                                                    view.set(resolved.from.start_of_month());
                                                }
                                                pending.set(None);
                                            },
                                            "{preset_label}"
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "kit-per__calendars",
                            for (index, month) in [left, right].into_iter().enumerate() {
                                {
                                    let title = english_title(month);
                                    let month_name = format!("{} {}", MONTHS[month.month as usize - 1], month.year);
                                    rsx! {
                                div { key: "{index}", class: "kit-per__month", role: "grid", "aria-label": "{title}",
                                    div { class: "kit-per__month-head",
                                        if index == 0 {
                                            button {
                                                r#type: "button",
                                                class: "kit-per__nav",
                                                "aria-label": "Previous month",
                                                onclick: move |_| view.set(view().add_months(-1)),
                                                "‹"
                                            }
                                        } else {
                                            span { class: "kit-per__nav-slot", "aria-hidden": "true" }
                                        }
                                        span { class: "kit-per__month-title", "{month_name}" }
                                        if index == 1 {
                                            button {
                                                r#type: "button",
                                                class: "kit-per__nav",
                                                "aria-label": "Next month",
                                                onclick: move |_| view.set(view().add_months(1)),
                                                "›"
                                            }
                                        } else {
                                            span { class: "kit-per__nav-slot", "aria-hidden": "true" }
                                        }
                                    }
                                    div { class: "kit-per__weekdays", role: "row",
                                        for col in 0..7usize {
                                            span { key: "{col}", role: "columnheader", "aria-hidden": "true",
                                                {WEEKDAYS[(col + week_start as usize) % 7]}
                                            }
                                        }
                                    }
                                    for week in month_grid(month, week_start).iter() {
                                        div { class: "kit-per__week", role: "row",
                                            for day in week.iter().copied() {
                                                {
                                                    let outside = day.month != month.month;
                                                    let blocked = !in_bounds(day, earliest_day, latest_bound);
                                                    let is_start = day == shown.from;
                                                    let is_end = day == shown.to;
                                                    let inside = shown.contains(day);
                                                    let in_candidate = band.is_some_and(|b| b.contains(day));
                                                    let in_comparison = comparison.is_some_and(|c| c.contains(day));
                                                    let is_today = day == today;
                                                    let focused = day == focus();
                                                    let label = english_long(day);
                                                    let iso = day.iso();
                                                    let selected = is_start || is_end;
                                                    rsx! {
                                                        button {
                                                            key: "{iso}",
                                                            id: "{id}-day-{iso}",
                                                            r#type: "button",
                                                            role: "gridcell",
                                                            class: "kit-per__day",
                                                            tabindex: if focused { "0" } else { "-1" },
                                                            "aria-label": "{label}",
                                                            "aria-selected": "{selected}",
                                                            "aria-current": is_today.then_some("date"),
                                                            disabled: blocked,
                                                            "data-outside": outside.then_some("true"),
                                                            "data-inside": inside.then_some("true"),
                                                            "data-start": is_start.then_some("true"),
                                                            "data-end": is_end.then_some("true"),
                                                            "data-candidate": in_candidate.then_some("true"),
                                                            "data-comparison": in_comparison.then_some("true"),
                                                            "data-today": is_today.then_some("true"),
                                                            onmouseenter: move |_| hover.set(Some(day)),
                                                            onclick: move |_| cell_handler(day),
                                                            "{day.day}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "kit-per__foot",
                        if show_comparison {
                            div { class: "kit-per__comparison", role: "radiogroup", "aria-label": "Compare with",
                                for option in [ComparisonMode::None, ComparisonMode::PreviousPeriod, ComparisonMode::PreviousYear] {
                                    button {
                                        key: "{option:?}",
                                        r#type: "button",
                                        role: "radio",
                                        class: "kit-per__radio",
                                        "aria-checked": if option == selected_mode { "true" } else { "false" },
                                        onclick: move |_| mode.set(option),
                                        match option {
                                            ComparisonMode::None => "None",
                                            ComparisonMode::PreviousPeriod => "Previous period",
                                            ComparisonMode::PreviousYear => "Previous year",
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "kit-per__actions",
                            span { class: "kit-per__summary", "{summary}" }
                            Button {
                            variant: ButtonVariant::Secondary,
                            label: "Cancel",
                            onclick: move |_| {
                                open.set(false);
                                pending.set(None);
                            },
                        }
                        Button {
                            label: "Apply",
                            onclick: move |_| {
                                let range = draft();
                                committed.set(range);
                                open.set(false);
                                pending.set(None);
                                on_apply.call((range, comparison_range(range, mode()), mode()));
                            },
                        }
                        }
                    }
                }
            }
        }
    }
}
