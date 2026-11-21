//! Email draft: an agent-written email waiting for review. The recipient can be
//! switched, the attachment removed, the tone changed and the text edited. Sending
//! or discarding folds the card into a one-line receipt with Undo.
//!
//! Port losses:
//! - The recipient menu closes on an outside press or Escape through a one-off
//!   listener per opening. A menu closed by picking a recipient keeps that listener
//!   until the next outside press.
//! - The height glide starts from a ResizeObserver set up in eval after mount, so
//!   the first render is not guarded against a transition.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, sleep};

const CSS: Asset = asset!("/assets/email-draft.css");

#[derive(Clone, PartialEq)]
pub struct DraftContact {
    pub name: String,
    pub email: String,
}

#[derive(Clone)]
pub struct DraftTone {
    pub id: String,
    pub label: String,
    pub body: fn(&DraftContact) -> String,
}

/// Tones are told apart by id and label; function pointers are not compared.
impl PartialEq for DraftTone {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.label == other.label
    }
}

#[derive(Clone, PartialEq)]
pub struct DraftLink {
    pub label: String,
    pub href: String,
}

#[derive(Clone, PartialEq)]
pub struct SentEmail {
    pub to: DraftContact,
    pub body: String,
    pub attachment: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Draft,
    Sending,
    Sent,
    Discarded,
}

/// `"Jonas Weber"` → `"Jonas"`.
pub fn first_name(name: &str) -> &str {
    name.split_whitespace().next().unwrap_or("")
}

/// Up to two capitals, one per word, from the first letter or digit of each word.
/// Emoji and other words without one are skipped.
pub fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().find(|c| c.is_alphanumeric()))
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Replaces the sender's full name with the new recipient's full name, and the
/// sender's first name as a whole word with the new first name. One pass, so a
/// replaced span is never matched again.
pub fn retarget_names(text: &str, from: &DraftContact, to: &DraftContact) -> String {
    let full = from.name.trim();
    let first = first_name(&from.name);
    let new_full = to.name.trim();
    let new_first = first_name(&to.name);
    let mut out = String::with_capacity(text.len());
    let mut pos = 0;
    while pos < text.len() {
        let rest = &text[pos..];
        if !full.is_empty() && rest.starts_with(full) {
            out.push_str(new_full);
            pos += full.len();
        } else if !first.is_empty()
            && rest.starts_with(first)
            && text[..pos].chars().next_back().is_none_or(|c| !is_word(c))
            && text[pos + first.len()..]
                .chars()
                .next()
                .is_none_or(|c| !is_word(c))
        {
            out.push_str(new_first);
            pos += first.len();
        } else {
            let ch = rest.chars().next().unwrap_or_default();
            out.push(ch);
            pos += ch.len_utf8();
        }
    }
    out
}

/// The sent body: the text, then the link as `label: href`.
pub fn email_body(body: &str, link: &DraftLink) -> String {
    format!("{body} {}: {}", link.label, link.href)
}

/// Badge text and tone for each phase.
pub fn badge_for(phase: Phase) -> (&'static str, &'static str) {
    match phase {
        Phase::Sent => ("Sent", "ok"),
        Phase::Discarded => ("Discarded", "muted"),
        Phase::Draft | Phase::Sending => ("Ready for review", "ok"),
    }
}

fn friendly_body(contact: &DraftContact) -> String {
    format!(
        "Hi {}, a heads-up that your Growth plan renews on October 9 for $8,900 a year. If you'd like to add seats or switch to monthly billing, you can review the quote here:",
        first_name(&contact.name)
    )
}

fn short_body(contact: &DraftContact) -> String {
    format!(
        "Hi {}, your Growth plan renews on October 9 ($8,900/year). Quote for changes:",
        first_name(&contact.name)
    )
}

fn formal_body(contact: &DraftContact) -> String {
    format!(
        "Dear {}, this is a reminder that your Growth subscription is scheduled to renew on October 9 at $8,900 per year. Should you wish to adjust seats or billing, please review the attached quote:",
        contact.name
    )
}

fn default_contacts() -> Vec<DraftContact> {
    vec![
        DraftContact {
            name: "Jonas Weber".into(),
            email: "jonas@northwind.io".into(),
        },
        DraftContact {
            name: "Lena Hart".into(),
            email: "lena@northwind.io".into(),
        },
        DraftContact {
            name: "Finance team".into(),
            email: "billing@northwind.io".into(),
        },
    ]
}

