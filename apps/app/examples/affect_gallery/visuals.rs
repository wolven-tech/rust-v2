use dioxus::prelude::*;
use rv2_affect::blocks::logo_marquee::{LogoMarquee, MarqueeLabels, MarqueeLogo};
use rv2_affect::blocks::testimonial_wall::{Testimonial, TestimonialWall};
use rv2_affect::motion::grid_background::{GridBackground, GridDirection};
use rv2_affect::motion::number_ticker::NumberTicker;
use rv2_affect::motion::shimmer_loader::{Length, ShimmerText, Skeleton};
use rv2_affect::motion::text_reveal::{TextReveal, TextTag};

use crate::{Item, Section};

const CARD: &str =
    "box-sizing: border-box; padding: 16px; border-radius: 12px; box-shadow: 0 0 0 1px #0000001a;";

/// A brand mark drawn in currentColor, so the marquee's grayscale applies to it.
fn logo(name: &str) -> MarqueeLogo {
    let mark = match name {
        "Acme" => rsx! {
            svg { "viewBox": "0 0 24 24", "aria-hidden": "true", fill: "currentColor",
                path { d: "M12 3a9 9 0 1 0 0 18 9 9 0 1 0 0-18Zm0 4a5 5 0 1 1 0 10 5 5 0 1 1 0-10Z", fill_rule: "evenodd" }
            }
        },
        "Globex" => rsx! {
            svg { "viewBox": "0 0 24 24", "aria-hidden": "true", fill: "currentColor",
                rect { x: "3", y: "3", width: "18", height: "18", rx: "4" }
            }
        },
        "Initech" => rsx! {
            svg { "viewBox": "0 0 24 24", "aria-hidden": "true", fill: "currentColor",
                path { d: "M12 3 22 20H2Z" }
            }
        },
        "Umbrella" => rsx! {
            svg { "viewBox": "0 0 24 24", "aria-hidden": "true", fill: "currentColor",
                path { d: "M12 2 22 12 12 22 2 12Z" }
            }
        },
        "Hooli" => rsx! {
            svg { "viewBox": "0 0 24 24", "aria-hidden": "true", fill: "currentColor",
                circle { cx: "12", cy: "12", r: "9" }
            }
        },
        _ => rsx! {
            svg { "viewBox": "0 0 24 24", "aria-hidden": "true", fill: "currentColor",
                path { d: "M12 2 21 7v10l-9 5-9-5V7Z" }
            }
        },
    };
    MarqueeLogo {
        svg: Some(mark),
        ..MarqueeLogo::named(name)
    }
}

fn quote(text: &str, name: &str, role: &str, company: &str) -> Testimonial {
    let mut item = Testimonial::new(text, name);
    item.role = Some(role.into());
    item.company = Some(company.into());
    item
}

fn rated(mut item: Testimonial, stars: u8) -> Testimonial {
    item.rating = Some(stars);
    item
}

fn verified(mut item: Testimonial) -> Testimonial {
    item.verified = true;
    item
}

fn quotes() -> Vec<Testimonial> {
    vec![
        verified(rated(
            quote(
                "Shipped in a week. The defaults were already right.",
                "Ines Duarte",
                "Design Lead",
                "Northwind",
            ),
            5,
        )),
        rated(
            quote(
                "The gallery saved us a sprint of review.",
                "Ben Ortiz",
                "Product Manager",
                "Brightloop",
            ),
            4,
        ),
        quote(
            "Clear defaults, easy to theme.",
            "Cy Lee",
            "Frontend Engineer",
            "Harbor Studio",
        ),
        verified(quote(
            "Every component is one file with no dependencies.",
            "Noah Fischer",
            "Tech Lead",
            "Parcelwise",
        )),
        quote(
            "We rebuilt onboarding in two days and nothing looked out of place.",
            "Sofia Marino",
            "Product Manager",
            "Brightloop",
        ),
        rated(
            quote(
                "Finally a library that does not look like every other dashboard.",
                "Elif Demir",
                "Brand Designer",
                "Harbor Studio",
            ),
            5,
        ),
        verified(quote(
            "Accessible by default, without me having to argue for it.",
            "Marcus Lee",
            "Accessibility Lead",
            "Quillstack",
        )),
        rated(
            quote(
                "It feels like it was built by people who care about the last 5%.",
                "Camille Roux",
                "Design Engineer",
                "Sundial",
            ),
            5,
        ),
        quote(
            "The motion is the part nobody notices, which is the point.",
            "Lena Hartmann",
            "Design Lead",
            "Fieldnote",
        ),
    ]
}

