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

fn member(role: &str) -> AgentTeamMember {
    AgentTeamMember {
        role: role.into(),
        avatar: "avatars/agent.png".into(),
        tools: vec![AgentTool::Slack, AgentTool::Gmail],
    }
}

fn avatar(name: &str) -> TeamInviteAvatar {
    TeamInviteAvatar {
        src: "avatars/person.png".into(),
        name: Some(name.into()),
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
                AgentTeam { members: [member("SDR"), member("AE"), member("CSM")] }
            }
            Item { name: "CoverFlow",
                div { class: "gl-box",
                    CoverFlow {
                        items: vec![
                            CoverFlowItem::titled("1", "Night Drive"),
                            CoverFlowItem::titled("2", "Harbour"),
                            CoverFlowItem::titled("3", "Glass Hall"),
                        ],
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
                TeamInvite { avatars: [avatar("Ada"), avatar("Ben"), avatar("Cy")] }
            }
        }
    }
}