fn default_tones() -> Vec<DraftTone> {
    vec![
        DraftTone {
            id: "friendly".into(),
            label: "Friendly".into(),
            body: friendly_body,
        },
        DraftTone {
            id: "short".into(),
            label: "Short".into(),
            body: short_body,
        },
        DraftTone {
            id: "formal".into(),
            label: "Formal".into(),
            body: formal_body,
        },
    ]
}

fn default_link() -> DraftLink {
    DraftLink {
        label: "Review quote".into(),
        href: "#".into(),
    }
}

/// Stands in for a recipient when there are no contacts. Its empty address keeps
/// Send disabled.
fn no_one() -> DraftContact {
    DraftContact {
        name: String::new(),
        email: String::new(),
    }
}

fn outside_press_script(menu_id: &str) -> String {
    format!(
        "await new Promise((resolve) => {{
           const done = () => {{
             document.removeEventListener('pointerdown', away);
             document.removeEventListener('keydown', esc);
             resolve(true);
           }};
           const away = (e) => {{
             const box = document.getElementById({menu_id:?});
             if (!box || !box.contains(e.target)) done();
           }};
           const esc = (e) => {{ if (e.key === 'Escape') done(); }};
           document.addEventListener('pointerdown', away);
           document.addEventListener('keydown', esc);
         }});
         return true;"
    )
}

fn start_editing(
    mut edited: Signal<Option<String>>,
    mut editing: Signal<bool>,
    body: String,
    text_id: &str,
) {
    edited.set(Some(body));
    editing.set(true);
    document::eval(&format!(
        "requestAnimationFrame(() => {{
           const el = document.getElementById({text_id:?});
           if (el) {{ el.focus(); el.setSelectionRange(el.value.length, el.value.length); }}
         }});"
    ));
}

fn send(
    mut phase: Signal<Phase>,
    mut editing: Signal<bool>,
    to: DraftContact,
    body: String,
    link: DraftLink,
    file: Option<String>,
    on_send: Option<EventHandler<SentEmail>>,
) {
    if phase() != Phase::Draft || to.email.is_empty() {
        return;
    }
    editing.set(false);
    phase.set(Phase::Sending);
    spawn(async move {
        sleep(900).await;
        phase.set(Phase::Sent);
        if let Some(handler) = on_send {
            handler.call(SentEmail {
                to,
                body: email_body(&body, &link),
                attachment: file,
            });
        }
    });
}

