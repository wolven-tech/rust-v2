//! Kanban task board: work items in status columns. Cards move by drag, by keyboard, or from
//! the detail panel's status and priority radios; tasks are added inline, searched, filtered by
//! label, and starred.
//!
//! Design choice: the board keeps its own copy of the items and reports the full list through
//! `on_items_update` after every move, edit, add or delete, so a move shows at once.
//! Design choice: a mouse drag uses the browser's drag and drop. Escape cancels it natively and
//! no update is sent.
//! Design choice: a touch drag starts after `HOLD_MS` on a card without movement. The card then
//! follows the finger and the page stops scrolling until the finger lifts.
//! Design choice: the drop line sits above the card under the pointer, or at the end of a column,
//! and a dropped card lands in that place.
//! Design choice: due dates are words (`due today`, `2 days late`), never a colour alone.
//! Browser checks: mouse drag into another column, drop line, Escape during drag, touch hold-drag,
//! quick add, search counts, keyboard move with Left/Right and Up/Down, status and priority radios
//! with arrows, star toggle, due-date words, phone board scroll.

use dioxus::prelude::*;
use rv2_affect::badge::{Badge, BadgeTone};

use crate::internal::{
    KIT_CSS, class_list, focus_after_frame, next_id, prefers_motion_script, sleep,
};

const CSS: Asset = asset!("/assets/kanban-task-board.css");

/// Cards shown per column before the "show more" control.
pub const COLUMN_LIMIT: usize = 50;
/// How long a touch rests on a card before the card lifts for dragging.
pub const HOLD_MS: u32 = 350;
const HOLD_SLOP_PX: f64 = 10.0;

/// The board's touchmove guard is attached at mount, before any touch: a listener added during
/// a lift arrives after Chrome has committed to scrolling, and that scroll cannot be prevented.
fn touch_guard_js(board: &str) -> String {
    format!(
        "const root = document.getElementById({board:?}); \
         if (root && !root.dataset.kbGuard) {{ \
           root.dataset.kbGuard = '1'; \
           root.addEventListener('touchmove', (e) => {{ if (window.__kitKbActive === {board:?}) e.preventDefault(); }}, {{ passive: false }}); \
         }}"
    )
}
const TOUCH_RELEASE_JS: &str = "window.__kitKbActive = null;";

#[derive(Clone, Debug, PartialEq)]
pub struct StatusColumn {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkItem {
    pub id: String,
    /// Shown above the title, such as `ST-214 · Size M · Created 3 Oct`.
    pub identifier_line: String,
    pub title: String,
    pub status_id: String,
    pub priority: u8,
    pub label_ids: Vec<String>,
    pub assignee_id: Option<String>,
    /// Days from today: negative when late, zero when due today, `None` when undated.
    pub due_in_days: Option<i64>,
    pub subtasks_done: u32,
    pub subtasks_total: u32,
    pub pull_request_ref: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TagOption {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemberOption {
    pub id: String,
    pub name: String,
    pub avatar: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
    Comfortable,
    Compact,
}

/// Where a dragged card would land: the column, and the card it goes in front of (`None` is
/// the end of the column).
type DropAt = (String, Option<String>);

/// Words for a due date: `due today`, `1 day late`, `3 days late`, `due in 2 days`.
pub fn due_words(days: i64) -> String {
    match days {
        0 => "due today".to_string(),
        -1 => "1 day late".to_string(),
        d if d < 0 => format!("{} days late", -d),
        1 => "due in 1 day".to_string(),
        d => format!("due in {d} days"),
    }
}

/// Moves a card to the end of `status_id`. Returns `None` when the id is unknown.
pub fn move_to_status(items: &[WorkItem], id: &str, status_id: &str) -> Option<Vec<WorkItem>> {
    let at = items.iter().position(|i| i.id == id)?;
    let mut next: Vec<WorkItem> = items.to_vec();
    let mut card = next.remove(at);
    card.status_id = status_id.to_string();
    next.push(card);
    Some(next)
}

/// Moves a card into `status_id` in front of the card `before`, or to the end of that column
/// when `before` is `None`. Dropping a card in front of itself changes nothing.
pub fn move_to_position(
    items: &[WorkItem],
    id: &str,
    status_id: &str,
    before: Option<&str>,
) -> Option<Vec<WorkItem>> {
    if before == Some(id) {
        return Some(items.to_vec());
    }
    let at = items.iter().position(|i| i.id == id)?;
    let mut next: Vec<WorkItem> = items.to_vec();
    let mut card = next.remove(at);
    card.status_id = status_id.to_string();
    let index = match before {
        Some(target) => next.iter().position(|i| i.id == target)?,
        None => next
            .iter()
            .rposition(|i| i.status_id == status_id)
            .map_or(next.len(), |p| p + 1),
    };
    next.insert(index, card);
    Some(next)
}

/// Moves a card one column left or right, to the end of that column.
pub fn move_column(
    items: &[WorkItem],
    columns: &[StatusColumn],
    id: &str,
    step: i64,
) -> Option<Vec<WorkItem>> {
    let card = items.iter().find(|i| i.id == id)?;
    let at = columns.iter().position(|c| c.id == card.status_id)? as i64;
    let target = at + step;
    if target < 0 || target >= columns.len() as i64 {
        return None;
    }
    move_to_status(items, id, &columns[target as usize].id)
}

/// Swaps a card with its neighbour in the same column; `step` is -1 (up) or 1 (down).
pub fn reorder_in_column(items: &[WorkItem], id: &str, step: i64) -> Option<Vec<WorkItem>> {
    let at = items.iter().position(|i| i.id == id)?;
    let status = items[at].status_id.clone();
    let column: Vec<usize> = items
        .iter()
        .enumerate()
        .filter(|(_, i)| i.status_id == status)
        .map(|(n, _)| n)
        .collect();
    let pos = column.iter().position(|n| *n == at)? as i64;
    let other = pos + step;
    if other < 0 || other >= column.len() as i64 {
        return None;
    }
    let mut next = items.to_vec();
    next.swap(at, column[other as usize]);
    Some(next)
}

/// Index after one step through a radio group of `len` options, clamped at both ends.
pub fn radio_step(current: usize, len: usize, forward: bool) -> usize {
    let last = len.saturating_sub(1);
    if forward {
        (current + 1).min(last)
    } else {
        current.saturating_sub(1)
    }
}

/// Items matching the search text (title or id) and carrying every chosen label.
pub fn filter_items<'a>(
    items: &'a [WorkItem],
    search: &str,
    labels: &[String],
) -> Vec<&'a WorkItem> {
    let needle = search.trim().to_lowercase();
    items
        .iter()
        .filter(|i| {
            needle.is_empty()
                || i.title.to_lowercase().contains(&needle)
                || i.id.to_lowercase().contains(&needle)
        })
        .filter(|i| labels.iter().all(|l| i.label_ids.contains(l)))
        .collect()
}

fn priority_word(p: u8) -> &'static str {
    match p {
        0 => "No priority",
        1 => "Low",
        2 => "Medium",
        3 => "High",
        _ => "Urgent",
    }
}

