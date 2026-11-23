//! The runway surface: the A2UI path end to end.
//!
//! The page owns no layout of the form or the answer. It opens the surface
//! `apps/api` serves, holds it in a signal, sends each action and applies what
//! comes back. What renders is what the agent placed, through `rv2-ui`.

use dioxus::prelude::*;
use rv2_a2ui::{ActionRequest, SurfaceError, Surfaces};
use rv2_a2ui_dioxus::A2uiSurface;
use rv2_ui::{ErrorBanner, PageHeader};

use crate::views::SessionExpired;

const SURFACE: &str = "runway";

/// A refused message is the agent's defect; tell it, and tell the user the
/// surface did not change.
fn refused(mut error: Signal<Option<String>>, surface_id: String, reason: &SurfaceError) {
    error.set(Some(format!(
        "The assistant sent something this page cannot show: {reason}"
    )));
    let report = reason.to_message(&surface_id);
    spawn(async move {
        // Best effort: the user already sees the failure above.
        let _ = rv2_client::report_surface_error(report).await;
    });
}

#[component]
pub fn Runway() -> Element {
    let mut surfaces = use_signal(Surfaces::new);
    let mut error = use_signal(|| None::<String>);
    let mut expired = use_signal(|| false);
    let mut opening = use_resource(move || async move { rv2_client::open_surface(SURFACE).await });

    use_effect(move || {
        if let Some(Ok(messages)) = &*opening.read() {
            let mut fresh = Surfaces::new();
            match fresh.apply_all(messages) {
                Ok(()) => surfaces.set(fresh),
                Err((surface_id, reason)) => refused(error, surface_id, &reason),
            }
        }
    });

    let on_action = move |request: ActionRequest| {
        spawn(async move {
            error.set(None);
            match rv2_client::send_action(request).await {
                Ok(messages) => {
                    let applied = surfaces.write().apply_all(&messages);
                    if let Err((surface_id, reason)) = applied {
                        refused(error, surface_id, &reason);
                    }
                }
                Err(failure) if failure.is_unauthenticated() => expired.set(true),
                Err(failure) => error.set(Some(failure.to_string())),
            }
        });
    };

    rsx! {
        if expired() {
            SessionExpired {}
        }
        PageHeader {
            title: "Runway",
            subtitle: "An A2UI surface: apps/api places the components, this page renders them from the rv2 catalog.",
        }
        if let Some(message) = error() {
            ErrorBanner { message }
        }
        match &*opening.read() {
            Some(Err(failure)) if failure.is_unauthenticated() => rsx! { SessionExpired {} },
            Some(Err(failure)) => rsx! {
                ErrorBanner {
                    message: failure.to_string(),
                    onretry: move |_| opening.restart(),
                }
            },
            _ => rsx! { A2uiSurface { surfaces, surface_id: SURFACE, on_action } },
        }
    }
}
