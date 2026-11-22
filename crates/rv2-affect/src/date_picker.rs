//! Date picker: a field that opens a month in a small card in the top layer. Arrow keys move a day,
//! PageUp / PageDown a month (Shift a year), Home / End the week, Enter picks, Escape closes.
//!
//! Port losses: a day is a civil `Day` (year, month, day), not a `Date` at local midnight. Locale names,
//! the week start and today come from the browser through `document::eval` after mount. Until it answers,
//! the card shows English names, the week from `fallback_week_start`, and 1970-01-01 as today unless
//! `today` is passed. The labels of a new month arrive after one browser round trip, so the names flash
//! in English for that moment. `ref` forwarding is not available.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/date-picker.css");

/// Gap between the field and the card, and the margin kept from the window edge.
const GAP: f64 = 6.0;
const EDGE: f64 = 8.0;
/// Today before the browser has answered, unless `today` is passed.
const FALLBACK_TODAY: Day = Day {
    year: 1970,
    month: 1,
    day: 1,
};

const WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
/// Regions whose week starts on Sunday. Used only where the browser's own week info is missing.
const SUNDAY_FIRST: [&str; 18] = [
    "US", "CA", "MX", "BR", "JP", "KR", "TW", "HK", "IL", "PH", "IN", "ZA", "SA", "PE", "CO", "VE",
    "GT", "PR",
];

/// A calendar day. Field order makes the derived `Ord` chronological.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Day {
    pub year: i32,
    /// 1 = January to 12 = December. Only `Day::NONE` has 0.
    pub month: u8,
    pub day: u8,
}

impl Day {
    /// No day. A controlled `value` of `Day::NONE` shows the placeholder.
    pub const NONE: Day = Day {
        year: 0,
        month: 0,
        day: 0,
    };

    /// The day, or `None` when the month or the day is out of range.
    pub fn new(year: i32, month: u8, day: u8) -> Option<Day> {
        if !(1..=12).contains(&month) || !(1..=days_in_month(year, month)).contains(&day) {
            return None;
        }
        Some(Day { year, month, day })
    }

    pub fn is_unset(self) -> bool {
        self.month == 0
    }

    /// 0 = Sunday to 6 = Saturday.
    pub fn weekday(self) -> u8 {
        (self.to_ordinal() + 4).rem_euclid(7) as u8
    }

    pub fn add_days(self, n: i64) -> Day {
        Day::from_ordinal(self.to_ordinal() + n)
    }

    /// The same day `n` months on, clamped to the end of the month: Jan 31 + 1 month is Feb 28 or 29.
    pub fn add_months(self, n: i32) -> Day {
        let total = self.year as i64 * 12 + (self.month as i64 - 1) + n as i64;
        let year = total.div_euclid(12) as i32;
        let month = (total.rem_euclid(12) + 1) as u8;
        Day {
            year,
            month,
            day: self.day.min(days_in_month(year, month)),
        }
    }

    pub fn start_of_month(self) -> Day {
        Day { day: 1, ..self }
    }

    /// Moved into `[min, max]`; `min` wins when the two cross.
    pub fn clamp_to(self, min: Option<Day>, max: Option<Day>) -> Day {
        match (min, max) {
            (Some(lo), _) if self < lo => lo,
            (_, Some(hi)) if self > hi => hi,
            _ => self,
        }
    }