fn column_label(columns: &[StatusColumn], id: &str) -> String {
    columns
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.label.clone())
        .unwrap_or_default()
}

fn toggle_id(mut ids: Signal<Vec<String>>, id: &str) {
    let mut next = ids.peek().clone();
    match next.iter().position(|s| s == id) {
        Some(at) => {
            next.remove(at);
        }
        None => next.push(id.to_string()),
    }
    ids.set(next);
}

/// The board element and a counter bumped by each card move. The counter's effect plays the
/// move once the new layout is in the DOM.
#[derive(Clone, Default, PartialEq)]
struct Flip {
    board: String,
    seq: u32,
}

/// Records each card's rect and cancels running card animations, so the rects are layout only.
fn snapshot_js(board: &str) -> String {
    format!(
        "const root = document.getElementById({board:?}); \
         if (root) {{ \
           const cards = [...root.querySelectorAll('[data-kb-card]')]; \
           cards.forEach((el) => el.getAnimations().forEach((a) => a.cancel())); \
           const from = {{}}; \
           cards.forEach((el) => {{ const r = el.getBoundingClientRect(); from[el.dataset.kbCard] = [r.left, r.top]; }}); \
           (window.__kitFlip ||= {{}})[{board:?}] = from; \
         }}"
    )
}

/// Glides every card that moved from its snapshot rect to its new one (FLIP). Cards that are
/// re-created in another column also lose their first-view rise, which is cancelled first.
fn play_js(board: &str) -> String {
    format!(
        "const root = document.getElementById({board:?}); \
         const from = window.__kitFlip?.[{board:?}]; \
         if (!root || !from) return; \
         delete window.__kitFlip[{board:?}]; \
         const cards = [...root.querySelectorAll('[data-kb-card]')]; \
         cards.forEach((el) => el.getAnimations().forEach((a) => a.cancel())); \
         if (!({motion})) return; \
         cards.forEach((el) => {{ \
           const before = from[el.dataset.kbCard]; \
           if (!before) return; \
           const r = el.getBoundingClientRect(); \
           const dx = before[0] - r.left; \
           const dy = before[1] - r.top; \
           if (Math.abs(dx) < 0.5 && Math.abs(dy) < 0.5) return; \
           el.animate( \
             [{{ transform: `translate(${{dx}}px, ${{dy}}px)` }}, {{ transform: 'translate(0, 0)' }}], \
             {{ duration: 240, easing: 'cubic-bezier(0.2, 0.8, 0.2, 1)' }} \
           ); \
         }});",
        motion = prefers_motion_script()
    )
}

