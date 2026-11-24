use dioxus::prelude::*;
use rv2_affect::illustrations::agent_team::{AgentTeam, AgentTeamMember, AgentTool};
use rv2_affect::illustrations::cover_flow::{CoverFlow, CoverFlowItem};
use rv2_affect::illustrations::empty_calendar::EmptyCalendar;
use rv2_affect::illustrations::enrichment_orbit::EnrichmentOrbit;
use rv2_affect::illustrations::frost_cloud::{CloudColor, FrostCloud};
use rv2_affect::illustrations::ghosted_chat::GhostedChat;
use rv2_affect::illustrations::glow_cloud::GlowCloud;
use rv2_affect::illustrations::integration_beam::{BeamCenter, IntegrationBeam};
use rv2_affect::illustrations::integration_grid::IntegrationGrid;
use rv2_affect::illustrations::layer_stack::LayerStack;
use rv2_affect::illustrations::lead_sourcing::LeadSourcing;
use rv2_affect::illustrations::logo_cluster::LogoCluster;
use rv2_affect::illustrations::model_wheel::ModelWheel;
use rv2_affect::illustrations::region_dach::{Region, RegionDach};
use rv2_affect::illustrations::side_folder::SideFolder;
use rv2_affect::illustrations::tab_overload::TabOverload;
use rv2_affect::illustrations::team_invite::{TeamInvite, TeamInviteAvatar};

use crate::{Item, Section};

/// Initials on a flat colour, as an SVG data URI: the demo ships no image files.
fn initials(text: &str, fill: &str) -> String {
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'><rect width='64' height='64' fill='{fill}'/><text x='32' y='40' font-family='sans-serif' font-size='24' font-weight='600' text-anchor='middle' fill='white'>{text}</text></svg>"
    );
    let encoded = svg
        .replace('#', "%23")
        .replace('<', "%3C")
        .replace('>', "%3E")
        .replace(' ', "%20");
    format!("data:image/svg+xml,{encoded}")
}

fn member(role: &str, text: &str, fill: &str) -> AgentTeamMember {
    AgentTeamMember {
        role: role.into(),
        avatar: initials(text, fill),
        tools: vec![AgentTool::Slack, AgentTool::Gmail],
    }
}

fn avatar(name: &str, text: &str, fill: &str) -> TeamInviteAvatar {
    TeamInviteAvatar {
        src: initials(text, fill),
        name: Some(name.into()),
    }
}

fn sheet(title: &str, from: &str, to: &str) -> CoverFlowItem {
    CoverFlowItem {
        id: title.into(),
        title: Some(title.into()),
        subtitle: Some(format!("{from} → {to}")),
        image: None,
        alt: Some(title.into()),
        cover: Some(rsx! {
            div { style: "height: 100%; background: linear-gradient(160deg, {from}, {to})" }
        }),
    }
}

#[component]
pub fn Illustrations() -> Element {
    let mut region = use_signal(|| Region::Dach);
    let mut cover = use_signal(|| 0usize);
    let mut layer = use_signal(|| 0usize);

    rsx! {
        Section { id: "illustrations", title: "Illustrations",
            Item { name: "AgentTeam",
                AgentTeam { members: [member("Lead researcher", "LR", "#FF8A7A"), member("Outreach writer", "OW", "#7A8CFF"), member("Deal scorer", "DS", "#3CB98A")] }
            }
            Item { name: "CoverFlow", wide: true,
                div { style: "width: 100%;",
                    CoverFlow {
                        items: vec![
                            sheet("Dawn", "#FFF4E5", "#FFD3A8"),
                            sheet("Mint", "#E8FAF1", "#B3E6CC"),
                            sheet("Sky", "#EAF3FF", "#B5D4FA"),
                            sheet("Sage", "#F0F6EA", "#C9DDB5"),
                            sheet("Rosewater", "#FDF0F5", "#F4B6D2"),
                            sheet("Sand", "#FBF6E9", "#EFDFB0"),
                            sheet("Lilac", "#F3EFFD", "#CFC3F5"),
                            sheet("Ice", "#EEF9FC", "#B8E3EE"),
                            sheet("Peach", "#FFF2EC", "#FFC4B0"),
                        ],
                        item_width: 200.0,
                        item_height: 264.0,
                        on_index_change: move |i: usize| cover.set(i),
                        on_select: move |_| {},
                    }
                }
                p { class: "gl-out", "Slide index: {cover}" }
            }
            Item { name: "EmptyCalendar",
                div { class: "gl-box", EmptyCalendar {} }
            }
            Item { name: "EnrichmentOrbit",
                EnrichmentOrbit {}
            }
            Item { name: "FrostCloud",
                FrostCloud { color: CloudColor::Orange, follow_pointer: false }
            }
            Item { name: "GhostedChat",
                div { class: "gl-box", GhostedChat {} }
            }
            Item { name: "GlowCloud",
                GlowCloud {}
            }
            Item { name: "IntegrationBeam",
                IntegrationBeam { center: BeamCenter { name: "Acme".into(), logo: None } }
            }
            Item { name: "IntegrationGrid",
                IntegrationGrid {}
            }
            Item { name: "LayerStack",
                div { class: "gl-box",
                    LayerStack { on_change: move |i: usize| layer.set(i) }
                }
                p { class: "gl-out", "Layer index: {layer}" }
            }
            Item { name: "LeadSourcing",
                LeadSourcing {}
            }
            Item { name: "LogoCluster",
                LogoCluster {}
            }
            Item { name: "ModelWheel",
                ModelWheel {}
            }
            Item { name: "RegionDach",
                div { class: "gl-box",
                    RegionDach { region: region(), on_region_change: move |r: Region| region.set(r) }
                }
                p { class: "gl-out", "Region: {region:?}" }
            }
            Item { name: "SideFolder",
                SideFolder {}
            }
            Item { name: "TabOverload",
                TabOverload {}
            }
            Item { name: "TeamInvite",
                TeamInvite { avatars: [avatar("Ada", "A", "#F4A6B7"), avatar("Ben", "B", "#8FB8F5"), avatar("Cy", "C", "#9ED9B8")] }
            }
        }
    }
}
