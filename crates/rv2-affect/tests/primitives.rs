use dioxus::prelude::*;
use rv2_affect::avatar::{
    Avatar, AvatarGroup, AvatarItem, AvatarKind, AvatarStatus, hue_of, initials_of, mark_cells,
};
use rv2_affect::badge::{Badge, BadgeTone, format_count, group_thousands};
use rv2_affect::button::{Button, ButtonSize, ButtonVariant};
use rv2_affect::card::{Card, CardContent, CardElement, CardFooter, CardHeader};
use rv2_affect::checkbox::{Checkbox, CheckboxVariant};
use rv2_affect::empty_state::EmptyState;
use rv2_affect::form_field::{FormField, FormFieldControl};
use rv2_affect::icon_button::{IconButton, IconButtonVariant};
use rv2_affect::kbd::{Kbd, KbdPlatform, describe_key};
use rv2_affect::separator::{Separator, SeparatorOrientation};
use rv2_affect::skeleton::{Loading, Skeleton};
use rv2_affect::spinner::Spinner;
use rv2_affect::switch::{Switch, SwitchPlacement, SwitchSize};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn button_renders_variant_size_and_a_shimmer_copy_of_its_label() {
    let html = render(|| {
        rsx! { Button { variant: ButtonVariant::Destructive, size: ButtonSize::Lg, label: "Delete" } }
    });
    assert!(html.contains("class=\"ui-buttonc\""));
    assert!(html.contains("data-variant=\"destructive\""));
    assert!(html.contains("data-size=\"lg\""));
    assert!(html.contains("data-text=\"Delete\""));
    assert!(html.contains("type=\"button\""));
}

#[test]
fn loading_button_is_busy_and_swaps_its_icon_for_a_spinner() {
    let html = render(|| {
        rsx! { Button { loading: true, label: "Save", icon: rsx! { svg { class: "my-icon" } } } }
    });
    assert!(html.contains("aria-busy=\"true\""));
    assert!(html.contains("data-loading=\"true\""));
    assert!(html.contains("ui-buttonc__spinner"));
    assert!(!html.contains("my-icon"));
}

#[test]
fn icon_without_label_renders_square_and_passes_aria_label_through() {
    let html = render(|| {
        rsx! { Button { aria_label: "Close", icon: rsx! { svg {} } } }
    });
    assert!(html.contains("data-icon-only=\"true\""));
    assert!(html.contains("aria-label=\"Close\""));
    assert!(!html.contains("ui-buttonc__label"));
}

#[test]
fn spinner_with_a_label_is_a_status_and_without_one_is_hidden() {
    let announced = render(|| rsx! { Spinner { label: "Saving" } });
    assert!(announced.contains("class=\"ui-spnc\""));
    assert!(announced.contains("role=\"status\""));
    assert!(announced.contains("aria-label=\"Saving\""));
    assert!(!announced.contains("aria-hidden"));

    let silent = render(|| rsx! { Spinner { size: 20u32 } });
    assert!(silent.contains("aria-hidden=\"true\""));
    assert!(silent.contains("width=\"20\""));
}

#[test]
fn decorative_horizontal_separator_is_hidden_from_screen_readers() {
    let html = render(|| rsx! { Separator {} });
    assert!(html.contains("class=\"ui-sepc\""));
    assert!(html.contains("data-orientation=\"horizontal\""));
    assert!(html.contains("role=\"none\""));
    assert!(!html.contains("data-label"));
}

#[test]
fn undecorated_vertical_separator_is_announced_with_its_orientation() {
    let html = render(
        || rsx! { Separator { orientation: SeparatorOrientation::Vertical, decorative: false } },
    );
    assert!(html.contains("role=\"separator\""));
    assert!(html.contains("aria-orientation=\"vertical\""));
    assert!(html.contains("data-orientation=\"vertical\""));
}

#[test]
fn labelled_separator_draws_its_word_between_two_lines() {
    let html = render(|| rsx! { Separator { label: rsx! { "or" } } });
    assert!(html.contains("data-label=\"\""));
    assert!(html.contains("class=\"ui-sepc__label\""));
    assert_eq!(html.matches("ui-sepc__line").count(), 2);
}

#[test]
fn line_skeleton_keeps_a_one_line_box_around_its_bar() {
    let html = render(|| rsx! { Skeleton { line: true, width: "40%" } });
    assert!(html.contains("class=\"ui-sklc__line\""));
    assert!(html.contains("height: 1lh;"));
    assert!(html.contains("class=\"ui-sklc\""));
    assert!(html.contains("width: 40%;"));
}

