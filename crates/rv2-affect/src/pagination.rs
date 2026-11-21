//! Pagination: the range you see, previous and next, page numbers with … between them, and a rows-per-page choice.
//!
//! Port losses: `ref` is not forwarded (give it an `id` and reach the element with `document::eval`). The page numbers
//! are unmounted while the row is too narrow for them, decided from a ResizeObserver on the nav element.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/pagination.css");

/// Reports `[width, overflows]` of the nav element on every size change of it or of its page list.
const MEASURE: &str = r#"
const nav = document.getElementById("__NAV__");
if (nav) {
  const pages = nav.querySelector(".ui-pgn__pages");
  const report = () => dioxus.send([nav.clientWidth, nav.scrollWidth > nav.clientWidth + 1]);
  const observer = new ResizeObserver(report);
  observer.observe(nav);
  if (pages) observer.observe(pages);
  report();
  await new Promise(() => {});
}
"#;

const FOCUS_PAGE: &str = r#"
document.getElementById("__NAV__")?.querySelector('[data-page="__PAGE__"]')?.focus({ preventScroll: true });
"#;

/// One slot in the page-number row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Page(usize),
    /// The … between two runs of pages.
    Gap,
}

/// The readout beside the arrows: `1–20 of 1,284`, `0 of 0`, or `Page 3 of 10` when the total is unknown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RangeText {
    Empty,
    Rows {
        first: String,
        last: String,
        total: String,
    },
    Pages {
        page: String,
        count: String,
    },
}

