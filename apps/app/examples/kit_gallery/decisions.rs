use dioxus::prelude::*;

use super::decisions_a::*;
use super::decisions_b::*;
use super::decisions_c::*;

#[component]
pub fn Decisions() -> Element {
    rsx! {
        super::Section { id: "decisions", title: "Decisions and chat",
            AccessGrantPromptDemo {}
            AgentQuestionCardDemo {}
            AnnotationSidebarDemo {}
            AssistantMemoryManagerDemo {}
            BackgroundJobsPopoverDemo {}
            ConsentRequestCardDemo {}
            DeliberationDigestDemo {}
            ExpandableWorkItemCardDemo {}
            GroundedReplyDemo {}
            InlineDraftingAssistantDemo {}
            InlineEditReviewDemo {}
            InterAgentMessageFeedDemo {}
            DecisionReviewQueueDemo {}
            PendingDecisionStackDemo {}
            SourceLinkedAnswerDemo {}
            TransientNoticeStackDemo {}
            WorkingTrailPanelDemo {}
            QuickActionPaletteDemo {}
            PromptInputBarDemo {}
        }
    }
}
