use dioxus::prelude::*;
use rv2_affect::badge::{Badge, BadgeTone};
use rv2_affect::color_picker::{ColorPicker, ColorPickerSize};
use rv2_affect::date_picker::{DatePicker, DatePickerSize, Day};
use rv2_affect::number_input::NumberInput;
use rv2_affect::otp_input::{OtpInput, OtpSeparator, OtpStatus};
use rv2_affect::rating::{Rating, RatingSize, ThumbRating, ThumbsRating};
use rv2_affect::segmented_control::{
    SegmentCount, SegmentKind, SegmentOption, SegmentSize, SegmentVariant, SegmentedControl,
};
use rv2_affect::tag_input::TagInput;
use rv2_affect::text_area::TextArea;
use rv2_affect::text_field::{TextField, TextFieldSize};
use rv2_affect::time_picker::TimePicker;
use rv2_affect::toggle_group::{
    ToggleGroup, ToggleGroupItem, ToggleKind, ToggleSize, ToggleVariant,
};

use crate::gallery::Ex;
use crate::{Item, Section, glyph};

const BOLD: &str = "M4 3h5a2.5 2.5 0 010 5H4zM4 8h6a2.5 2.5 0 010 5H4z";
const ITALIC: &str = "M6 3h6M4 13h6M9 3L7 13";
const UNDERLINE: &str = "M4 2v6a4 4 0 008 0V2M3 14h10";
const INBOX: &str = "M2 9l2-5h8l2 5v3H2zM2 9h3l1 2h4l1-2h3";
const SEND: &str = "M2 8l12-5-4 11-2-4z";
const DRAFT: &str = "M3 13l1-3 7-7 2 2-7 7zM10 4l2 2";

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

fn segment_count(value: &str, label: &str, count: f64) -> SegmentOption {
    SegmentOption {
        count: Some(SegmentCount::Number(count)),
        ..segment(value, label)
    }
}