/// Keeps the board's copy current and reports the full list to the host.
#[derive(Clone, Copy)]
struct BoardSink {
    items: Signal<Vec<WorkItem>>,
    on_update: Option<EventHandler<Vec<WorkItem>>>,
    flip: Signal<Flip>,
}

impl BoardSink {
    fn commit(self, next: Vec<WorkItem>) {
        let mut items = self.items;
        items.set(next.clone());
        if let Some(handler) = self.on_update {
            handler.call(next);
        }
    }

    /// Commits a card move. The rects are read before the new list renders.
    fn commit_move(self, next: Vec<WorkItem>) {
        let board = self.flip.peek().board.clone();
        document::eval(&snapshot_js(&board));
        let mut flip = self.flip;
        let seq = flip.peek().seq.wrapping_add(1);
        flip.set(Flip { board, seq });
        self.commit(next);
    }
}

#[derive(Clone, Default, PartialEq)]
struct TouchHold {
    token: u32,
    armed: Option<String>,
    origin: (f64, f64),
    active: bool,
    moved: bool,
}

/// The drag state shared by the mouse and touch paths.
#[derive(Clone, Copy)]
struct Drag {
    sink: BoardSink,
    dragging: Signal<Option<String>>,
    over: Signal<Option<String>>,
    drop_at: Signal<Option<DropAt>>,
    announce: Signal<String>,
    touch: Signal<TouchHold>,
}

impl Drag {
    fn set_over(self, column: &str) {
        if self.over.peek().as_deref() != Some(column) {
            let mut over = self.over;
            over.set(Some(column.to_string()));
        }
    }

    fn set_drop(self, target: DropAt) {
        if self.drop_at.peek().as_ref() != Some(&target) {
            let mut drop_at = self.drop_at;
            drop_at.set(Some(target));
        }
    }

    fn reset(self) {
        let mut dragging = self.dragging;
        dragging.set(None);
        let mut over = self.over;
        over.set(None);
        let mut drop_at = self.drop_at;
        drop_at.set(None);
    }

    fn drop_onto(self, card: &str, column: &str, before: Option<&str>, columns: &[StatusColumn]) {
        let current = self.sink.items.peek().clone();
        let Some(next) = move_to_position(&current, card, column, before) else {
            return;
        };
        if next == current {
            return;
        }
        let mut announce = self.announce;
        announce.set(format!("Moved to {}", column_label(columns, column)));
        self.sink.commit_move(next);
    }

    /// Ends a touch drag. `at` is where the finger lifted; `None` (a cancelled touch) keeps the
    /// last reported target.
    fn finish_touch(self, columns: &[StatusColumn], at: Option<(f64, f64)>) {
        let mut hold = self.touch;
        let was_active = {
            let mut t = hold.write();
            let was = t.active;
            t.token += 1;
            t.armed = None;
            t.active = false;
            t.moved = was;
            was
        };
        if !was_active {
            return;
        }
        let card = self.dragging.peek().clone();
        let fallback = self.drop_at.peek().clone();
        self.reset();
        document::eval(TOUCH_RELEASE_JS);
        let Some(card) = card else { return };
        let columns = columns.to_vec();
        spawn(async move {
            let probed = match at {
                Some((x, y)) => probe_target(x, y).await,
                None => None,
            };
            if let Some((column, before)) = probed.or(fallback) {
                self.drop_onto(&card, &column, before.as_deref(), &columns);
            }
        });
    }
}

fn focus_card(id: &str) {
    document::eval(&format!(
        "requestAnimationFrame(() => document.getElementById({id:?})?.focus({{ preventScroll: true }}));"
    ));
}

/// Element at the pointer, as `[column id, card id]`; each `null` when not on one.
fn probe_js(x: f64, y: f64) -> String {
    format!(
        "const el = document.elementFromPoint({x}, {y}); \
         const col = el && el.closest('[data-kb-col]'); \
         const card = el && el.closest('[data-kb-card]'); \
         return [col ? col.dataset.kbCol : null, card ? card.dataset.kbCard : null];"
    )
}

/// The drop target under a viewport point, or `None` when the point is off every column.
async fn probe_target(x: f64, y: f64) -> Option<DropAt> {
    let value = document::eval(&probe_js(x, y)).await.ok()?;
    let parts = value.as_array()?;
    let column = parts.first()?.as_str()?.to_string();
    let card = parts.get(1).and_then(|v| v.as_str()).map(str::to_string);
    Some((column, card))
}