    /// `YYYY-MM-DD`, the value a hidden input sends.
    pub fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Days since 1970-01-01 in the proleptic Gregorian calendar.
    pub fn to_ordinal(self) -> i64 {
        let y = self.year as i64 - i64::from(self.month <= 2);
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let mp = (self.month as i64 + 9) % 12;
        let doy = (153 * mp + 2) / 5 + self.day as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    pub fn from_ordinal(z: i64) -> Day {
        let z = z + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = yoe + era * 400 + i64::from(month <= 2);
        Day {
            year: year as i32,
            month: month as u8,
            day: day as u8,
        }
    }
}

pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// 28 to 31. A month outside 1 to 12 counts as 31; `Day::new` checks the month first.
pub fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Six weeks from the start of the week that the month's first day falls in, so the card never changes height.
pub fn month_grid(month: Day, week_start: u8) -> [[Day; 7]; 6] {
    let first = month.start_of_month();
    let start = first.add_days(-column(first, week_start));
    let mut grid = [[start; 7]; 6];
    for (i, cell) in grid.iter_mut().flatten().enumerate() {
        *cell = start.add_days(i as i64);
    }
    grid
}

/// Column of `day` in a week that starts on `week_start` (0 = Sunday).
fn column(day: Day, week_start: u8) -> i64 {
    ((day.weekday() + 7 - week_start % 7) % 7) as i64
}

fn same_month(a: Day, b: Day) -> bool {
    a.year == b.year && a.month == b.month
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridKey {
    Left,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
}

/// The day a grid key moves the focus to, clamped to `[min, max]`. With `rtl`, Left and Right swap.
/// Shift turns a page into a year. Home and End go to the first and last day of the week.
pub fn key_target(
    focus: Day,
    key: GridKey,
    shift: bool,
    week_start: u8,
    rtl: bool,
    min: Option<Day>,
    max: Option<Day>,
) -> Day {
    let side = if rtl { -1 } else { 1 };
    let next = match key {
        GridKey::Left => focus.add_days(-side),
        GridKey::Right => focus.add_days(side),
        GridKey::Up => focus.add_days(-7),
        GridKey::Down => focus.add_days(7),
        GridKey::PageUp => focus.add_months(if shift { -12 } else { -1 }),
        GridKey::PageDown => focus.add_months(if shift { 12 } else { 1 }),
        GridKey::Home => focus.add_days(-column(focus, week_start)),
        GridKey::End => focus.add_days(6 - column(focus, week_start)),
    };
    next.clamp_to(min, max)
}

/// 0 for regions that start the week on Sunday, 1 (Monday) for the rest. `region` is upper-case ISO 3166.
pub fn week_start_for_region(region: &str) -> u8 {
    if SUNDAY_FIRST.contains(&region) { 0 } else { 1 }
}

/// The week start from the locale tag's region, before the browser's own week info is read.
/// A tag without a region (`en`, `ja`) gives Monday.
pub fn fallback_week_start(locale: Option<&str>) -> u8 {
    let region = locale.and_then(|tag| {
        tag.split(['-', '_'])
            .skip(1)
            .take_while(|part| part.len() != 1)
            .find(|part| part.len() == 2 && part.bytes().all(|b| b.is_ascii_alphabetic()))
            .map(str::to_ascii_uppercase)
    });
    region.map_or(1, |r| week_start_for_region(&r))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edges {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extent {
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    pub left: f64,
    pub top: f64,
    pub up: bool,
}

/// Under the field, or above it when there is more room there. The card starts at the field's start edge
/// (its end edge in RTL) and stays inside the window.
pub fn place(trigger: Edges, card: Extent, view: Extent, rtl: bool) -> Placed {
    let below = view.height - trigger.bottom - GAP - EDGE;
    let above = trigger.top - GAP - EDGE;
    let up = below < card.height && above > below;
    let start = if rtl {
        trigger.right - card.width
    } else {
        trigger.left
    };
    let left = start
        .max(EDGE)
        .min((view.width - EDGE - card.width).max(EDGE));
    let top = if up {
        (trigger.top - GAP - card.height).max(EDGE)
    } else {
        trigger.bottom + GAP
    };
    Placed { left, top, up }
}

/// English "Tuesday, March 3, 2026", for a valid day. Used until the browser's names arrive.
pub fn english_long(day: Day) -> String {
    format!(
        "{}, {} {}, {}",
        WEEKDAYS[day.weekday() as usize],
        MONTHS[day.month as usize - 1],
        day.day,
        day.year
    )
}

/// English "Tue, Mar 3, 2026", for a valid day.
pub fn english_short(day: Day) -> String {
    let weekday = WEEKDAYS[day.weekday() as usize];
    let month = MONTHS[day.month as usize - 1];
    format!(
        "{}, {} {}, {}",
        &weekday[..3],
        &month[..3],
        day.day,
        day.year
    )
}

/// English "March 2026", for a valid month.
pub fn english_title(month: Day) -> String {
    format!("{} {}", MONTHS[month.month as usize - 1], month.year)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DatePickerSize {
    /// 28px high.
    Sm,
    /// 32px high.
    #[default]
    Md,
    /// 36px high.
    Lg,
}

impl DatePickerSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

/// What the browser said: names by weekday from Sunday, and the locale's own day labels.
#[derive(Clone, Debug, Default, PartialEq)]
struct Labels {
    today: Option<Day>,
    week_start: Option<u8>,
    /// (short head, long name), indexed by weekday from Sunday.
    heads: Vec<(String, String)>,
    title: Option<(Day, String)>,
    /// (day, number, long label) for the cells of the shown month.
    cells: Vec<(Day, String, String)>,
    /// (day, short label, long label) for the chosen day.
    value: Option<(Day, String, String)>,
}

impl Labels {
    fn head(&self, weekday: usize) -> (String, String) {
        match self.heads.get(weekday) {
            Some(head) => head.clone(),
            None => (
                WEEKDAYS[weekday][..2].to_string(),
                WEEKDAYS[weekday].to_string(),
            ),
        }
    }

    fn cell(&self, day: Day) -> Option<(String, String)> {
        self.cells
            .iter()
            .find(|c| c.0 == day)
            .map(|c| (c.1.clone(), c.2.clone()))
    }
}

#[derive(Clone, Default, PartialEq)]
struct Ids {
    trigger: String,
    panel: String,
    label: String,
    title: String,
    message: String,
}

/// Signals and the props the handlers need. Copy, so each handler takes its own.
#[derive(Clone, Copy)]
struct Ctl {
    ids: Signal<Ids>,
    open: Signal<bool>,
    own: Signal<Option<Day>>,
    focus: Signal<Day>,
    month: Signal<Day>,
    dir: Signal<i8>,
    move_focus: Signal<bool>,
    placed: Signal<Option<Placed>>,
    rtl: Signal<bool>,
    controlled: bool,
    value: Option<Day>,
    today: Day,
    min: Option<Day>,
    max: Option<Day>,
    disabled: bool,
    blocked_by: Option<Callback<Day, bool>>,
    onchange: Option<EventHandler<Day>>,
    week_start: u8,
}

impl Ctl {
    fn blocked(self, day: Day) -> bool {
        self.min.is_some_and(|m| day < m)
            || self.max.is_some_and(|m| day > m)
            || self.blocked_by.is_some_and(|f| f.call(day))
    }

    fn start_day(self) -> Day {
        self.value
            .unwrap_or(self.today)
            .clamp_to(self.min, self.max)
    }

    fn show(self) {
        if self.disabled {
            return;
        }
        let day = self.start_day();
        let mut focus = self.focus;
        focus.set(day);
        let mut month = self.month;
        month.set(day.start_of_month());
        let mut moving = self.move_focus;
        moving.set(true);
        let mut open = self.open;
        open.set(true);
    }

    fn hide(self, focus_back: bool) {
        let mut open = self.open;
        open.set(false);
        let mut moving = self.move_focus;
        moving.set(false);
        if focus_back {
            focus_element(&self.ids.read().trigger);
        }
    }

    fn pick(self, day: Day) {
        if self.blocked(day) {
            return;
        }
        if !self.controlled {
            let mut own = self.own;
            own.set(Some(day));
        }
        if let (Some(onchange), true) = (self.onchange, self.value != Some(day)) {
            onchange.call(day);
        }
        self.hide(true);
    }

    fn go_month(self, next: Day) {
        let target = next.start_of_month();
        let current = (self.month)();
        if target != current {
            let mut dir = self.dir;
            dir.set(if target > current { 1 } else { -1 });
            let mut month = self.month;
            month.set(target);
        }
    }

    /// The month arrows: the focus stays on the arrow, and the focused day follows into the new month.
    fn shift(self, n: i32) {
        let target = (self.month)().start_of_month().add_months(n);
        let focus = (self.focus)().add_months(n).clamp_to(self.min, self.max);
        let mut moving = self.move_focus;
        moving.set(false);
        let mut focus_sig = self.focus;
        focus_sig.set(focus);
        let mut dir = self.dir;
        dir.set(if n > 0 { 1 } else { -1 });
        let mut month = self.month;
        month.set(target);
    }

    fn on_grid_key(self, e: KeyboardEvent) {
        let pressed = e.key();
        if matches!(pressed, Key::Enter)
            || matches!(&pressed, Key::Character(c) if c.as_str() == " ")
        {
            e.prevent_default();
            self.pick((self.focus)());
            return;
        }
        let key = match pressed {
            Key::ArrowLeft => GridKey::Left,
            Key::ArrowRight => GridKey::Right,
            Key::ArrowUp => GridKey::Up,
            Key::ArrowDown => GridKey::Down,
            Key::PageUp => GridKey::PageUp,
            Key::PageDown => GridKey::PageDown,
            Key::Home => GridKey::Home,
            Key::End => GridKey::End,
            _ => return,
        };
        e.prevent_default();
        let next = key_target(
            (self.focus)(),
            key,
            e.modifiers().shift(),
            self.week_start,
            (self.rtl)(),
            self.min,
            self.max,
        );
        let mut moving = self.move_focus;
        moving.set(true);
        let mut focus = self.focus;
        focus.set(next);
        self.go_month(next);
    }
}

/// Moves focus to the element with `id`. Fire-and-forget.
fn focus_element(id: &str) {
    document::eval(&format!(
        "document.getElementById({id:?})?.focus({{ preventScroll: true }});"
    ));
}

fn js_string(s: Option<&str>) -> String {
    s.map_or_else(|| "null".to_string(), |s| format!("{s:?}"))
}

/// Reads the browser's week info, weekday names and today. Locale-only, so it runs when the locale changes.
fn names_script(locale: Option<&str>) -> String {
    let loc = js_string(locale);
    format!(
        r#"const loc = {loc} ?? undefined;
const now = new Date();
let first = null, region = null;
try {{
  const L = new Intl.Locale(loc ?? new Intl.DateTimeFormat().resolvedOptions().locale);
  first = (L.getWeekInfo?.() ?? L.weekInfo)?.firstDay ?? null;
  region = L.maximize().region ?? null;
}} catch (_) {{}}
const wShort = new Intl.DateTimeFormat(loc, {{ weekday: "short", calendar: "gregory" }});
const wNarrow = new Intl.DateTimeFormat(loc, {{ weekday: "narrow", calendar: "gregory" }});
const wLong = new Intl.DateTimeFormat(loc, {{ weekday: "long", calendar: "gregory" }});
const heads = [];
for (let i = 0; i < 7; i++) {{
  const date = new Date(2024, 0, 7 + i);
  const short = wShort.format(date).replace(/\.$/, "");
  heads.push([/^\p{{Script=Latin}}/u.test(short) ? short.slice(0, 2) : short.length > 3 ? wNarrow.format(date) : short, wLong.format(date)]);
}}
return {{ today: [now.getFullYear(), now.getMonth() + 1, now.getDate()], first, region, heads }};"#
    )
}

/// The day labels of the six weeks in `dates`, and the chosen day's labels. Runs when the month, the value or the locale changes.
fn labels_script(locale: Option<&str>, value: Option<Day>, month: Day, dates: &[Day]) -> String {
    let loc = js_string(locale);
    let value_js = match value {
        Some(v) => format!("[{},{},{}]", v.year, v.month, v.day),
        None => "null".to_string(),
    };
    let first_js = format!("[{},{},1]", month.year, month.month);
    let dates_js = dates
        .iter()
        .map(|d| format!("[{},{},{}]", d.year, d.month, d.day))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"const loc = {loc} ?? undefined;
const at = (p) => new Date(p[0], p[1] - 1, p[2]);
const fmt = (o) => new Intl.DateTimeFormat(loc, {{ ...o, calendar: "gregory" }});
const longF = fmt({{ weekday: "long", month: "long", day: "numeric", year: "numeric" }});
const shortF = fmt({{ weekday: "short", month: "short", day: "numeric", year: "numeric" }});
const numF = fmt({{ day: "numeric" }});
const dayNumber = (date) => numF.formatToParts(date).find((p) => p.type === "day")?.value ?? String(date.getDate());
const v = {value_js};
return {{
  title: fmt({{ month: "long", year: "numeric" }}).format(at({first_js})),
  cells: [{dates_js}].map((p) => [dayNumber(at(p)), longF.format(at(p))]),
  value: v ? [shortF.format(at(v)), longF.format(at(v))] : null,
}};"#
    )
}

/// The trigger's rect, the viewport, the card's size and the field's direction. Shows the card first, so it can be measured.
fn measure_script(ids: &Ids) -> String {
    format!(
        r#"const el = document.getElementById({panel:?});
const t = document.getElementById({trigger:?});
if (!el || !t) return null;
if ("showPopover" in el && !el.matches(":popover-open")) el.showPopover();
const r = t.getBoundingClientRect();
return [r.left, r.top, r.right, r.bottom, document.documentElement.clientWidth || window.innerWidth, window.innerHeight, el.offsetWidth, el.offsetHeight, getComputedStyle(t).direction === "rtl"];"#,
        panel = ids.panel,
        trigger = ids.trigger,
    )
}

/// Window and outside-press listeners for the open card. Installing again replaces the previous set.
fn listen_script(ids: &Ids) -> String {
    format!(
        r#"const owners = (window.__affectOwners ||= {{}});
owners[{panel:?}]?.remove();
const handlers = [];
const on = (target, type, fn, options) => {{ target.addEventListener(type, fn, options); handlers.push(() => target.removeEventListener(type, fn, options)); }};
const send = (message) => {{ try {{ dioxus.send(message); }} catch (_) {{}} }};
owners[{panel:?}] = {{ remove: () => handlers.forEach((undo) => undo()) }};
const panel = () => document.getElementById({panel:?});
const trigger = () => document.getElementById({trigger:?});
const inside = (node) => !!node && (!!panel()?.contains(node) || !!trigger()?.contains(node));
on(document, "pointerdown", (e) => {{ if (!inside(e.target)) send("outside"); }}, true);
on(document, "focusout", (e) => {{ if (panel()?.contains(e.target) && e.relatedTarget && !inside(e.relatedTarget)) send("blur"); }});
on(window, "resize", () => send("move"));
on(window, "scroll", () => send("move"), true);"#,
        panel = ids.panel,
        trigger = ids.trigger,
    )
}

fn release_script(panel: &str) -> String {
    format!("window.__affectOwners?.[{panel:?}]?.remove();")
}

/// Measures the card and sets its place. A failed measurement keeps the last place.
fn place_now(ctl: Ctl) {
    let script = measure_script(&ctl.ids.read());
    spawn(async move {
        let Ok(v) = document::eval(&script).await else {
            return;
        };
        let Some(nums) = (0..8)
            .map(|i| v.get(i)?.as_f64())
            .collect::<Option<Vec<f64>>>()
        else {
            return;
        };
        let rtl = v.get(8).and_then(|x| x.as_bool()).unwrap_or(false);
        let trigger = Edges {
            left: nums[0],
            top: nums[1],
            right: nums[2],
            bottom: nums[3],
        };
        let view = Extent {
            width: nums[4],
            height: nums[5],
        };
        let card = Extent {
            width: nums[6],
            height: nums[7],
        };
        let mut placed = ctl.placed;
        placed.set(Some(place(trigger, card, view, rtl)));
        let mut rtl_sig = ctl.rtl;
        rtl_sig.set(rtl);
    });
}

/// Starts the listeners for the open card. The script's `send` messages move the card or close it.
fn listen(ctl: Ctl, script: String) {
    spawn(async move {
        let mut events = document::eval(&script);
        while let Ok(message) = events.recv::<String>().await {
            match message.as_str() {
                "move" => place_now(ctl),
                "outside" | "blur" => ctl.hide(false),
                _ => {}
            }
        }
    });
}

/// One day cell of the grid, with its labels and state already worked out.
struct Cell {
    day: Day,
    iso: String,
    number: String,
    long: String,
    chosen: bool,
    off: bool,
    outside: bool,
    today: bool,
    tab: bool,
}

/// A field that opens a calendar. Controlled when `value` is passed (`Day::NONE` shows the placeholder),
/// otherwise `default_value` starts it.
#[component]
pub fn DatePicker(
    #[props(default)] value: Option<Day>,
    #[props(default)] default_value: Option<Day>,
    #[props(default)] onchange: Option<EventHandler<Day>>,
    #[props(default)] min: Option<Day>,
    #[props(default)] max: Option<Day>,
    /// Single days that can't be picked, e.g. weekends. They stay visible.
    #[props(default)]
    is_date_disabled: Option<Callback<Day, bool>>,
    #[props(default, into)] locale: Option<String>,
    /// 0 = Sunday to 6 = Saturday. By default, the locale's first day of the week.
    #[props(default)]
    week_starts_on: Option<u8>,
    #[props(default = "Pick a date".to_string(), into)] placeholder: String,
    #[props(default)] label: Option<Element>,
    #[props(default)] hint: Option<Element>,
    /// Marks the field invalid. It takes the place of the hint.
    #[props(default)]
    error: Option<Element>,
    #[props(default)] optional: bool,
    #[props(default)] size: DatePickerSize,
    #[props(default)] disabled: bool,
    #[props(default, into)] name: Option<String>,
    #[props(default)] required: bool,
    #[props(default)] today: Option<Day>,
    #[props(default)] default_open: bool,
    #[props(default, into)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default, into)] aria_label: Option<String>,
) -> Element {
    let ids = use_signal(|| {
        let auto = format!("ui-dpkc-{}", next_id());
        Ids {
            trigger: id.clone().unwrap_or_else(|| format!("{auto}-trigger")),
            panel: format!("{auto}-panel"),
            label: format!("{auto}-label"),
            title: format!("{auto}-title"),
            message: format!("{auto}-message"),
        }
    });
    let ids_now = ids.read().clone();

    let own = use_signal(|| default_value.filter(|d| !d.is_unset()));
    let controlled = value.is_some();
    let value = match value {
        Some(v) => (!v.is_unset()).then_some(v),
        None => own(),
    };

    let labels = use_signal(Labels::default);
    let labels_now = labels.read().clone();
    let today = today.or(labels_now.today).unwrap_or(FALLBACK_TODAY);
    let week_start = week_starts_on.filter(|w| *w <= 6).unwrap_or_else(|| {
        labels_now
            .week_start
            .unwrap_or_else(|| fallback_week_start(locale.as_deref()))
    });
    let max = max.filter(|hi| min.is_none_or(|lo| *hi >= lo));

    let start = value.unwrap_or(today).clamp_to(min, max);
    let open_sig = use_signal(|| default_open && !disabled);
    let focus_sig = use_signal(move || start);
    let month_sig = use_signal(move || start.start_of_month());
    let dir_sig = use_signal(|| 1i8);
    let move_focus = use_signal(|| false);
    let placed = use_signal(|| None::<Placed>);
    let rtl = use_signal(|| false);

    let ctl = Ctl {
        ids,
        open: open_sig,
        own,
        focus: focus_sig,
        month: month_sig,
        dir: dir_sig,
        move_focus,
        placed,
        rtl,
        controlled,
        value,
        today,
        min,
        max,
        disabled,
        blocked_by: is_date_disabled,
        onchange,
        week_start,
    };

    use_effect(use_reactive((&locale,), move |(locale,)| {
        let script = names_script(locale.as_deref());
        spawn(async move {
            let Ok(v) = document::eval(&script).await else {
                return;
            };
            let part = |i: usize| v["today"].get(i).and_then(|x| x.as_i64());
            let today = (|| Day::new(part(0)? as i32, part(1)? as u8, part(2)? as u8))();
            let week = v["first"]
                .as_u64()
                .map(|n| (n % 7) as u8)
                .or_else(|| v["region"].as_str().map(week_start_for_region));
            let heads = v["heads"]
                .as_array()
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| {
                            Some((
                                item.get(0)?.as_str()?.to_owned(),
                                item.get(1)?.as_str()?.to_owned(),
                            ))
                        })
                        .collect()
                })
                .unwrap_or_default();
            let mut sig = labels;
            let mut write = sig.write();
            write.today = today;
            write.week_start = week;
            write.heads = heads;
        });
    }));

    use_effect(use_reactive(
        (&locale, &value, &month_sig(), &week_start),
        move |(locale, value, month, week_start)| {
            let dates: Vec<Day> = month_grid(month, week_start)
                .iter()
                .flatten()
                .copied()
                .collect();
            let script = labels_script(locale.as_deref(), value, month, &dates);
            spawn(async move {
                let Ok(v) = document::eval(&script).await else {
                    return;
                };
                let title = v["title"]
                    .as_str()
                    .map(|t| (month.start_of_month(), t.to_owned()));
                let cells = v["cells"]
                    .as_array()
                    .map(|items| {
                        items
                            .iter()
                            .zip(dates.iter())
                            .filter_map(|(item, day)| {
                                Some((
                                    *day,
                                    item.get(0)?.as_str()?.to_owned(),
                                    item.get(1)?.as_str()?.to_owned(),
                                ))
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let value_label = value.and_then(|day| {
                    let pair = v["value"].as_array()?;
                    Some((
                        day,
                        pair.first()?.as_str()?.to_owned(),
                        pair.get(1)?.as_str()?.to_owned(),
                    ))
                });
                let mut sig = labels;
                let mut write = sig.write();
                write.title = title;
                write.cells = cells;
                write.value = value_label;
            });
        },
    ));

    use_effect(move || {
        if !open_sig() {
            let mut placed = ctl.placed;
            placed.set(None);
            document::eval(&release_script(&ctl.ids.read().panel));
            return;
        }
        let script = listen_script(&ctl.ids.read());
        listen(ctl, script);
        place_now(ctl);
    });

    // A day gets focus after an opening or a key, not after a press on the month arrows.
    use_effect(move || {
        let is_open = open_sig();
        let target = focus_sig();
        let _shown = month_sig();
        // The panel stays visibility:hidden until its place is measured; focus waits for that.
        let _placed = placed();
        if !is_open || placed.peek().is_none() || !*move_focus.peek() {
            return;
        }
        let mut consumed = move_focus;
        consumed.set(false);
        let ids_at = ids.read();
        let script = format!(
            r#"const panel = document.getElementById({panel:?}); if (panel && "showPopover" in panel && !panel.matches(":popover-open")) panel.showPopover(); panel?.querySelector('[data-day="{day}"]')?.focus({{ preventScroll: true }});"#,
            panel = ids_at.panel,
            day = target.iso(),
        );
        document::eval(&script);
    });

    let is_open = open_sig();
    let month = month_sig();
    let focus = focus_sig();
    let dir = dir_sig();
    let scale = size.as_str();
    let invalid = error.is_some();
    let has_label = label.is_some();
    let message = error.clone().or_else(|| hint.clone());
    let msg_key = if invalid { "error" } else { "hint" };
    let hidden_value = value.map(|v| v.iso()).unwrap_or_default();

    let value_key = value.map_or_else(|| "empty".to_string(), |v| v.iso());
    let (value_short, value_long) = match value {
        Some(v) => match labels_now.value.as_ref().filter(|(d, _, _)| *d == v) {
            Some((_, short, long)) => (short.clone(), long.clone()),
            None => (english_short(v), english_long(v)),
        },
        None => (placeholder.clone(), placeholder.clone()),
    };
    let aria_text = match value {
        Some(_) => value_long,
        None => placeholder.clone(),
    };
    let trigger_aria = aria_label.as_ref().map(|al| format!("{al}, {aria_text}"));
    let labelled_by = (aria_label.is_none() && has_label)
        .then(|| format!("{} {}", ids_now.label, ids_now.trigger));
    let described_by = message.as_ref().map(|_| ids_now.message.clone());

    let title_text = match labels_now
        .title
        .as_ref()
        .filter(|(m, _)| *m == month.start_of_month())
    {
        Some((_, text)) => text.clone(),
        None => english_title(month),
    };
    let can_prev = min.is_none_or(|m| m.start_of_month() < month);
    let can_next = max.is_none_or(|m| m.start_of_month() > month);

    let rows = month_grid(month, week_start);
    let visible: Vec<Day> = rows.iter().flatten().copied().collect();
    let tab_day = if visible.contains(&focus) {
        focus
    } else {
        visible
            .iter()
            .copied()
            .find(|d| same_month(*d, month) && !ctl.blocked(*d))
            .or_else(|| visible.iter().copied().find(|d| same_month(*d, month)))
            .unwrap_or(focus)
    };
    let heads: Vec<(usize, String, String)> = (0..7)
        .map(|col| {
            let weekday = (week_start as usize + col) % 7;
            let (text, long) = labels_now.head(weekday);
            (col, text, long)
        })
        .collect();
    let weeks: Vec<Vec<Cell>> = rows
        .iter()
        .map(|week| {
            week.iter()
                .map(|day| {
                    let (number, long) = labels_now
                        .cell(*day)
                        .unwrap_or_else(|| (day.day.to_string(), english_long(*day)));
                    Cell {
                        day: *day,
                        iso: day.iso(),
                        number,
                        long,
                        chosen: value == Some(*day),
                        off: ctl.blocked(*day),
                        outside: !same_month(*day, month),
                        today: *day == today,
                        tab: *day == tab_day,
                    }
                })
                .collect()
        })
        .collect();

    let style = match placed() {
        Some(p) => format!(
            "top:{}px;left:{}px;visibility:visible;",
            p.top.round(),
            p.left.round()
        ),
        None => "visibility:hidden;".to_string(),
    };
    let up = placed().is_some_and(|p| p.up).then_some("");
    let class = class_list("ui-dpkc", &class);
    let month_key = month.iso();

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: "{class}",
            "data-size": "{scale}",
            "data-open": is_open.then_some("true"),
            "data-invalid": invalid.then_some("true"),
            "data-disabled": disabled.then_some("true"),
            if let Some(label_text) = label {
                div { class: "ui-dpkc__head",
                    label { class: "ui-dpkc__label", id: "{ids_now.label}", r#for: "{ids_now.trigger}", {label_text} }
                    if optional {
                        span { class: "ui-dpkc__optional", "Optional" }
                    }
                }
            }
            button {
                id: "{ids_now.trigger}",
                r#type: "button",
                class: "ui-dpkc__trigger",
                aria_haspopup: "dialog",
                aria_expanded: "{is_open}",
                aria_controls: is_open.then(|| ids_now.panel.clone()),
                aria_label: trigger_aria,
                aria_labelledby: labelled_by,
                aria_describedby: described_by,
                aria_invalid: invalid.then_some("true"),
                aria_required: required.then_some("true"),
                disabled,
                onclick: move |_| {
                    if is_open {
                        ctl.hide(true);
                    } else {
                        ctl.show();
                    }
                },
                onkeydown: move |e: KeyboardEvent| {
                    let key = e.key();
                    if !is_open && matches!(key, Key::ArrowDown | Key::ArrowUp) {
                        e.prevent_default();
                        ctl.show();
                    } else if is_open && key == Key::Escape {
                        e.prevent_default();
                        ctl.hide(true);
                    }
                },
                span { class: "ui-dpkc__icon", aria_hidden: "true", CalendarIcon {} }
                span {
                    class: "ui-dpkc__value",
                    key: "{value_key}",
                    "data-empty": value.is_none().then_some(""),
                    "{value_short}"
                }
                span { class: "ui-dpkc__chevron", aria_hidden: "true", ChevronIcon {} }
            }
            if let Some(name) = name {
                input {
                    r#type: "hidden",
                    name: "{name}",
                    value: "{hidden_value}",
                }
            }

            if is_open {
                div {
                    id: "{ids_now.panel}",
                    class: "ui-dpkc__panel",
                    role: "dialog",
                    aria_modal: "false",
                    aria_labelledby: "{ids_now.title}",
                    "popover": "manual",
                    "data-up": up,
                    style: "{style}",
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Escape {
                            e.prevent_default();
                            e.stop_propagation();
                            ctl.hide(true);
                        }
                    },
                    div { class: "ui-dpkc__nav",
                        button {
                            r#type: "button",
                            class: "ui-dpkc__arrow",
                            aria_label: "Previous month",
                            disabled: !can_prev,
                            onclick: move |_| ctl.shift(-1),
                            ArrowIcon { d: "M15 6C15 6 9 10.42 9 12C9 13.58 15 18 15 18" }
                        }
                        div { class: "ui-dpkc__title", id: "{ids_now.title}", aria_live: "polite",
                            span { key: "{month_key}", "data-dir": "{dir}", "{title_text}" }
                        }
                        button {
                            r#type: "button",
                            class: "ui-dpkc__arrow",
                            aria_label: "Next month",
                            disabled: !can_next,
                            onclick: move |_| ctl.shift(1),
                            ArrowIcon { d: "M9 6C9 6 15 10.42 15 12C15 13.58 9 18 9 18" }
                        }
                    }

                    div {
                        role: "grid",
                        aria_labelledby: "{ids_now.title}",
                        class: "ui-dpkc__grid",
                        onkeydown: move |e: KeyboardEvent| ctl.on_grid_key(e),
                        onpointerdown: move |_| {
                            let mut moving = ctl.move_focus;
                            moving.set(false);
                        },
                        div { role: "row", class: "ui-dpkc__weekdays",
                            for (col, text, long) in heads {
                                span { key: "{col}", role: "columnheader", aria_label: "{long}", class: "ui-dpkc__weekday",
                                    abbr { title: "{long}", "{text}" }
                                }
                            }
                        }
                        div { key: "{month_key}", class: "ui-dpkc__weeks", "data-dir": "{dir}",
                            for (wi, week) in weeks.into_iter().enumerate() {
                                div { key: "{wi}", role: "row", class: "ui-dpkc__week",
                                    for cell in week {
                                        span { key: "{cell.iso}", role: "gridcell", aria_selected: "{cell.chosen}", class: "ui-dpkc__cell",
                                            button {
                                                r#type: "button",
                                                class: "ui-dpkc__day",
                                                "data-day": "{cell.iso}",
                                                "data-outside": cell.outside.then_some("true"),
                                                "data-chosen": cell.chosen.then_some("true"),
                                                tabindex: if cell.tab { "0" } else { "-1" },
                                                aria_label: "{cell.long}",
                                                aria_current: cell.today.then_some("date"),
                                                aria_disabled: cell.off.then_some("true"),
                                                onclick: move |_| ctl.pick(cell.day),
                                                onfocus: move |_| {
                                                    if (ctl.focus)() != cell.day {
                                                        let mut focus = ctl.focus;
                                                        focus.set(cell.day);
                                                    }
                                                },
                                                "{cell.number}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(text) = message {
                p {
                    key: "{msg_key}",
                    class: "ui-dpkc__message",
                    id: "{ids_now.message}",
                    aria_live: invalid.then_some("polite"),
                    {text}
                }
            }
        }
    }
}

#[component]
fn ArrowIcon(#[props(into)] d: String) -> Element {
    rsx! {
        svg {
            "viewBox": "0 0 24 24",
            "width": "16",
            "height": "16",
            "fill": "none",
            "stroke": "currentColor",
            "stroke-width": "1.5",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            "aria-hidden": "true",
            path { "d": "{d}" }
        }
    }
}

#[component]
fn ChevronIcon() -> Element {
    rsx! {
        svg {
            "viewBox": "0 0 24 24",
            "width": "14",
            "height": "14",
            "fill": "none",
            "stroke": "currentColor",
            "stroke-width": "1.8",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            "aria-hidden": "true",
            path { "d": "M18 9C18 9 13.58 15 12 15C10.42 15 6 9 6 9" }
        }
    }
}

#[component]
fn CalendarIcon() -> Element {
    rsx! {
        svg {
            "viewBox": "0 0 24 24",
            "width": "16",
            "height": "16",
            "fill": "none",
            "stroke": "currentColor",
            "stroke-width": "1.5",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            "aria-hidden": "true",
            path { "d": "M16 2V6M8 2V6" }
            path { "d": "M13 4H11C7.23 4 5.34 4 4.17 5.17C3 6.34 3 8.23 3 12V14C3 17.77 3 19.66 4.17 20.83C5.34 22 7.23 22 11 22H13C16.77 22 18.66 22 19.83 20.83C21 19.66 21 17.77 21 14V12C21 8.23 21 6.34 19.83 5.17C18.66 4 16.77 4 13 4Z" }
            path { "d": "M3 10H21" }
            path { "d": "M12 14H12.01M12 18H12.01M7.5 14H7.51M7.5 18H7.51M16.5 14H16.51", "stroke-width": "2" }
        }
    }
}