#[test]
fn loading_wrapper_is_a_busy_status_named_for_its_area() {
    let html = render(|| rsx! { Loading { label: "Loading runs", Skeleton {} } });
    assert!(html.contains("role=\"status\""));
    assert!(html.contains("aria-busy=\"true\""));
    assert!(html.contains("aria-label=\"Loading runs\""));
    assert!(html.contains("class=\"ui-sklc\""));
}

#[test]
fn key_names_resolve_per_platform_and_letters_are_capitalised() {
    let mac = describe_key("mod", KbdPlatform::Mac);
    assert_eq!(mac.label, "⌘");
    assert_eq!(mac.spoken, "Command");
    assert!(mac.glyph);

    let other = describe_key("mod", KbdPlatform::Other);
    assert_eq!(other.label, "Ctrl");
    assert_eq!(other.spoken, "Control");
    assert!(!other.glyph);

    assert_eq!(describe_key("k", KbdPlatform::Other).label, "K");
    assert_eq!(describe_key("escape", KbdPlatform::Mac).label, "Esc");
    assert_eq!(describe_key(".", KbdPlatform::Mac).spoken, "Period");
}

#[test]
fn kbd_hides_its_glyphs_from_screen_readers_and_speaks_the_keys() {
    let html = render(|| rsx! { Kbd { keys: ["mod", "K"], platform: KbdPlatform::Mac } });
    assert!(html.contains("class=\"ui-kbdc\""));
    assert!(html.contains("data-os=\"mac\""));
    assert!(html.contains("data-glyph"));
    assert!(html.contains("class=\"ui-kbdc__sr\">Command K</span>"));
}

#[test]
fn kbd_without_keys_or_children_renders_nothing() {
    let html = render(|| rsx! { Kbd {} });
    assert!(!html.contains("ui-kbdc"));
}

#[test]
fn badge_count_is_capped_at_max_and_grouped_in_thousands() {
    assert_eq!(format_count(7, 99), "7");
    assert_eq!(format_count(1234, 9999), "1,234");
    assert_eq!(format_count(100, 99), "99+");
    assert_eq!(format_count(-3, 99), "0");
    assert_eq!(group_thousands(1_234_567), "1,234,567");
}

#[test]
fn live_badge_draws_a_pulsing_dot_before_its_text() {
    let html = render(|| rsx! { Badge { tone: BadgeTone::Info, live: true, "Running" } });
    assert!(html.contains("data-tone=\"info\""));
    assert!(html.contains("data-lead=\"dot\""));
    assert!(html.contains("data-live=\"\""));
    assert!(html.contains("ui-bdgc__label"));
}

#[test]
fn count_badge_is_round_and_shows_the_capped_number_without_a_label() {
    let html = render(|| rsx! { Badge { count: 120 } });
    assert!(html.contains("data-count=\"\""));
    assert!(html.contains(">99+<"));
    assert!(!html.contains("ui-bdgc__label"));
}

#[test]
fn badge_with_nothing_to_show_renders_nothing() {
    let html = render(|| rsx! { Badge {} });
    assert!(!html.contains("ui-bdgc"));
}

#[test]
fn initials_take_the_first_and_last_word_and_skip_suffixes() {
    assert_eq!(initials_of("Mara Lindqvist"), "ML");
    assert_eq!(initials_of("cher"), "C");
    assert_eq!(initials_of("Christopher Montgomery III"), "CM");
    assert_eq!(initials_of("(Ana) Ruiz"), "AR");
    assert_eq!(initials_of("🦊 Fox Team"), "🦊");
    assert_eq!(initials_of("   "), "");
}

#[test]
fn hue_and_agent_mark_are_stable_for_the_same_id_and_mirrored() {
    assert_eq!(hue_of("user_42"), hue_of("user_42"));
    assert!(hue_of("user_42") < 8);

    let cells = mark_cells("agent-7");
    assert_eq!(cells, mark_cells("agent-7"));
    assert!(cells.iter().filter(|on| **on).count() >= 5);
    assert_eq!(cells[0], cells[2]);
    assert_eq!(cells[3], cells[5]);
    assert_eq!(cells[6], cells[8]);
}

#[test]
fn person_avatar_shows_initials_on_a_hue_and_names_its_status() {
    let html = render(|| rsx! { Avatar { name: "Mara Lindqvist", status: AvatarStatus::Busy } });
    assert!(html.contains("data-kind=\"person\""));
    assert!(html.contains("ui-avtc__initials"));
    assert!(html.contains(">ML<"));
    assert!(html.contains("data-hue=\""));
    assert!(html.contains("aria-label=\"Mara Lindqvist, busy\""));
    assert!(html.contains("data-status=\"busy\""));
}

