use dioxus::prelude::*;
use rv2_affect::color_picker::ColorPicker;
use rv2_affect::date_picker::{DatePicker, Day};
use rv2_affect::number_input::NumberInput;
use rv2_affect::otp_input::{OtpInput, OtpSeparator, OtpStatus};
use rv2_affect::rating::{Rating, ThumbRating, ThumbsRating};
use rv2_affect::segmented_control::{SegmentKind, SegmentOption, SegmentedControl};
use rv2_affect::tag_input::TagInput;
use rv2_affect::text_area::TextArea;
use rv2_affect::text_field::{TextField, TextFieldSize};
use rv2_affect::time_picker::TimePicker;
use rv2_affect::toggle_group::{ToggleGroup, ToggleGroupItem};

use crate::{Item, Section, glyph};

const BOLD: &str = "M4 3h5a2.5 2.5 0 010 5H4zM4 8h6a2.5 2.5 0 010 5H4z";
const ITALIC: &str = "M6 3h6M4 13h6M9 3L7 13";
const UNDERLINE: &str = "M4 2v6a4 4 0 008 0V2M3 14h10";

fn segment(value: &str, label: &str) -> SegmentOption {
    SegmentOption {
        value: value.to_string(),
        label: Some(rsx! { "{label}" }),
        icon: None,
        count: None,
        badge: None,
        disabled: false,
        aria_label: None,
        controls: None,
    }
}

fn toggle(value: &str, label: &str) -> ToggleGroupItem {
    ToggleGroupItem {
        value: value.to_string(),
        label: Some(rsx! { "{label}" }),
        icon: None,
        aria_label: None,
        disabled: false,
    }
}