#[component]
pub fn Visuals() -> Element {
    let mut revenue = use_signal(|| 48240.0f64);
    let mut replay = use_signal(|| 0usize);

    rsx! {
        Section { id: "visuals", title: "Motion and blocks",
            Item { name: "GridBackground",
                div { style: "position: relative; min-height: 300px; border-radius: 12px; overflow: hidden; box-shadow: 0 0 0 1px #0000001a;",
                    GridBackground { speed: 1.0, direction: GridDirection::Diagonal,
                        div { style: "display: grid; justify-items: center; align-content: center; gap: 10px; min-height: 300px; padding: 24px; text-align: center",
                            span { style: "font-size: 12px; padding: 2px 10px; border-radius: 999px; box-shadow: 0 0 0 1px #0000001f", "New" }
                            span { style: "font-size: 26px; font-weight: 600; letter-spacing: -0.02em", "Ship on a steady grid" }
                            span { style: "font-size: 14px; opacity: 0.6; max-width: 320px", "A quiet backdrop that keeps moving, so your page never feels idle." }
                        }
                    }
                }
            }
            Item { name: "NumberTicker",
                div { class: "gl-box", style: CARD,
                    div { style: "display: flex; justify-content: space-between; font-size: 13px",
                        span { "Revenue" }
                        span { style: "opacity: 0.6", "This month" }
                    }
                    div { style: "margin: 6px 0; font-size: 32px; font-weight: 600; line-height: 1.2",
                        NumberTicker { value: revenue(), prefix: "$", grouping: true }
                    }
                    div { style: "display: flex; align-items: center; gap: 8px; font-size: 12px",
                        span { style: "color: #1f9d55; background: #e7f7ee; padding: 2px 8px; border-radius: 999px", "↑ +12.4%" }
                        span { style: "opacity: 0.6", "vs last month" }
                    }
                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-top: 14px",
                        button { r#type: "button", onclick: move |_| revenue.set(revenue() - 1250.0), "−" }
                        button { r#type: "button", onclick: move |_| revenue.set(revenue() + 1250.0), "+" }
                    }
                }
            }
            Item { name: "ShimmerText",
                div { class: "gl-box", style: "display: grid; gap: 14px; box-sizing: border-box; padding: 16px; border-radius: 12px; box-shadow: 0 0 0 1px #0000001a;",
                    div { style: "display: flex; justify-content: space-between; align-items: center; font-size: 13px",
                        ShimmerText { duration: 3.0, "Generating summary…" }
                        button { r#type: "button", "Reload" }
                    }
                    div { style: "display: flex; gap: 12px; align-items: center",
                        Skeleton { width: Length::Px(36.0), circle: true }
                        div { style: "display: grid; gap: 6px; flex: 1",
                            Skeleton { width: Length::from("55%"), height: Length::Px(9.0) }
                            Skeleton { width: Length::from("35%"), height: Length::Px(9.0) }
                        }
                    }
                    Skeleton { width: Length::from("100%"), height: Length::Px(9.0) }
                    Skeleton { width: Length::from("72%"), height: Length::Px(9.0) }
                    div { style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px",
                        Skeleton { height: Length::Px(56.0), radius: Length::Px(8.0) }
                        Skeleton { height: Length::Px(56.0), radius: Length::Px(8.0) }
                        Skeleton { height: Length::Px(56.0), radius: Length::Px(8.0) }
                    }
                }
            }
            Item { name: "Shimmer Skeleton",
                div { class: "gl-stack",
                    Skeleton { width: Length::Px(120.0), height: Length::from("2em") }
                    Skeleton { width: Length::from("100%"), height: Length::from("1em") }
                }
            }
            Item { name: "TextReveal",
                div { class: "gl-box", key: "{replay}", style: "display: grid; justify-items: center; gap: 12px; padding: 16px 0; text-align: center",
                    div { style: "font-size: 26.67px; font-weight: 600; line-height: 1.15; letter-spacing: -0.02em",
                        TextReveal { text: "Ideas that land one word at a time.", as_tag: TextTag::H2 }
                    }
                    p { style: "margin: 0; font-size: 15px; opacity: 0.6; max-width: 380px",
                        "Each word fades in from a soft blur and settles into place, so a line reads as if someone is saying it."
                    }
                    button { r#type: "button", onclick: move |_| replay += 1, "Replay" }
                }
            }
            Item { name: "LogoMarquee",
                div { class: "gl-box",
                    p { style: "margin: 0 0 10px; text-align: center; font-size: 11px; letter-spacing: 0.08em; text-transform: uppercase; opacity: 0.6", "Trusted by teams at" }
                    LogoMarquee {
                        logos: vec![
                            logo("Acme"),
                            logo("Globex"),
                            logo("Initech"),
                            logo("Umbrella"),
                            logo("Hooli"),
                            logo("Soylent"),
                        ],
                        labels: MarqueeLabels::Beside,
                        label: "Customers",
                    }
                }
            }
            Item { name: "TestimonialWall", wide: true,
                div { style: "display: grid; gap: 4px; margin-bottom: 14px; text-align: center",
                    span { style: "font-size: 11px; letter-spacing: 0.08em; text-transform: uppercase; opacity: 0.6", "Testimonials" }
                    span { style: "font-size: 22px; font-weight: 600; letter-spacing: -0.01em", "Loved by early teams" }
                }
                TestimonialWall { items: quotes(), columns: 3 }
            }
        }
    }
}