#[test]
fn agent_avatar_draws_nine_pixels_and_twinkles_while_working() {
    let html = render(
        || rsx! { Avatar { name: "Scout", kind: AvatarKind::Agent, working: true, color: "#0060e2" } },
    );
    assert!(html.contains("data-kind=\"agent\""));
    assert!(html.contains("data-working=\"\""));
    assert!(html.contains("--ui-avtc-mark: #0060e2;"));
    assert_eq!(html.matches("<i ").count(), 9);
    assert!(!html.contains("ui-avtc__initials"));
}

#[test]
fn avatar_group_stops_at_max_and_counts_the_rest() {
    let html = render(|| {
        let members: Vec<AvatarItem> = ["Ana Ruiz", "Ben Ortiz", "Cy Lee", "Dee Park", "Eve Moss"]
            .into_iter()
            .map(|name| AvatarItem {
                name: name.to_string(),
                ..Default::default()
            })
            .collect();
        rsx! { AvatarGroup { avatars: members, max: 3 } }
    });
    assert_eq!(html.matches("class=\"ui-avtc\"").count(), 3);
    assert!(html.contains(">+2<"));
    assert!(html.contains("aria-label=\"2 more\""));
}

#[test]
fn card_sets_its_footer_after_the_content() {
    let html = render(|| {
        rsx! {
            Card {
                element: CardElement::Section,
                interactive: true,
                selected: true,
                footer: rsx! { CardFooter { "Saved" } },
                CardHeader { title: rsx! { "Plan" }, description: rsx! { "Q3" } }
                CardContent { "Body" }
            }
        }
    });
    assert!(html.contains("<section"));
    assert!(html.contains("data-interactive=\"\""));
    assert!(html.contains("data-selected=\"\""));
    assert!(html.contains("class=\"ui-crdc__title\""));
    let body = html
        .find("class=\"ui-crdc__body\"")
        .expect("content rendered");
    let foot = html
        .find("class=\"ui-crdc__foot\"")
        .expect("footer rendered");
    assert!(body < foot);
}

#[test]
fn checkbox_card_variant_marks_indeterminate_and_links_its_hint() {
    let html = render(|| {
        rsx! {
            Checkbox { variant: CheckboxVariant::Card, indeterminate: true, label: rsx! { "Select all" }, description: rsx! { "Every row" } }
        }
    });
    assert!(html.contains("data-variant=\"card\""));
    assert!(html.contains("data-indeterminate=\"true\""));
    assert!(html.contains("aria-describedby=\"ui-checkbox-"));
    assert!(html.contains(">Every row<"));
}

#[test]
fn checkbox_error_marks_invalid_and_is_announced_politely() {
    let html =
        render(|| rsx! { Checkbox { label: rsx! { "Accept" }, error: rsx! { "Required" } } });
    assert!(html.contains("data-invalid=\"true\""));
    assert!(html.contains("aria-invalid=\"true\""));
    assert!(html.contains("class=\"ui-chkc__error\""));
    assert!(html.contains("aria-live=\"polite\""));
    assert!(html.contains(">Required<"));
}

#[test]
fn switch_is_a_role_switch_checkbox_with_its_description_linked() {
    let html = render(|| {
        rsx! {
            Switch {
                placement: SwitchPlacement::End,
                size: SwitchSize::Sm,
                label: rsx! { "Notify" },
                description: rsx! { "Email me" },
                disabled: true,
            }
        }
    });
    assert!(html.contains("role=\"switch\""));
    assert!(html.contains("type=\"checkbox\""));
    assert!(html.contains("data-placement=\"end\""));
    assert!(html.contains("data-size=\"sm\""));
    assert!(html.contains("data-disabled=\"true\""));
    assert!(html.contains("class=\"ui-swc__desc\""));
    assert!(html.contains("aria-describedby=\"ui-switch-"));
}

#[test]
fn form_field_links_label_hint_and_counter_to_the_control() {
    let html = render(|| {
        rsx! {
            FormField {
                label: rsx! { "Name" },
                required: true,
                hint: rsx! { "As on your passport" },
                counter: rsx! { "4/40" },
                html_for: "name-input",
                render: |control: FormFieldControl| {
                    let id = control.id.clone();
                    rsx! { input { id: "{id}", aria_describedby: control.described_by } }
                },
            }
        }
    });
    assert!(html.contains("for=\"name-input\""));
    assert!(html.contains("id=\"name-input\""));
    assert!(html.contains("class=\"ui-fldc__required\""));
    assert!(html.contains(">As on your passport<"));
    assert!(html.contains(">4/40<"));
    assert!(html.contains("aria-describedby=\""));
}

