//! Text that arrives piece by piece when it scrolls into view: each word, or each letter,
//! fades in out of a soft blur and rises a little, one after another. Screen readers get
//! the whole sentence at once; the moving pieces are hidden from them.
//!
//! Port losses: letters are split by `char`, not by grapheme cluster, so a family emoji or a
//! decomposed accent can split across two pieces. The visibility check (IntersectionObserver)
//! runs in `document::eval` and sets `data-shown` on the element.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/text-reveal.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextBy {
    #[default]
    Word,
    Char,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextTag {
    #[default]
    P,
    H1,
    H2,
    H3,
    Span,
}

/// One unit of the revealed text. `index` is the piece's place in the sequence and sets its delay.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Chunk {
    /// Whitespace between words. Stays plain text so lines wrap normally.
    Space(String),
    Piece {
        text: String,
        index: usize,
    },
    /// The letters of one word, kept together so the word does not break mid-letter.
    Group(Vec<(String, usize)>),
}

/// Splits text into whitespace and pieces, numbering the pieces in reading order.
pub fn chunks(text: &str, by: TextBy) -> Vec<Chunk> {
    let mut out = Vec::new();
    let mut index = 0;
    let mut run = String::new();
    let mut run_is_space: Option<bool> = None;
    let flush = |run: &mut String, is_space: bool, out: &mut Vec<Chunk>, index: &mut usize| {
        if run.is_empty() {
            return;
        }
        let text = std::mem::take(run);
        if is_space {
            out.push(Chunk::Space(text));
            return;
        }
        match by {
            TextBy::Word => {
                out.push(Chunk::Piece {
                    text,
                    index: *index,
                });
                *index += 1;
            }
            TextBy::Char => {
                let group = text
                    .chars()
                    .map(|c| {
                        let pair = (c.to_string(), *index);
                        *index += 1;
                        pair
                    })
                    .collect();
                out.push(Chunk::Group(group));
            }
        }
    };
    for ch in text.chars() {
        let is_space = ch.is_whitespace();
        if run_is_space.is_some_and(|prev| prev != is_space) {
            flush(
                &mut run,
                run_is_space.unwrap_or(false),
                &mut out,
                &mut index,
            );
        }
        run_is_space = Some(is_space);
        run.push(ch);
    }
    flush(
        &mut run,
        run_is_space.unwrap_or(false),
        &mut out,
        &mut index,
    );
    out
}

/// Inline custom properties for the timing. Durations are milliseconds.
pub fn timing_style(stagger: u32, delay: u32, duration: u32) -> String {
    format!(
        "--ui-trv-stagger: {stagger}ms; --ui-trv-delay: {delay}ms; --ui-trv-duration: {duration}ms;"
    )
}

fn visibility_script(id: &str, once: bool) -> String {
    format!(
        "const el = document.getElementById({id:?});
         if (!el) return;
         if (typeof IntersectionObserver === 'undefined') {{ el.dataset.shown = ''; return; }}
         const io = new IntersectionObserver((entries) => {{
           for (const e of entries) {{
             if (e.isIntersecting) {{
               el.dataset.shown = '';
               if ({once}) io.disconnect();
             }} else if (!{once}) {{
               delete el.dataset.shown;
             }}
           }}
         }}, {{ threshold: 0 }});
         io.observe(el);"
    )
}

#[component]
pub fn TextReveal(
    #[props(into)] text: String,
    #[props(default)] by: TextBy,
    #[props(default = 40)] stagger: u32,
    #[props(default = 0)] delay: u32,
    #[props(default = 700)] duration: u32,
    #[props(default)] as_tag: TextTag,
    #[props(default = true)] once: bool,
    #[props(default)] class: String,
) -> Element {
    let id = use_hook(|| format!("ui-trv-{}", next_id()));
    let observe_id = id.clone();
    use_effect(move || {
        document::eval(&visibility_script(&observe_id, once));
    });

    let nodes = chunks(&text, by);
    let style = timing_style(stagger, delay, duration);
    let class = class_list("ui-trv", &class);
    let body = rsx! {
        span { class: "ui-trv__sr", "{text}" }
        span { "aria-hidden": "true",
            for chunk in nodes {
                match chunk {
                    Chunk::Space(space) => rsx! { "{space}" },
                    Chunk::Piece { text, index } => rsx! {
                        span { class: "ui-trv__p", style: "--ui-trv-i: {index}", "{text}" }
                    },
                    Chunk::Group(letters) => rsx! {
                        span { class: "ui-trv__w",
                            for (letter, index) in letters {
                                span { class: "ui-trv__p", style: "--ui-trv-i: {index}", "{letter}" }
                            }
                        }
                    },
                }
            }
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        match as_tag {
            TextTag::P => rsx! { p { id: "{id}", class, style, {body} } },
            TextTag::H1 => rsx! { h1 { id: "{id}", class, style, {body} } },
            TextTag::H2 => rsx! { h2 { id: "{id}", class, style, {body} } },
            TextTag::H3 => rsx! { h3 { id: "{id}", class, style, {body} } },
            TextTag::Span => rsx! { span { id: "{id}", class, style, {body} } },
        }
    }
}
