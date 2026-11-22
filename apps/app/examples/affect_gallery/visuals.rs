use dioxus::prelude::*;
use rv2_affect::blocks::logo_marquee::{LogoMarquee, MarqueeLogo};
use rv2_affect::blocks::testimonial_wall::{Testimonial, TestimonialWall};
use rv2_affect::motion::grid_background::{GridBackground, GridDirection};
use rv2_affect::motion::number_ticker::NumberTicker;
use rv2_affect::motion::shimmer_loader::{Length, ShimmerText, Skeleton};
use rv2_affect::motion::text_reveal::{TextReveal, TextTag};

use crate::{Item, Section};

#[component]
pub fn Visuals() -> Element {
    let mut revenue = use_signal(|| 48240.0f64);
    let mut replay = use_signal(|| 0usize);

    rsx! {
        Section { id: "visuals", title: "Motion and blocks",
            Item { name: "GridBackground",
                div { class: "gl-canvas",
                    GridBackground { speed: 1.0, direction: GridDirection::Diagonal, span { "Content over a moving grid" } }
                }
            }
            Item { name: "NumberTicker",
                div { class: "gl-row",
                    NumberTicker { value: revenue(), prefix: "$", grouping: true }
                    button { r#type: "button", onclick: move |_| revenue.set(revenue() + 1250.0), "Add 1,250" }
                }
            }
            Item { name: "ShimmerText",
                ShimmerText { duration: 3.0, "Thinking about your request" }
            }
            Item { name: "Shimmer Skeleton",
                div { class: "gl-stack",
                    Skeleton { width: Length::Px(120.0), height: Length::from("2em") }
                    Skeleton { width: Length::from("100%"), height: Length::from("1em") }
                }
            }
            Item { name: "TextReveal",
                div { class: "gl-row", key: "{replay}",
                    TextReveal { text: "Ship it", as_tag: TextTag::H2 }
                    button { r#type: "button", onclick: move |_| replay += 1, "Replay" }
                }
            }
            Item { name: "LogoMarquee",
                LogoMarquee {
                    logos: vec![MarqueeLogo::named("Acme"), MarqueeLogo::named("Globex"), MarqueeLogo::named("Initech"), MarqueeLogo::named("Umbrella")],
                    label: "Customers",
                }
            }
            Item { name: "TestimonialWall",
                TestimonialWall {
                    items: vec![
                        {
                            let mut quote = Testimonial::new("Shipped in a week.", "Ada Lovelace");
                            quote.rating = Some(5);
                            quote.verified = true;
                            quote
                        },
                        {
                            let mut quote = Testimonial::new("The gallery saved us a sprint of review.", "Ben Ortiz");
                            quote.rating = Some(4);
                            quote
                        },
                        Testimonial::new("Clear defaults, easy to theme.", "Cy Lee"),
                    ],
                    columns: 3,
                }
            }
        }
    }
}