/// Digits grouped by thousands, `1284` → `1,284`.
pub fn group_digits(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// The number of pages: from the row total when it is known, else the caller's page count. Never below 1.
pub fn page_count(rows: Option<usize>, page_size: usize, hint: usize) -> usize {
    match rows {
        Some(total) => total.div_ceil(page_size.max(1)).max(1),
        None => hint.max(1),
    }
}

/// The page shown, held between 1 and the page count.
pub fn clamp_page(page: usize, count: usize) -> usize {
    page.max(1).min(count)
}

/// The page numbers for `page` out of `count`: every page when they fit in `2 * siblings + 5` slots, otherwise
/// first, last, the current page with its siblings, and … on each side that hides pages. The slot count never changes.
pub fn page_slots(page: usize, count: usize, siblings: usize) -> Vec<Slot> {
    let slots = 2 * siblings + 5;
    let run = |from: usize, to: usize| (from..=to).map(Slot::Page).collect::<Vec<_>>();
    if count <= slots {
        return run(1, count);
    }
    let mut out = Vec::with_capacity(slots);
    if page <= siblings + 3 {
        out.extend(run(1, slots - 2));
        out.push(Slot::Gap);
        out.push(Slot::Page(count));
    } else if page + siblings + 2 >= count {
        out.push(Slot::Page(1));
        out.push(Slot::Gap);
        out.extend(run(count - (slots - 3), count));
    } else {
        out.push(Slot::Page(1));
        out.push(Slot::Gap);
        out.extend(run(page - siblings, page + siblings));
        out.push(Slot::Gap);
        out.push(Slot::Page(count));
    }
    out
}

/// The readout for the page shown. Without a total, only the page and the count are known.
pub fn range_text(page: usize, page_size: usize, rows: Option<usize>, count: usize) -> RangeText {
    match rows {
        None => RangeText::Pages {
            page: group_digits(page),
            count: group_digits(count),
        },
        Some(0) => RangeText::Empty,
        Some(total) => {
            let first = (page - 1) * page_size + 1;
            let last = (page * page_size).min(total);
            RangeText::Rows {
                first: group_digits(first),
                last: group_digits(last),
                total: group_digits(total),
            }
        }
    }
}

/// The rows-per-page choices: the caller's sizes plus the current one, without repeats or zeros, smallest first.
pub fn size_choices(sizes: &[usize], current: usize) -> Vec<usize> {
    let mut out: Vec<usize> = sizes
        .iter()
        .copied()
        .chain(std::iter::once(current))
        .filter(|s| *s > 0)
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// The page that holds the first row seen before a size change.
pub fn page_after_size_change(page: usize, old_size: usize, new_size: usize) -> usize {
    ((page - 1) * old_size) / new_size.max(1) + 1
}

/// Too narrow once the numbers overflow; wide enough again only when the row is wider than it was when it went
/// narrow, so the numbers do not flicker at the threshold. Returns the new `(tight, tight_at)`.
pub fn next_tight(tight: bool, tight_at: f64, width: f64, overflow: bool) -> (bool, f64) {
    if !tight {
        if overflow {
            (true, width)
        } else {
            (false, tight_at)
        }
    } else if width > tight_at + 1.0 {
        (false, tight_at)
    } else {
        (true, tight_at)
    }
}

/// A pager for `total` rows (or `page_count` pages when the total is unknown). Previous and next are `aria-disabled`
/// rather than `disabled`, so they keep focus.
#[component]
pub fn Pagination(
    #[props(default = 1)] page: usize,
    on_page_change: EventHandler<usize>,
    #[props(default)] total: Option<usize>,
    #[props(default = 20)] page_size: usize,
    #[props(default)] page_count: Option<usize>,
    #[props(default)] page_sizes: Vec<usize>,
    #[props(default)] on_page_size_change: Option<EventHandler<usize>>,
    #[props(default = 1)] siblings: usize,
    #[props(default)] compact: bool,
    #[props(default)] disabled: bool,
    #[props(default)] class: String,
    #[props(default = "Pagination".to_string(), into)] aria_label: String,
    #[props(extends = GlobalAttributes, extends = nav)] attributes: Vec<Attribute>,
) -> Element {
    let id = use_hook(|| format!("ui-pgn-{}", next_id()));
    let size = page_size.max(1);
    let count = page_count_for(total, size, page_count);
    let page = clamp_page(page, count);
    let digits = group_digits(count).len();
    let choices = size_choices(&page_sizes, size);

    let mut tight = use_signal(|| false);
    let mut tight_at = use_signal(|| 0.0_f64);
    let mut follow = use_signal(|| false);

    // More pages (wider numbers) or other slots: measure again from the full row.
    use_effect(use_reactive((&(count, siblings, compact),), move |_| {
        tight.set(false)
    }));

    let measure_id = id.clone();
    use_effect(move || {
        let mut channel = document::eval(&MEASURE.replace("__NAV__", &measure_id));
        spawn(async move {
            while let Ok((width, overflow)) = channel.recv::<(f64, bool)>().await {
                let (next, at) = next_tight(tight(), tight_at(), width, overflow);
                tight.set(next);
                tight_at.set(at);
            }
        });
    });

    // A pressed number keeps the focus on its new slot.
    let focus_id = id.clone();
    use_effect(use_reactive((&page,), move |(page,)| {
        if follow() {
            follow.set(false);
            let script = FOCUS_PAGE
                .replace("__NAV__", &focus_id)
                .replace("__PAGE__", &page.to_string());
            document::eval(&script);
        }
    }));

    let mut go = move |to: usize, from_number: bool| {
        if disabled {
            return;
        }
        let next = clamp_page(to, count);
        if next == page {
            return;
        }
        follow.set(from_number);
        on_page_change.call(next);
    };

    let numbers = !compact && !tight();
    let class = class_list("ui-pgn", &class);
    let sizes_shown = !page_sizes.is_empty();

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        nav {
            id: "{id}",
            class,
            aria_label: "{aria_label}",
            "data-disabled": disabled.then_some("true"),
            style: "--ui-pgn-digits: {digits};",
            ..attributes,
            div { class: "ui-pgn__info",
                if sizes_shown {
                    label { class: "ui-pgn__size",
                        span { class: "ui-pgn__size-label", "Rows per page" }
                        span { class: "ui-pgn__select",
                            select {
                                id: "{id}-size",
                                value: "{size}",
                                disabled,
                                onchange: move |event: FormEvent| {
                                    let Ok(next) = event.value().parse::<usize>() else { return };
                                    if let Some(handler) = on_page_size_change {
                                        handler.call(next);
                                    }
                                    let keep = page_after_size_change(page, size, next);
                                    if keep != page {
                                        on_page_change.call(keep);
                                    }
                                },
                                for s in choices.clone() {
                                    option { key: "{s}", value: "{s}", {group_digits(s)} }
                                }
                            }
                            {chevron_down()}
                        }
                    }
                }
                span { class: "ui-pgn__range", aria_live: "polite", aria_atomic: "true",
                    {range_view(range_text(page, size, total, count))}
                }
            }
            ul { class: "ui-pgn__pages",
                li {
                    button {
                        "type": "button",
                        class: "ui-pgn__step",
                        aria_label: "Previous page",
                        aria_disabled: (disabled || page <= 1).then_some("true"),
                        onclick: move |_| go(page.saturating_sub(1), false),
                        {chevron(true)}
                    }
                }
                if numbers {
                    for (i, slot) in page_slots(page, count, siblings).into_iter().enumerate() {
                        {slot_view(i, slot, page, disabled, go)}
                    }
                }
                li {
                    button {
                        "type": "button",
                        class: "ui-pgn__step",
                        aria_label: "Next page",
                        aria_disabled: (disabled || page >= count).then_some("true"),
                        onclick: move |_| go(page + 1, false),
                        {chevron(false)}
                    }
                }
            }
        }
    }
}

fn page_count_for(total: Option<usize>, size: usize, hint: Option<usize>) -> usize {
    page_count(total, size, hint.unwrap_or(1))
}

fn slot_view(
    i: usize,
    slot: Slot,
    page: usize,
    disabled: bool,
    go: impl FnMut(usize, bool) + Copy + 'static,
) -> Element {
    match slot {
        Slot::Gap => rsx! {
            li { key: "gap-{i}", class: "ui-pgn__gap", "aria-hidden": "true", "…" }
        },
        Slot::Page(p) => {
            let shown = group_digits(p);
            rsx! {
                li { key: "page-{p}",
                    button {
                        "type": "button",
                        class: "ui-pgn__num",
                        "data-page": "{p}",
                        aria_label: "Page {shown}",
                        aria_current: (p == page).then_some("page"),
                        aria_disabled: disabled.then_some("true"),
                        onclick: move |_| {
                            let mut step = go;
                            step(p, true);
                        },
                        "{shown}"
                    }
                }
            }
        }
    }
}

fn range_view(text: RangeText) -> Element {
    match text {
        RangeText::Empty => rsx! { "0 of 0" },
        RangeText::Rows { first, last, total } => rsx! {
            "{first}–{last} "
            span { class: "ui-pgn__of", "of" }
            " {total}"
        },
        RangeText::Pages { page, count } => rsx! {
            "Page {page} "
            span { class: "ui-pgn__of", "of" }
            " {count}"
        },
    }
}

fn chevron(left: bool) -> Element {
    let d = if left {
        "M15 6s-6 4.42-6 6 6 6 6 6"
    } else {
        "M9 6s6 4.42 6 6-6 6-6 6"
    };
    rsx! {
        svg { view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "focusable": "false", "aria-hidden": "true",
            path { d }
        }
    }
}

fn chevron_down() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "focusable": "false", "aria-hidden": "true",
            path { d: "M18 9s-4.42 6-6 6-6-6-6-6" }
        }
    }
}