#[component]
pub fn Inputs() -> Element {
    let mut color = use_signal(|| "#0190fe".to_string());
    let mut day = use_signal(|| Day::new(2026, 3, 3).expect("valid date"));
    let mut time = use_signal(|| Some("09:30".to_string()));
    let mut count = use_signal(|| Some(4.0f64));
    let mut otp = use_signal(String::new);
    let mut stars = use_signal(|| 3.0f64);
    let mut thumb = use_signal(|| ThumbRating::Unrated);
    let mut segment_value = use_signal(|| "week".to_string());
    let mut tags = use_signal(|| vec!["rust".to_string(), "dioxus".to_string()]);
    let mut notes = use_signal(String::new);
    let mut name = use_signal(String::new);
    let mut marks = use_signal(|| vec!["bold".to_string()]);
    let mut resend = use_signal(|| 0usize);

    rsx! {
        Section { id: "inputs", title: "Inputs",
            Item { name: "ColorPicker",
                div { class: "gl-row",
                    ColorPicker {
                        label: "Brand color",
                        value: color(),
                        alpha: true,
                        on_value_change: move |v: String| color.set(v),
                    }
                }
                p { class: "gl-out", "Value: {color}" }
            }
            Item { name: "DatePicker",
                div { class: "gl-box",
                    DatePicker {
                        label: rsx! { "Start date" },
                        value: day(),
                        hint: rsx! { "Weekends are disabled" },
                        is_date_disabled: move |d: Day| d.weekday() == 0 || d.weekday() == 6,
                        onchange: move |d: Day| day.set(d),
                    }
                }
                p { class: "gl-out", "Picked weekday index: {day().weekday()}" }
            }
            Item { name: "NumberInput",
                div { class: "gl-box",
                    NumberInput {
                        label: rsx! { "Seats" },
                        name: "seats",
                        value: count(),
                        min: 0.0,
                        max: 99.0,
                        step: 1.0,
                        on_value_change: move |v: Option<f64>| count.set(v),
                    }
                }
                p { class: "gl-out", "Seats value: {count:?}" }
            }
            Item { name: "OtpInput",
                div { class: "gl-stack",
                    OtpInput {
                        length: 6,
                        separator: OtpSeparator::Middle,
                        value: otp(),
                        status: OtpStatus::Idle,
                        resend_in: 30u32,
                        on_change: move |v: String| otp.set(v),
                        on_resend: move |_| resend += 1,
                    }
                    p { class: "gl-out", "Code: {otp} (resend clicks: {resend})" }
                }
            }
            Item { name: "Rating",
                div { class: "gl-row",
                    Rating { value: stars(), name: "stars", on_value_change: move |v: f64| stars.set(v) }
                    Rating { read_only: true, value: 4.5 }
                }
                p { class: "gl-out", "Stars: {stars}" }
            }
            Item { name: "ThumbsRating",
                div { class: "gl-row",
                    ThumbsRating { value: thumb(), name: "answer", on_value_change: move |v: ThumbRating| thumb.set(v) }
                    ThumbsRating { read_only: true, value: ThumbRating::Down }
                }
                p { class: "gl-out", "Rating: {thumb:?}" }
            }
            Item { name: "SegmentedControl",
                div { class: "gl-stack",
                    SegmentedControl {
                        options: vec![segment("day", "Day"), segment("week", "Week"), segment("month", "Month")],
                        value: segment_value(),
                        aria_label: "Range",
                        on_change: move |v: String| segment_value.set(v),
                    }
                    SegmentedControl {
                        kind: SegmentKind::Tabs,
                        options: vec![segment("one", "Overview"), segment("two", "Activity")],
                        default_value: "one",
                        aria_label: "Views",
                    }
                }
                p { class: "gl-out", "Segment: {segment_value}" }
            }
            Item { name: "TagInput",
                div { class: "gl-box",
                    TagInput {
                        label: rsx! { "Skills" },
                        name: "skills",
                        value: tags(),
                        placeholder: "Add a skill",
                        max: 5usize,
                        on_change: move |v: Vec<String>| tags.set(v),
                    }
                }
                p { class: "gl-out", "Tags: {tags:?}" }
            }
            Item { name: "TextArea",
                div { class: "gl-box",
                    TextArea {
                        counter: true,
                        max_length: 280usize,
                        default_value: "Draft a short note".to_string(),
                        hint: rsx! { "Shown to the reviewer" },
                        on_input: move |e: FormEvent| notes.set(e.value()),
                    }
                }
                p { class: "gl-out", "Note length: {notes.read().chars().count()}" }
            }
            Item { name: "TextField",
                div { class: "gl-stack",
                    TextField {
                        label: rsx! { "Full name" },
                        placeholder: "Ada Lovelace",
                        clearable: true,
                        value: name(),
                        size: TextFieldSize::Lg,
                        on_input: move |e: FormEvent| name.set(e.value()),
                        on_clear: move |_| name.set(String::new()),
                    }
                    TextField {
                        label: rsx! { "Password" },
                        r#type: "password",
                        default_value: "secret".to_string(),
                    }
                    TextField {
                        label: rsx! { "Website" },
                        error: rsx! { "Enter a URL" },
                        prefix: rsx! { "https://" },
                        copyable: true,
                    }
                }
                p { class: "gl-out", "Name: {name}" }
            }
            Item { name: "TimePicker",
                div { class: "gl-box",
                    TimePicker {
                        label: rsx! { "Meeting time" },
                        value: time(),
                        step: 15u32,
                        min: "08:00",
                        max: "18:00",
                        on_change: move |v: Option<String>| time.set(v),
                    }
                }
                p { class: "gl-out", "Time: {time:?}" }
            }
            Item { name: "ToggleGroup",
                div { class: "gl-stack",
                    ToggleGroup {
                        items: vec![toggle("bold", "Bold"), toggle("italic", "Italic"), toggle("underline", "Underline")],
                        default_value: vec!["bold".to_string()],
                        on_change: move |v: Vec<String>| marks.set(v),
                    }
                    ToggleGroup {
                        items: vec![
                            ToggleGroupItem { value: "b".into(), label: None, icon: Some(glyph(BOLD)), aria_label: Some("Bold".into()), disabled: false },
                            ToggleGroupItem { value: "i".into(), label: None, icon: Some(glyph(ITALIC)), aria_label: Some("Italic".into()), disabled: false },
                            ToggleGroupItem { value: "u".into(), label: None, icon: Some(glyph(UNDERLINE)), aria_label: Some("Underline".into()), disabled: true },
                        ],
                        default_value: vec!["i".to_string()],
                    }
                }
                p { class: "gl-out", "Formats: {marks:?}" }
            }
        }
    }
}