#[test]
fn form_field_error_takes_the_hint_place_and_marks_the_control_invalid() {
    let html = render(|| {
        rsx! {
            FormField {
                label: rsx! { "Email" },
                hint: rsx! { "Work address" },
                error: rsx! { "Enter an email" },
                optional: true,
                render: |control: FormFieldControl| {
                    let id = control.id.clone();
                    rsx! { input { id: "{id}", aria_invalid: control.invalid.then_some("true") } }
                },
            }
        }
    });
    assert!(html.contains("data-invalid=\"true\""));
    assert!(html.contains("aria-invalid=\"true\""));
    assert!(html.contains(">Enter an email<"));
    assert!(!html.contains("Work address"));
    assert!(html.contains("Optional"));
}

#[test]
fn empty_state_draws_the_default_file_tile_and_its_action() {
    let html = render(|| {
        rsx! {
            EmptyState {
                title: rsx! { "No schedules yet" },
                text: rsx! { "Add one to start" },
                action: rsx! { button { "Create" } },
            }
        }
    });
    assert!(html.contains("class=\"ui-estc__icon\""));
    assert!(html.contains("<path"));
    assert!(html.contains(">No schedules yet<"));
    assert!(html.contains("class=\"ui-estc__action\""));
}

#[test]
fn no_icon_drops_the_tile_and_bare_removes_the_frame() {
    let html =
        render(|| rsx! { EmptyState { title: rsx! { "Nothing" }, no_icon: true, bare: true } });
    assert!(!html.contains("ui-estc__icon"));
    assert!(html.contains("data-bare=\"true\""));
}

#[test]
fn icon_button_uses_its_label_as_name_and_tooltip() {
    let html = render(|| rsx! { IconButton { icon: rsx! { svg {} }, label: "Close" } });
    assert!(html.contains("aria-label=\"Close\""));
    assert!(html.contains("title=\"Close\""));
    assert!(html.contains("data-variant=\"ghost\""));
    assert!(html.contains("data-size=\"md\""));
    assert!(html.contains("type=\"button\""));
    assert!(!html.contains("aria-pressed"));
}

#[test]
fn toggled_icon_button_reports_pressed_and_an_empty_title_hides_the_tooltip() {
    let html = render(|| {
        rsx! { IconButton { icon: rsx! { svg {} }, label: "Bold", variant: IconButtonVariant::Secondary, pressed: true, title: "" } }
    });
    assert!(html.contains("aria-pressed=\"true\""));
    assert!(html.contains("data-variant=\"secondary\""));
    assert!(!html.contains("title="));
}

#[test]
fn button_accepts_focus_blur_and_key_handlers_and_still_renders() {
    let html = render(|| {
        rsx! {
            Button {
                label: "Save",
                onclick: move |_| {},
                onfocus: move |_| {},
                onblur: move |_| {},
                onkeydown: move |_| {},
            }
        }
    });
    assert!(html.contains("class=\"ui-buttonc\""));
    assert!(html.contains("Save"));
}

#[test]
fn icon_button_accepts_click_focus_blur_and_key_handlers_and_still_renders() {
    let html = render(|| {
        rsx! {
            IconButton {
                icon: rsx! { svg {} },
                label: "Bold",
                onclick: move |_| {},
                onfocus: move |_| {},
                onblur: move |_| {},
                onkeydown: move |_| {},
            }
        }
    });
    assert!(html.contains("aria-label=\"Bold\""));
    assert!(html.contains("class=\"ui-icbc__icon\""));
}

#[test]
fn card_accepts_click_and_key_handlers_on_its_root() {
    let html = render(|| {
        rsx! {
            Card {
                interactive: true,
                onclick: move |_| {},
                onkeydown: move |_| {},
                CardContent { "Body" }
            }
        }
    });
    assert!(html.contains("data-interactive=\"\""));
    assert!(html.contains("Body"));
}

#[test]
fn checkbox_accepts_change_input_focus_blur_and_key_handlers_and_still_renders() {
    let html = render(|| {
        rsx! {
            Checkbox {
                label: rsx! { "Agree" },
                onchange: move |_| {},
                oninput: move |_| {},
                onfocus: move |_| {},
                onblur: move |_| {},
                onkeydown: move |_| {},
            }
        }
    });
    assert!(html.contains("type=\"checkbox\""));
    assert!(html.contains("Agree"));
}

#[test]
fn switch_accepts_change_input_focus_blur_and_key_handlers_and_still_renders() {
    let html = render(|| {
        rsx! {
            Switch {
                label: rsx! { "Notify me" },
                onchange: move |_| {},
                oninput: move |_| {},
                onfocus: move |_| {},
                onblur: move |_| {},
                onkeydown: move |_| {},
            }
        }
    });
    assert!(html.contains("role=\"switch\""));
    assert!(html.contains("Notify me"));
}