#[component]
pub fn KanbanTaskBoard(
    breadcrumb_segments: Vec<String>,
    #[props(default = "Board".to_string(), into)] board_name: String,
    status_columns: Vec<StatusColumn>,
    work_items: Vec<WorkItem>,
    #[props(default)] tag_options: Vec<TagOption>,
    #[props(default)] member_options: Vec<MemberOption>,
    #[props(default)] starred_ids: Vec<String>,
    #[props(default)] on_items_update: Option<EventHandler<Vec<WorkItem>>>,
    #[props(default)] initial_view: ViewMode,
    #[props(default, into)] extra_classes: String,
) -> Element {
    let items = use_signal(|| work_items.clone());
    let mut search = use_signal(String::new);
    let mut label_filter = use_signal(Vec::<String>::new);
    let mut view = use_signal(|| initial_view);
    let dragging = use_signal(|| None::<String>);
    let over = use_signal(|| None::<String>);
    let drop_at = use_signal(|| None::<DropAt>);
    let mut open_card = use_signal(|| None::<String>);
    let mut adding = use_signal(|| None::<String>);
    let mut new_title = use_signal(String::new);
    let announce = use_signal(String::new);
    let mut shown_limit = use_signal(|| COLUMN_LIMIT);
    let starred = use_signal(|| starred_ids.clone());
    let mut starred_only = use_signal(|| false);
    let touch = use_signal(TouchHold::default);
    let touch_probing = use_signal(|| false);
    let board_dom = use_hook(|| format!("kit-kb-{}", next_id()));
    let flip = use_signal({
        let board_dom = board_dom.clone();
        move || Flip {
            board: board_dom.clone(),
            seq: 0,
        }
    });
    use_effect({
        let board_dom = board_dom.clone();
        move || {
            document::eval(&touch_guard_js(&board_dom));
        }
    });
    use_effect(move || {
        let Flip { board, seq } = flip();
        if seq == 0 {
            return;
        }
        document::eval(&play_js(&board));
    });

    let sink = BoardSink {
        items,
        on_update: on_items_update,
        flip,
    };
    let drag = Drag {
        sink,
        dragging,
        over,
        drop_at,
        announce,
        touch,
    };

    let board = items();
    let starred_now = starred();
    let visible: Vec<&WorkItem> = filter_items(&board, &search(), &label_filter())
        .into_iter()
        .filter(|i| !starred_only() || starred_now.contains(&i.id))
        .collect();
    let total = board.len();
    let compact = view() == ViewMode::Compact;
    let root_class = class_list("kit-kb", &extra_classes);
    let open_item = open_card().and_then(|id| board.iter().find(|i| i.id == id).cloned());
    let columns = status_columns.clone();
    let shown_total = visible.len();
    let count_text = if shown_total == total {
        format!("{total} tasks")
    } else {
        format!("{shown_total} of {total} tasks")
    };
    let limit = shown_limit();
    let dragging_now = dragging();
    let drop_now = drop_at();
    let over_now = over();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { id: "{board_dom}", class: "{root_class}", "aria-label": "{board_name}", "data-compact": compact.then_some("true"),
            header { class: "kit-kb__top",
                nav { class: "kit-kb__crumbs", "aria-label": "Breadcrumb",
                    ol {
                        for (i, seg) in breadcrumb_segments.iter().enumerate() {
                            li { key: "{i}", "{seg}" }
                        }
                    }
                }
                h2 { class: "kit-kb__name", "{board_name}" }
            }
            div { class: "kit-kb__toolbar",
                span { class: "kit-kb__count", "{count_text}" }
                input {
                    class: "kit-kb__search",
                    r#type: "search",
                    "aria-label": "Search tasks",
                    placeholder: "Search",
                    value: "{search}",
                    oninput: move |e: FormEvent| search.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Escape && !search().is_empty() {
                            e.prevent_default();
                            search.set(String::new());
                        }
                    },
                }
                div { class: "kit-kb__labels", role: "group", "aria-label": "Filter by label",
                    for tag in tag_options.iter() {
                        {
                            let on = label_filter().contains(&tag.id);
                            let tid = tag.id.clone();
                            rsx! {
                                button {
                                    key: "{tag.id}",
                                    r#type: "button",
                                    class: "kit-kb__chip",
                                    "aria-pressed": if on { "true" } else { "false" },
                                    onclick: move |_| {
                                        let mut next = label_filter();
                                        match next.iter().position(|l| *l == tid) {
                                            Some(at) => { next.remove(at); }
                                            None => next.push(tid.clone()),
                                        }
                                        label_filter.set(next);
                                    },
                                    "{tag.label}"
                                }
                            }
                        }
                    }
                }
                div { class: "kit-kb__view", role: "group", "aria-label": "View",
                    button {
                        r#type: "button",
                        class: "kit-kb__chip",
                        "aria-pressed": if starred_only() { "true" } else { "false" },
                        onclick: move |_| starred_only.set(!starred_only()),
                        "Starred"
                    }
                    button {
                        r#type: "button",
                        class: "kit-kb__chip",
                        "aria-pressed": if compact { "true" } else { "false" },
                        onclick: move |_| view.set(if compact { ViewMode::Comfortable } else { ViewMode::Compact }),
                        "Compact"
                    }
                }
            }
            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
            div { class: "kit-kb__columns",
                for column in columns.iter() {
                    {
                        let cid = column.id.clone();
                        let cards: Vec<WorkItem> = visible.iter().filter(|i| i.status_id == cid).map(|i| (*i).clone()).collect();
                        let count = cards.len();
                        let overflow = count > limit;
                        let remaining = count.saturating_sub(limit);
                        let heading_id = format!("kit-kb-{cid}-h");
                        let add_id = format!("kit-kb-{cid}-add");
                        let adding_here = adding() == Some(cid.clone());
                        let is_over = over_now.as_deref() == Some(cid.as_str()) && dragging_now.is_some();
                        let end_line = dragging_now.is_some() && drop_now == Some((cid.clone(), None));
                        let quick_cid = cid.clone();
                        let column_cid = cid.clone();
                        let columns_for_drop = columns.clone();
                        rsx! {
                            section { key: "{cid}", class: "kit-kb__column", "aria-labelledby": "{heading_id}",
                                "data-kb-col": "{cid}",
                                "data-over": is_over.then_some("true"),
                                ondragover: move |e: DragEvent| {
                                    if dragging.peek().is_none() { return; }
                                    e.prevent_default();
                                    drag.set_over(&column_cid);
                                    drag.set_drop((column_cid.clone(), None));
                                },
                                ondrop: {
                                    let target = cid.clone();
                                    move |e: DragEvent| {
                                        e.prevent_default();
                                        let Some(card) = dragging.peek().clone() else { return };
                                        let before = match drop_at.peek().as_ref() {
                                            Some((col, before)) if *col == target => before.clone(),
                                            _ => None,
                                        };
                                        drag.drop_onto(&card, &target, before.as_deref(), &columns_for_drop);
                                        drag.reset();
                                    }
                                },
                                header { class: "kit-kb__col-head",
                                    h3 { id: "{heading_id}", class: "kit-kb__col-title", "{column.label} " span { class: "kit-kb__count", "{count}" } }
                                    button {
                                        id: "{add_id}",
                                        r#type: "button",
                                        class: "kit-kb__add",
                                        "aria-label": "Add a task to {column.label}",
                                        onclick: move |_| {
                                            adding.set(Some(quick_cid.clone()));
                                            new_title.set(String::new());
                                            let field = format!("kit-kb-{quick_cid}-field");
                                            document::eval(&format!("requestAnimationFrame(() => document.getElementById({field:?})?.focus());"));
                                        },
                                        "+"
                                    }
                                }
                                if adding_here {
                                    {
                                        let field_id = format!("kit-kb-{cid}-field");
                                        let add_target = cid.clone();
                                        let back_to = add_id.clone();
                                        rsx! {
                                            input {
                                                id: "{field_id}",
                                                class: "kit-kb__quick",
                                                r#type: "text",
                                                "aria-label": "New task title",
                                                placeholder: "Task title",
                                                value: "{new_title}",
                                                oninput: move |e: FormEvent| new_title.set(e.value()),
                                                onkeydown: move |e: KeyboardEvent| {
                                                    match e.key() {
                                                        Key::Enter => {
                                                            let title = new_title().trim().to_string();
                                                            if title.is_empty() { return; }
                                                            let n = board_len_next(&items());
                                                            let id = format!("new-{n}");
                                                            let mut next = items();
                                                            next.push(WorkItem {
                                                                id: id.clone(),
                                                                identifier_line: String::new(),
                                                                title,
                                                                status_id: add_target.clone(),
                                                                priority: 0,
                                                                label_ids: Vec::new(),
                                                                assignee_id: None,
                                                                due_in_days: None,
                                                                subtasks_done: 0,
                                                                subtasks_total: 0,
                                                                pull_request_ref: None,
                                                            });
                                                            adding.set(None);
                                                            new_title.set(String::new());
                                                            sink.commit(next);
                                                            focus_card(&format!("kit-kb-card-{id}"));
                                                        }
                                                        Key::Escape => {
                                                            e.prevent_default();
                                                            adding.set(None);
                                                            focus_after_frame(&back_to);
                                                        }
                                                        _ => {}
                                                    }
                                                },
                                            }
                                        }
                                    }
                                }
                                ul { class: "kit-kb__cards",
                                    for (i, card) in cards.iter().take(limit).enumerate() {
                                        {
                                            let card_dom = format!("kit-kb-card-{}", card.id);
                                            let card_id = card.id.clone();
                                            let start_id = card.id.clone();
                                            let touch_id = card.id.clone();
                                            let key_id = card.id.clone();
                                            let key_board = items();
                                            let key_columns = columns.clone();
                                            let key_cid = card.id.clone();
                                            let star_id = card.id.clone();
                                            let line_here = dragging_now.is_some() && drop_now == Some((cid.clone(), Some(card.id.clone())));
                                            let is_starred = starred_now.contains(&card.id);
                                            let selected = open_card() == Some(card.id.clone());
                                            let due = card.due_in_days.map(due_words);
                                            let late = card.due_in_days.is_some_and(|d| d < 0);
                                            let title = card.title.clone();
                                            let identifier = card.identifier_line.clone();
                                            let star_label = format!("Star {}", card.title);
                                            let labels: Vec<String> = card.label_ids.iter().filter_map(|l| tag_options.iter().find(|t| t.id == *l).map(|t| t.label.clone())).collect();
                                            let assignee = card.assignee_id.as_ref().and_then(|a| member_options.iter().find(|m| m.id == *a).cloned());
                                            let priority = priority_word(card.priority);
                                            let hover_cid = cid.clone();
                                            let hover_card = card.id.clone();
                                            let columns_for_up = columns.clone();
                                            let columns_for_cancel = columns.clone();
                                            let board_lift = board_dom.clone();
                                            rsx! {
                                                li { key: "{card.id}", class: "kit-kb__card-item", "data-kb-card": "{card.id}", style: "--kit-i: {i};",
                                                    ondragover: move |e: DragEvent| {
                                                        if dragging.peek().is_none() { return; }
                                                        e.prevent_default();
                                                        e.stop_propagation();
                                                        drag.set_over(&hover_cid);
                                                        drag.set_drop((hover_cid.clone(), Some(hover_card.clone())));
                                                    },
                                                    if line_here {
                                                        span { class: "kit-kb__drop", "aria-hidden": "true" }
                                                    }
                                                    button {
                                                        id: "{card_dom}",
                                                        r#type: "button",
                                                        class: "kit-kb__card",
                                                        draggable: "true",
                                                        "aria-pressed": if selected { "true" } else { "false" },
                                                        "data-selected": selected.then_some("true"),
                                                        "data-dragging": (dragging_now.as_deref() == Some(card.id.as_str())).then_some("true"),
                                                        ondragstart: move |_| {
                                                            let mut dragging = dragging;
                                                            dragging.set(Some(start_id.clone()));
                                                        },
                                                        ondragend: move |_| drag.reset(),
                                                        onpointerdown: move |e: PointerEvent| {
                                                            if e.pointer_type() != "touch" { return; }
                                                            let p = e.client_coordinates();
                                                            let token = touch.peek().token + 1;
                                                            let armed = touch_id.clone();
                                                            let lift_board = board_lift.clone();
                                                            let mut hold = touch;
                                                            hold.set(TouchHold { token, armed: Some(armed.clone()), origin: (p.x, p.y), active: false, moved: false });
                                                            spawn(async move {
                                                                sleep(HOLD_MS).await;
                                                                let lift = {
                                                                    let t = touch.peek();
                                                                    t.token == token && t.armed.as_deref() == Some(armed.as_str()) && !t.active
                                                                };
                                                                if !lift { return; }
                                                                let mut hold = touch;
                                                                hold.write().active = true;
                                                                let mut lifted = dragging;
                                                                lifted.set(Some(armed));
                                                                let mut over = drag.over;
                                                                over.set(None);
                                                                let mut drop_at = drag.drop_at;
                                                                drop_at.set(None);
                                                                document::eval(&format!("window.__kitKbActive = {lift_board:?};"));
                                                            });
                                                        },
                                                        onpointermove: move |e: PointerEvent| {
                                                            if e.pointer_type() != "touch" { return; }
                                                            let p = e.client_coordinates();
                                                            let (active, origin, armed) = {
                                                                let t = touch.peek();
                                                                (t.active, t.origin, t.armed.is_some())
                                                            };
                                                            if !armed { return; }
                                                            if !active {
                                                                let travel = ((p.x - origin.0).powi(2) + (p.y - origin.1).powi(2)).sqrt();
                                                                if travel > HOLD_SLOP_PX {
                                                                    let mut hold = touch;
                                                                    let mut t = hold.write();
                                                                    t.token += 1;
                                                                    t.armed = None;
                                                                }
                                                                return;
                                                            }
                                                            if *touch_probing.peek() { return; }
                                                            let mut probing = touch_probing;
                                                            probing.set(true);
                                                            let (x, y) = (p.x, p.y);
                                                            spawn(async move {
                                                                let found = probe_target(x, y).await;
                                                                let mut probing = touch_probing;
                                                                probing.set(false);
                                                                if dragging.peek().is_none() { return; }
                                                                if let Some((column, card)) = found {
                                                                    drag.set_over(&column);
                                                                    drag.set_drop((column, card));
                                                                }
                                                            });
                                                        },
                                                        onpointerup: move |e: PointerEvent| {
                                                            if e.pointer_type() != "touch" { return; }
                                                            let p = e.client_coordinates();
                                                            drag.finish_touch(&columns_for_up, Some((p.x, p.y)));
                                                        },
                                                        onpointercancel: move |e: PointerEvent| {
                                                            if e.pointer_type() == "touch" { drag.finish_touch(&columns_for_cancel, None); }
                                                        },
                                                        onclick: move |_| {
                                                            if touch.peek().moved {
                                                                let mut hold = touch;
                                                                hold.write().moved = false;
                                                                return;
                                                            }
                                                            open_card.set(Some(card_id.clone()));
                                                        },
                                                        onkeydown: move |e: KeyboardEvent| {
                                                            let next = match e.key() {
                                                                Key::ArrowLeft => move_column(&key_board, &key_columns, &key_id, -1),
                                                                Key::ArrowRight => move_column(&key_board, &key_columns, &key_id, 1),
                                                                Key::ArrowUp => reorder_in_column(&key_board, &key_id, -1),
                                                                Key::ArrowDown => reorder_in_column(&key_board, &key_id, 1),
                                                                _ => None,
                                                            };
                                                            if let Some(next) = next {
                                                                e.prevent_default();
                                                                let mut announce = announce;
                                                                announce.set(format!("Moved {}", key_cid));
                                                                sink.commit_move(next);
                                                                focus_card(&format!("kit-kb-card-{key_id}"));
                                                            }
                                                        },
                                                        span { class: "kit-kb__id", "{identifier}" }
                                                        span { class: "kit-kb__title", "{title}" }
                                                        if !labels.is_empty() {
                                                            span { class: "kit-kb__labels-line",
                                                                for label in labels.iter() {
                                                                    Badge { key: "{label}", tone: BadgeTone::Neutral, "{label}" }
                                                                }
                                                            }
                                                        }
                                                        span { class: "kit-kb__foot",
                                                            span { class: "kit-kb__priority", "{priority}" }
                                                            if card.subtasks_total > 0 {
                                                                span { class: "kit-kb__sub", "{card.subtasks_done}/{card.subtasks_total}" }
                                                            }
                                                            if let Some(words) = due.clone() {
                                                                span { class: "kit-kb__due", "data-late": late.then_some("true"), "{words}" }
                                                            }
                                                            if let Some(pr) = card.pull_request_ref.clone() {
                                                                span { class: "kit-kb__pr", "PR {pr}" }
                                                            }
                                                            if let Some(member) = assignee.clone() {
                                                                span { class: "kit-kb__avatar", "aria-label": "Assigned to {member.name}", "{member.avatar}" }
                                                            } else {
                                                                span { class: "kit-kb__avatar kit-kb__avatar--none", "aria-label": "Unassigned" }
                                                            }
                                                        }
                                                    }
                                                    button {
                                                        r#type: "button",
                                                        class: "kit-kb__star",
                                                        "aria-pressed": if is_starred { "true" } else { "false" },
                                                        "aria-label": "{star_label}",
                                                        onclick: move |_| toggle_id(starred, &star_id),
                                                        span { "aria-hidden": "true", "★" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if overflow {
                                        li { class: "kit-kb__more-item",
                                            button {
                                                r#type: "button",
                                                class: "kit-kb__more",
                                                onclick: move |_| shown_limit.set(limit + COLUMN_LIMIT),
                                                "Show more ({remaining} left)"
                                            }
                                        }
                                    }
                                }
                                if end_line {
                                    div { class: "kit-kb__drop", "aria-hidden": "true" }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(item) = open_item {
                {
                    let item_id = item.id.clone();
                    let return_dom = format!("kit-kb-card-{}", item.id);
                    let status_now = item.status_id.clone();
                    let priority_now = item.priority;
                    let status_cols = columns.clone();
                    let status_count = status_cols.len();
                    let priority_key = item_id.clone();
                    let status_key = item_id.clone();
                    let star_key = item_id.clone();
                    let assignee_name = item.assignee_id.as_ref().and_then(|a| member_options.iter().find(|m| m.id == *a)).map(|m| m.name.clone()).unwrap_or_else(|| "Unassigned".to_string());
                    let sub = format!("{} of {} subtasks done", item.subtasks_done, item.subtasks_total);
                    let panel_starred = starred_now.contains(&item_id);
                    let status_cols_for_keys = status_cols.clone();
                    rsx! {
                        aside { class: "kit-kb__panel", role: "dialog", "aria-modal": "false", "aria-labelledby": "kit-kb-panel-title",
                            onkeydown: move |e: KeyboardEvent| {
                                if e.key() == Key::Escape {
                                    e.prevent_default();
                                    open_card.set(None);
                                    focus_after_frame(&return_dom);
                                }
                            },
                            header { class: "kit-kb__panel-head",
                                h3 { id: "kit-kb-panel-title", class: "kit-kb__panel-title", "{item.title}" }
                                button {
                                    r#type: "button",
                                    class: "kit-kb__add",
                                    "aria-label": "Close task",
                                    onclick: move |_| open_card.set(None),
                                    "×"
                                }
                            }
                            fieldset { class: "kit-kb__radio-group",
                                legend { "Status" }
                                div { class: "kit-kb__radios", role: "radiogroup", "aria-label": "Status",
                                    for (idx, column) in status_cols.iter().enumerate() {
                                        {
                                            let checked = column.id == status_now;
                                            let target = column.id.clone();
                                            let item_key = status_key.clone();
                                            let key_item = status_key.clone();
                                            let key_cols = status_cols_for_keys.clone();
                                            let status_dom = format!("kit-kb-status-{}", column.id);
                                            rsx! {
                                                button {
                                                    key: "{column.id}",
                                                    id: "{status_dom}",
                                                    r#type: "button",
                                                    role: "radio",
                                                    class: "kit-kb__chip",
                                                    "aria-checked": if checked { "true" } else { "false" },
                                                    tabindex: if checked { "0" } else { "-1" },
                                                    onclick: move |_| {
                                                        let current = sink.items.peek().clone();
                                                        if let Some(next) = move_to_status(&current, &item_key, &target) {
                                                            sink.commit_move(next);
                                                        }
                                                    },
                                                    onkeydown: move |e: KeyboardEvent| {
                                                        let forward = match e.key() {
                                                            Key::ArrowRight | Key::ArrowDown => true,
                                                            Key::ArrowLeft | Key::ArrowUp => false,
                                                            _ => return,
                                                        };
                                                        e.prevent_default();
                                                        let to = radio_step(idx, status_count, forward);
                                                        if to == idx { return; }
                                                        let target = key_cols[to].id.clone();
                                                        let current = sink.items.peek().clone();
                                                        if let Some(next) = move_to_status(&current, &key_item, &target) {
                                                            let mut announce = announce;
                                                            announce.set(format!("Moved to {}", key_cols[to].label));
                                                            sink.commit_move(next);
                                                            focus_after_frame(&format!("kit-kb-status-{}", target));
                                                        }
                                                    },
                                                    "{column.label}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            fieldset { class: "kit-kb__radio-group",
                                legend { "Priority" }
                                div { class: "kit-kb__radios", role: "radiogroup", "aria-label": "Priority",
                                    for level in 0_u8..=4 {
                                        {
                                            let checked = level == priority_now;
                                            let item_key = priority_key.clone();
                                            let key_item = priority_key.clone();
                                            let priority_dom = format!("kit-kb-priority-{level}");
                                            rsx! {
                                                button {
                                                    key: "{level}",
                                                    id: "{priority_dom}",
                                                    r#type: "button",
                                                    role: "radio",
                                                    class: "kit-kb__chip",
                                                    "aria-checked": if checked { "true" } else { "false" },
                                                    tabindex: if checked { "0" } else { "-1" },
                                                    onclick: move |_| {
                                                        let mut next = sink.items.peek().clone();
                                                        if let Some(card) = next.iter_mut().find(|i| i.id == item_key) {
                                                            card.priority = level;
                                                        }
                                                        sink.commit(next);
                                                    },
                                                    onkeydown: move |e: KeyboardEvent| {
                                                        let forward = match e.key() {
                                                            Key::ArrowRight | Key::ArrowDown => true,
                                                            Key::ArrowLeft | Key::ArrowUp => false,
                                                            _ => return,
                                                        };
                                                        e.prevent_default();
                                                        let to = radio_step(level as usize, 5, forward) as u8;
                                                        if to == level { return; }
                                                        let mut next = sink.items.peek().clone();
                                                        if let Some(card) = next.iter_mut().find(|i| i.id == key_item) {
                                                            card.priority = to;
                                                        }
                                                        let mut announce = announce;
                                                        announce.set(format!("Priority {}", priority_word(to)));
                                                        sink.commit(next);
                                                        focus_after_frame(&format!("kit-kb-priority-{to}"));
                                                    },
                                                    "{priority_word(level)}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            button {
                                r#type: "button",
                                class: "kit-kb__chip",
                                "aria-pressed": if panel_starred { "true" } else { "false" },
                                onclick: move |_| toggle_id(starred, &star_key),
                                "Star task"
                            }
                            p { class: "kit-kb__meta", "Assignee: {assignee_name}" }
                            p { class: "kit-kb__meta", "{sub}" }
                        }
                    }
                }
            }
        }
    }
}

/// Next id number for a new card: one more than the count of cards already on the board.
pub fn board_len_next(items: &[WorkItem]) -> usize {
    items.len() + 1
}