#[component]
pub fn EmailDraft(
    /// Reference of the record, e.g. a subscription or invoice number.
    #[props(default = "SUB-1182".to_string(), into)]
    reference: String,
    /// Account or company under the reference.
    #[props(default = "Northwind Logistics".to_string(), into)]
    account: String,
    /// Amount shown on the right.
    #[props(default = "$8,900".to_string(), into)]
    amount: String,
    /// Status chip next to the reference.
    #[props(default = Some("Renews in 5 days".to_string()), into)]
    status: Option<String>,
    /// Who wrote the draft.
    #[props(default = "Renewal agent".to_string(), into)]
    agent: String,
    #[props(default = default_contacts())] contacts: Vec<DraftContact>,
    #[props(default = Some("Renewal-quote.pdf".to_string()), into)] attachment: Option<String>,
    /// Text variants to switch between; the first one is shown first.
    #[props(default = default_tones())]
    tones: Vec<DraftTone>,
    /// Label and target of the link appended to the email.
    #[props(default = default_link())]
    link: DraftLink,
    #[props(default)] on_send: Option<EventHandler<SentEmail>>,
    #[props(default)] on_discard: Option<EventHandler<()>>,
    #[props(default)] class: String,
) -> Element {
    let mut to = use_signal(|| contacts.first().cloned().unwrap_or_else(no_one));
    let mut file = use_signal(|| attachment.clone());
    let mut tone = use_signal(|| tones.first().map(|t| t.id.clone()).unwrap_or_default());
    // Edited text wins over the generated variant until the tone or recipient changes.
    let mut edited = use_signal(|| None::<String>);
    let mut editing = use_signal(|| false);
    let mut menu = use_signal(|| false);
    let mut phase = use_signal(|| Phase::Draft);
    let mut tone_key = use_signal(|| 0u32);
    let id = use_hook(|| format!("ui-ed-{}", next_id()));
    let outer_id = format!("{id}-outer");
    let inner_id = format!("{id}-inner");
    let menu_id = format!("{id}-to");
    let text_id = format!("{id}-text");

    let generated = tones
        .iter()
        .find(|t| t.id == tone())
        .map(|t| (t.body)(&to()))
        .unwrap_or_default();
    let body = edited().unwrap_or(generated);

    let menu_for_close = menu_id.clone();
    use_effect(use_reactive((&menu(),), move |(is_open,)| {
        if !is_open {
            return;
        }
        let script = outside_press_script(&menu_for_close);
        spawn(async move {
            let _ = document::eval(&script).await;
            menu.set(false);
        });
    }));

    let text_for_grow = text_id.clone();
    use_effect(use_reactive(
        (&editing(), &body),
        move |(_editing, _body)| {
            document::eval(&format!(
                "const el = document.getElementById({text_for_grow:?});
             if (el) {{ el.style.height = 'auto'; el.style.height = el.scrollHeight + 'px'; }}"
            ));
        },
    ));

    let outer_for_sync = outer_id.clone();
    let inner_for_sync = inner_id.clone();
    use_effect(move || {
        document::eval(&format!(
            "const o = document.getElementById({outer_for_sync:?});
             const i = document.getElementById({inner_for_sync:?});
             if (!o || !i) return;
             const sync = () => {{ o.style.height = i.offsetHeight + 'px'; }};
             sync();
             const ro = new ResizeObserver(sync);
             ro.observe(i);
             requestAnimationFrame(() => o.setAttribute('data-smooth', ''));
             o.__sync = ro;"
        ));
    });
    let outer_for_drop = outer_id.clone();
    use_drop(move || {
        document::eval(&format!(
            "document.getElementById({outer_for_drop:?})?.__sync?.disconnect();"
        ));
    });

    let current_to = to();
    let current_file = file();
    let link_for_send = link.clone();
    let body_for_send = body.clone();
    let file_for_send = current_file.clone();
    let send_click = move || {
        send(
            phase,
            editing,
            to(),
            body_for_send.clone(),
            link_for_send.clone(),
            file_for_send.clone(),
            on_send,
        );
    };
    let send_key = send_click.clone();
    let body_for_edit = body.clone();
    let text_for_edit = text_id.clone();

    let open = matches!(phase(), Phase::Draft | Phase::Sending);
    let (badge_text, badge_tone) = badge_for(phase());
    let sending = phase() == Phase::Sending;
    let recipient_title = format!("{} <{}>", current_to.name, current_to.email);
    let recipient_name = current_to.name.clone();
    let recipient_email = current_to.email.clone();
    let has_recipient = !current_to.email.is_empty();
    let file_suffix = current_file
        .as_ref()
        .map(|f| format!(" with {f}"))
        .unwrap_or_default();
    let class = class_list("ui-ed", &class);
    let body_text = body.clone();
    let body_for_textarea = edited().unwrap_or_default();
    let agent_text = agent.clone();
    let link_label = link.label.clone();
    let link_href = link.href.clone();
    let current_tone = tone();
    let tone_key_value = tone_key();
    let edited_active = edited().is_some();
    let attach_label = attachment.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class,
            div { class: "ui-ed__record",
                div { class: "ui-ed__ref",
                    span { class: "ui-ed__id", title: "{reference}", "{reference}" }
                    if let Some(status) = status.clone() {
                        span { class: "ui-ed__status", dir: "auto", title: "{status}", "{status}" }
                    }
                    span { class: "ui-ed__amount", "{amount}" }
                }
                span { class: "ui-ed__account", dir: "auto", "{account}" }
            }

            div { id: "{outer_id}", class: "ui-ed__outer",
                div { id: "{inner_id}",
                    if open {
                        div {
                            class: "ui-ed__card",
                            "data-sending": sending.then_some(""),
                            div { class: "ui-ed__meta",
                                div { id: "{menu_id}", class: "ui-ed__to",
                                    span { class: "ui-ed__label", "to" }
                                    if has_recipient {
                                        button {
                                            r#type: "button",
                                            class: "ui-ed__recipient",
                                            aria_haspopup: "listbox",
                                            aria_expanded: if menu() { "true" } else { "false" },
                                            title: "{recipient_title}",
                                            onclick: move |_| menu.set(!menu()),
                                            span { class: "ui-ed__name", dir: "auto", "{recipient_name}" }
                                            " "
                                            span { class: "ui-ed__email", "({recipient_email})" }
                                            svg { view_box: "0 0 24 24", "aria-hidden": "true",
                                                path { d: "m7 10 5 5 5-5" }
                                            }
                                        }
                                    } else {
                                        span { class: "ui-ed__email", "No recipient" }
                                    }
                                    if menu() {
                                        ul { class: "ui-ed__menu", role: "listbox", aria_label: "Recipient",
                                            for (i, contact) in contacts.iter().enumerate() {
                                                {
                                                    let picked = contact.clone();
                                                    let previous = current_to.clone();
                                                    let selected = contact.email == current_to.email;
                                                    let initial_letters = initials(&contact.name);
                                                    let contact_name = contact.name.clone();
                                                    let contact_email = contact.email.clone();
                                                    let option_key = format!("{}-{i}", contact.email);
                                                    rsx! {
                                                        li { key: "{option_key}",
                                                            button {
                                                                r#type: "button",
                                                                role: "option",
                                                                aria_selected: if selected { "true" } else { "false" },
                                                                onclick: move |_| {
                                                                    let updated = edited().map(|text| retarget_names(&text, &previous, &picked));
                                                                    to.set(picked.clone());
                                                                    menu.set(false);
                                                                    if updated.is_some() {
                                                                        edited.set(updated);
                                                                    }
                                                                },
                                                                span { class: "ui-ed__avatar", "aria-hidden": "true", "{initial_letters}" }
                                                                span { class: "ui-ed__menu-text", dir: "auto",
                                                                    "{contact_name}"
                                                                    small { "{contact_email}" }
                                                                }
                                                                if selected {
                                                                    svg { class: "ui-ed__tick", view_box: "0 0 24 24", "aria-hidden": "true",
                                                                        path { d: "m6 12.5 4 4 8-9" }
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
                                if let Some(name) = current_file.clone() {
                                    span { class: "ui-ed__file",
                                        svg { view_box: "0 0 24 24", "aria-hidden": "true",
                                            path { d: "m15.5 7.5-6.4 6.4a1.8 1.8 0 0 0 2.5 2.5l6.7-6.7a3.5 3.5 0 0 0-5-5l-6.9 6.9a5.2 5.2 0 0 0 7.4 7.4l6-6" }
                                        }
                                        span { class: "ui-ed__file-name", title: "{name}", "{name}" }
                                        button {
                                            r#type: "button",
                                            aria_label: "Remove {name}",
                                            onclick: move |_| file.set(None),
                                            svg { view_box: "0 0 24 24", "aria-hidden": "true",
                                                path { d: "M7 7l10 10M17 7 7 17" }
                                            }
                                        }
                                    }
                                } else if let Some(offer) = attach_label.clone() {
                                    button {
                                        r#type: "button",
                                        class: "ui-ed__attach",
                                        onclick: {
                                            let picked = offer.clone();
                                            move |_| file.set(Some(picked.clone()))
                                        },
                                        svg { view_box: "0 0 24 24", "aria-hidden": "true",
                                            path { d: "M12 6v12M6 12h12" }
                                        }
                                        span { class: "ui-ed__file-name", title: "{offer}", "Attach {offer}" }
                                    }
                                }
                            }

                            div { class: "ui-ed__body",
                                if editing() {
                                    textarea {
                                        id: "{text_for_edit}",
                                        class: "ui-ed__edit",
                                        dir: "auto",
                                        value: "{body_for_textarea}",
                                        aria_label: "Email text",
                                        oninput: move |event| edited.set(Some(event.value())),
                                        onblur: move |_| editing.set(false),
                                        onkeydown: move |event| {
                                            let mods = event.modifiers();
                                            if event.key() == Key::Enter && (mods.meta() || mods.ctrl()) {
                                                event.prevent_default();
                                                send_key();
                                            }
                                            if event.key() == Key::Escape {
                                                editing.set(false);
                                            }
                                        },
                                    }
                                } else {
                                    p {
                                        key: "{current_tone}-{tone_key_value}",
                                        class: "ui-ed__text",
                                        dir: "auto",
                                        title: "Click to edit",
                                        onclick: {
                                            let text_id = text_for_edit.clone();
                                            let start_body = body_for_edit.clone();
                                            move |_| start_editing(edited, editing, start_body.clone(), &text_id)
                                        },
                                        "{body_text} "
                                        a {
                                            href: "{link_href}",
                                            onclick: move |event| event.stop_propagation(),
                                            "{link_label}"
                                        }
                                    }
                                }
                            }

                            div { class: "ui-ed__actions",
                                div { class: "ui-ed__tones", role: "group", aria_label: "Tone",
                                    for t in tones.iter() {
                                        {
                                            let tone_id = t.id.clone();
                                            let tone_label = t.label.clone();
                                            let pressed = current_tone == t.id && !edited_active;
                                            rsx! {
                                                button {
                                                    key: "{tone_id}",
                                                    r#type: "button",
                                                    aria_pressed: if pressed { "true" } else { "false" },
                                                    onclick: move |_| {
                                                        if tone() == tone_id && edited().is_none() {
                                                            return;
                                                        }
                                                        tone.set(tone_id.clone());
                                                        edited.set(None);
                                                        editing.set(false);
                                                        tone_key.set(tone_key() + 1);
                                                    },
                                                    "{tone_label}"
                                                }
                                            }
                                        }
                                    }
                                    button {
                                        r#type: "button",
                                        class: "ui-ed__edit-btn",
                                        aria_pressed: if editing() { "true" } else { "false" },
                                        aria_label: "Edit text",
                                        onclick: {
                                            let text_id = text_for_edit.clone();
                                            let start_body = body_for_edit.clone();
                                            move |_| start_editing(edited, editing, start_body.clone(), &text_id)
                                        },
                                        svg { view_box: "0 0 24 24", "aria-hidden": "true",
                                            path { d: "M14.5 5.5 18.5 9.5M5 19l1-4L15.5 5.5l4 4L10 19l-5 0Z" }
                                        }
                                    }
                                }
                                div { class: "ui-ed__buttons",
                                    button {
                                        r#type: "button",
                                        class: "ui-ed__secondary",
                                        disabled: sending,
                                        onclick: move |_| {
                                            editing.set(false);
                                            phase.set(Phase::Discarded);
                                            if let Some(handler) = on_discard {
                                                handler.call(());
                                            }
                                        },
                                        "Discard"
                                    }
                                    button {
                                        r#type: "button",
                                        class: "ui-ed__primary",
                                        disabled: sending || !has_recipient,
                                        onclick: move |_| send_click(),
                                        if sending {
                                            span { class: "ui-ed__spinner", aria_hidden: "true" }
                                            "Sending"
                                        } else {
                                            "Send email"
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "ui-ed__receipt", role: "status",
                            span {
                                class: "ui-ed__receipt-icon",
                                "data-tone": if phase() == Phase::Sent { "ok" } else { "muted" },
                                "aria-hidden": "true",
                                svg { view_box: "0 0 24 24", "aria-hidden": "true",
                                    if phase() == Phase::Sent {
                                        path { d: "m6 12.5 4 4 8-9" }
                                    } else {
                                        path { d: "M7 7l10 10M17 7 7 17" }
                                    }
                                }
                            }
                            span { class: "ui-ed__receipt-text",
                                if phase() == Phase::Sent {
                                    "Sent to "
                                    b { dir: "auto", "{recipient_name}" }
                                    "{file_suffix}"
                                } else {
                                    "Draft discarded"
                                }
                            }
                            button {
                                r#type: "button",
                                class: "ui-ed__undo",
                                onclick: move |_| phase.set(Phase::Draft),
                                "Undo"
                            }
                        }
                    }
                }
            }

            div { class: "ui-ed__foot",
                span { class: "ui-ed__agent", title: "{agent_text}", "{agent_text}" }
                span { class: "ui-ed__badge", "data-tone": badge_tone, key: "{badge_text}", "{badge_text}" }
                if open {
                    span { class: "ui-ed__hint", "⌘ Enter to send" }
                }
            }
        }
    }
}