fn segment_icon(value: &str, label: &str, path: &'static str) -> SegmentOption {
    SegmentOption {
        icon: Some(glyph(path)),
        ..segment(value, label)
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

fn icon_toggle(value: &str, label: &str, path: &'static str) -> ToggleGroupItem {
    ToggleGroupItem {
        value: value.to_string(),
        label: None,
        icon: Some(glyph(path)),
        aria_label: Some(label.to_string()),
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
    let mut runs = use_signal(|| vec!["needs".to_string()]);
    let mut group = use_signal(|| vec!["status".to_string()]);
    let mut resend = use_signal(|| 0usize);

    rsx! {
        Section { id: "inputs", title: "Inputs",
            Item { name: "ColorPicker",
                Ex { label: "Closed, with alpha",
                    ColorPicker {
                        label: "Brand color",
                        value: color(),
                        alpha: true,
                        on_value_change: move |v: String| color.set(v),
                    }
                }
                Ex { label: "Small, opaque",
                    ColorPicker { label: "Label color", value: "#7c3aed", size: ColorPickerSize::Sm }
                }
                Ex { label: "Open panel",
                    div { class: "gl-space",
                        ColorPicker { label: "Accent", value: "#16a34a", open: true }
                    }
                }
                p { class: "gl-out", "Value: {color}" }
            }
            Item { name: "DatePicker",
                Ex { label: "Weekends disabled",
                    div { class: "gl-box",
                        DatePicker {
                            label: rsx! { "Start date" },
                            value: day(),
                            hint: rsx! { "Weekends are disabled" },
                            is_date_disabled: move |d: Day| d.weekday() == 0 || d.weekday() == 6,
                            onchange: move |d: Day| day.set(d),
                        }
                    }
                }
                Ex { label: "Empty and optional, small",
                    div { class: "gl-box",
                        DatePicker {
                            label: rsx! { "Renewal reminder" },
                            optional: true,
                            size: DatePickerSize::Sm,
                            placeholder: "Pick a date",
                        }
                    }
                }
                Ex { label: "Open calendar",
                    div { class: "gl-space",
                        DatePicker {
                            label: rsx! { "Due" },
                            value: day(),
                            default_open: true,
                        }
                    }
                }
                p { class: "gl-out", "Picked weekday index: {day().weekday()}" }
            }
            Item { name: "NumberInput",
                Ex { label: "Bounded, with step",
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
                }
                Ex { label: "Step of five, empty",
                    div { class: "gl-box",
                        NumberInput {
                            label: rsx! { "Discount %" },
                            name: "discount",
                            value: None::<f64>,
                            min: 0.0,
                            max: 100.0,
                            step: 5.0,
                        }
                    }
                }
                p { class: "gl-out", "Seats value: {count:?}" }
            }
            Item { name: "OtpInput",
                Ex { label: "Idle, middle separator",
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
                    }
                }
                Ex { label: "Verifying",
                    OtpInput { length: 6, value: "482", status: OtpStatus::Verifying }
                }
                Ex { label: "Wrong code",
                    OtpInput { length: 6, value: "123456", status: OtpStatus::Error }
                }
                Ex { label: "Accepted",
                    OtpInput { length: 6, value: "482913", status: OtpStatus::Success }
                }
                p { class: "gl-out", "Code: {otp} (resend clicks: {resend})" }
            }
            Item { name: "Rating",
                Ex { label: "Interactive",
                    Rating { value: stars(), name: "stars", on_value_change: move |v: f64| stars.set(v) }
                }
                Ex { label: "Read only, 4.5 of 5",
                    Rating { read_only: true, value: 4.5 }
                }
                Ex { label: "Small and large",
                    Rating { read_only: true, value: 3.0, size: RatingSize::Sm }
                    Rating { read_only: true, value: 3.0, size: RatingSize::Lg }
                }
                p { class: "gl-out", "Stars: {stars}" }
            }
            Item { name: "ThumbsRating",
                Ex { label: "Interactive, unrated",
                    ThumbsRating { value: thumb(), name: "answer", on_value_change: move |v: ThumbRating| thumb.set(v) }
                }
                Ex { label: "Read only, marked down",
                    ThumbsRating { read_only: true, value: ThumbRating::Down }
                }
                Ex { label: "Large, marked up",
                    ThumbsRating { read_only: true, value: ThumbRating::Up, size: RatingSize::Lg }
                }
                p { class: "gl-out", "Rating: {thumb:?}" }
            }
            Item { name: "SegmentedControl",
                Ex { label: "Range, single select",
                    SegmentedControl {
                        options: vec![segment("day", "Day"), segment("week", "Week"), segment("month", "Month")],
                        value: segment_value(),
                        aria_label: "Range",
                        on_change: move |v: String| segment_value.set(v),
                    }
                }
                Ex { label: "Tray, with counts",
                    SegmentedControl {
                        variant: SegmentVariant::Tray,
                        options: vec![segment_count("open", "Open", 12.0), segment_count("won", "Won", 4.0), segment_count("lost", "Lost", 1.0)],
                        default_value: "open",
                        aria_label: "Pipeline stage",
                    }
                }
                Ex { label: "With icons",
                    SegmentedControl {
                        options: vec![segment_icon("draft", "Drafts", DRAFT), segment_icon("sent", "Sent", SEND), segment_icon("inbox", "Inbox", INBOX)],
                        default_value: "sent",
                        aria_label: "Mailbox",
                    }
                }
                Ex { label: "Underline tabs, with badge",
                    SegmentedControl {
                        kind: SegmentKind::Tabs,
                        variant: SegmentVariant::Underline,
                        options: vec![
                            segment("overview", "Overview"),
                            SegmentOption { badge: Some(rsx! { Badge { tone: BadgeTone::Warning, count: 3 } }), ..segment("activity", "Activity") },
                        ],
                        default_value: "overview",
                        aria_label: "Views",
                    }
                }
                Ex { label: "Sizes",
                    SegmentedControl { size: SegmentSize::Sm, options: vec![segment("a", "Small"), segment("b", "Two")], default_value: "a", aria_label: "Small" }
                    SegmentedControl { size: SegmentSize::Lg, options: vec![segment("a", "Large"), segment("b", "Two")], default_value: "a", aria_label: "Large" }
                }
                Ex { label: "Full width, one disabled",
                    div { class: "gl-box",
                        SegmentedControl {
                            full_width: true,
                            options: vec![
                                segment("sm", "Hourly"),
                                segment("md", "Daily"),
                                SegmentOption { disabled: true, ..segment("lg", "Yearly") },
                            ],
                            default_value: "md",
                            aria_label: "Billing unit",
                        }
                    }
                }
                p { class: "gl-out", "Segment: {segment_value}" }
            }
            Item { name: "TagInput",
                Ex { label: "Filled, up to five",
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
                }
                Ex { label: "Empty",
                    div { class: "gl-box",
                        TagInput {
                            label: rsx! { "Labels" },
                            name: "labels",
                            value: Vec::<String>::new(),
                            placeholder: "Type and press Enter",
                        }
                    }
                }
                p { class: "gl-out", "Tags: {tags:?}" }
            }
            Item { name: "TextArea",
                Ex { label: "Counter and hint",
                    div { class: "gl-box",
                        TextArea {
                            counter: true,
                            max_length: 280usize,
                            default_value: "Draft a short note".to_string(),
                            hint: rsx! { "Shown to the reviewer" },
                            on_input: move |e: FormEvent| notes.set(e.value()),
                        }
                    }
                }
                Ex { label: "Plain",
                    div { class: "gl-box",
                        TextArea { default_value: "Thanks for the quick turnaround on the renewal terms.".to_string() }
                    }
                }
                Ex { label: "At the limit",
                    div { class: "gl-box",
                        TextArea { counter: true, max_length: 12usize, default_value: "Ship on Friday".to_string() }
                    }
                }
                p { class: "gl-out", "Note length: {notes.read().chars().count()}" }
            }
            Item { name: "TextField",
                Ex { label: "Sizes",
                    div { class: "gl-box",
                        TextField { label: rsx! { "Small" }, placeholder: "Ada", size: TextFieldSize::Sm }
                        TextField { label: rsx! { "Medium" }, placeholder: "Ada", size: TextFieldSize::Md }
                        TextField { label: rsx! { "Large" }, placeholder: "Ada Lovelace", size: TextFieldSize::Lg }
                    }
                }
                Ex { label: "Clearable",
                    div { class: "gl-box",
                        TextField {
                            label: rsx! { "Full name" },
                            placeholder: "Ada Lovelace",
                            clearable: true,
                            value: name(),
                            on_input: move |e: FormEvent| name.set(e.value()),
                            on_clear: move |_| name.set(String::new()),
                        }
                    }
                }
                Ex { label: "Password",
                    div { class: "gl-box",
                        TextField {
                            label: rsx! { "Password" },
                            r#type: "password",
                            default_value: "secret".to_string(),
                        }
                    }
                }
                Ex { label: "Prefix, copyable, error",
                    div { class: "gl-box",
                        TextField {
                            label: rsx! { "Website" },
                            error: rsx! { "Enter a URL" },
                            prefix: rsx! { "https://" },
                            copyable: true,
                        }
                    }
                }
                Ex { label: "Disabled",
                    div { class: "gl-box",
                        TextField { label: rsx! { "Account" }, default_value: "Northwind Traders".to_string(), disabled: true }
                    }
                }
                p { class: "gl-out", "Name: {name}" }
            }
            Item { name: "TimePicker",
                Ex { label: "Fifteen-minute steps",
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
                }
                Ex { label: "Empty, small",
                    div { class: "gl-box",
                        TimePicker { label: rsx! { "Call back at" }, value: None::<String>, step: 30u32, size: rv2_affect::time_picker::TimeSize::Sm }
                    }
                }
                p { class: "gl-out", "Time: {time:?}" }
            }
            Item { name: "ToggleGroup",
                Ex { label: "Show runs: multi-select",
                    ToggleGroup {
                        items: vec![toggle("running", "Running"), toggle("needs", "Needs you"), toggle("done", "Done"), toggle("failed", "Failed")],
                        value: runs(),
                        on_change: move |v: Vec<String>| runs.set(v),
                    }
                }
                Ex { label: "Prompt formatting: icon only",
                    ToggleGroup {
                        items: vec![icon_toggle("b", "Bold", BOLD), icon_toggle("i", "Italic", ITALIC), icon_toggle("u", "Underline", UNDERLINE)],
                        default_value: vec!["i".to_string()],
                        on_change: move |v: Vec<String>| marks.set(v),
                    }
                }
                Ex { label: "Group by: single, press again for none",
                    ToggleGroup {
                        kind: ToggleKind::Single,
                        items: vec![toggle("status", "Status"), toggle("owner", "Owner"), toggle("due", "Due date")],
                        value: group(),
                        on_change: move |v: Vec<String>| group.set(v),
                    }
                }
                Ex { label: "Tray, sizes",
                    ToggleGroup {
                        variant: ToggleVariant::Tray,
                        size: ToggleSize::Sm,
                        items: vec![toggle("day", "Day"), toggle("week", "Week"), toggle("month", "Month")],
                        default_value: vec!["week".to_string()],
                    }
                    ToggleGroup {
                        variant: ToggleVariant::Tray,
                        size: ToggleSize::Lg,
                        items: vec![toggle("day", "Day"), toggle("week", "Week")],
                        default_value: vec!["day".to_string()],
                    }
                }
                Ex { label: "One item disabled",
                    ToggleGroup {
                        items: vec![
                            icon_toggle("b", "Bold", BOLD),
                            icon_toggle("i", "Italic", ITALIC),
                            ToggleGroupItem { disabled: true, ..icon_toggle("u", "Underline", UNDERLINE) },
                        ],
                        default_value: vec!["b".to_string()],
                    }
                }
                p { class: "gl-out", "Formats: {marks:?}. Runs: {runs:?}. Group: {group:?}" }
            }
        }
    }
}
