//! Views.
//!
//! State management follows §6.4: `use_signal` for local state,
//! `use_context_provider` at the shell for the session, `use_resource` for
//! server data. Deliberately **no** Redux-shaped global store and no
//! react-query equivalent.

use dioxus::prelude::*;
use rv2_api_types::{CreatePostRequest, PostView, SessionView};
use rv2_ui::{
    Button, Card, EmptyState, ErrorBanner, PageHeader, Skeleton, TextArea, TextField, Variant,
};
use uuid::Uuid;

use crate::post_card::PostCard;

use crate::routes::Route;

/// The session, provided by [`Shell`] to everything under it.
#[derive(Clone, Copy)]
pub struct Session(pub Signal<Option<SessionView>>);

/// The authenticated layout, and the auth gate.
#[component]
pub fn Shell() -> Element {
    let mut session = use_signal(|| None::<SessionView>);
    use_context_provider(|| Session(session));

    let navigator = use_navigator();
    let bootstrap = use_resource(move || async move { rv2_client::get_session().await });

    // The redirect that replaces `proxy.ts`. Only fires once the resource has
    // actually resolved — redirecting while it is still `None`-because-pending
    // would bounce every user to /login on every load.
    use_effect(move || {
        if let Some(Ok(result)) = &*bootstrap.read() {
            session.set(result.clone());
            if result.is_none() {
                navigator.replace(Route::Login {});
            }
        }
    });

    rsx! {
        div { class: "min-h-screen bg-slate-50 text-slate-900",
            nav { class: "border-b border-slate-200 bg-white",
                div { class: "mx-auto flex max-w-4xl items-center gap-6 px-6 py-3",
                    Link { class: "font-semibold", to: Route::Dashboard {}, "rust-v2" }
                    Link { class: "text-sm text-slate-600", to: Route::Posts {}, "Posts" }
                    div { class: "flex-1" }
                    if let Some(current) = session.read().as_ref() {
                        span { class: "text-sm text-slate-500", "{current.email}" }
                    }
                }
            }
            main { class: "mx-auto max-w-4xl px-6 py-8", Outlet::<Route> {} }
        }
    }
}

#[component]
pub fn Dashboard() -> Element {
    rsx! {
        PageHeader {
            title: "Dashboard",
            subtitle: "Everything here is folded from AllSource events. Nothing is fixture data.",
        }
        Card {
            p { class: "text-sm text-slate-600",
                "The vertical slice lives under Posts: a create appends "
                code { "content.post.created" }
                " to AllSource, and the list is served from the "
                code { "posts_v1" }
                " projection."
            }
        }
    }
}

/// The read half of the vertical slice: `GET /posts`, rendered.
#[component]
pub fn Posts() -> Element {
    let mut posts = use_resource(move || async move { rv2_client::list_posts().await });

    let mut title = use_signal(String::new);
    let mut content = use_signal(String::new);
    let mut submit_error = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    let create = move |_| async move {
        submitting.set(true);
        submit_error.set(None);
        let request = CreatePostRequest {
            title: title(),
            content: content(),
        };
        match rv2_client::create_post(&request).await {
            Ok(_) => {
                title.set(String::new());
                content.set(String::new());
                // Re-read through the API rather than pushing the response into
                // a local cache: the list is served by the projection worker,
                // and re-reading is what proves the fold actually happened.
                posts.restart();
            }
            Err(error) => submit_error.set(Some(error.to_string())),
        }
        submitting.set(false);
    };

    rsx! {
        PageHeader { title: "Posts", subtitle: "Served from the posts_v1 projection." }

        Card {
            form {
                class: "space-y-3",
                onsubmit: move |event| {
                    event.prevent_default();
                    create(())
                },
                TextField {
                    label: "Title",
                    value: title(),
                    placeholder: "On the Analytical Engine",
                    oninput: move |event: FormEvent| title.set(event.value()),
                }
                TextArea {
                    label: "Content",
                    value: content(),
                    rows: 4,
                    oninput: move |event: FormEvent| content.set(event.value()),
                }
                if let Some(message) = submit_error() {
                    ErrorBanner { message }
                }
                Button { r#type: "submit", disabled: submitting(), "Publish" }
            }
        }

        div { class: "mt-6 space-y-3",
            match &*posts.read() {
                None => rsx! { Skeleton { lines: 4 } },
                Some(Err(error)) => rsx! {
                    ErrorBanner {
                        message: error.to_string(),
                        onretry: move |_| posts.restart(),
                    }
                },
                Some(Ok(list)) if list.is_empty() => rsx! {
                    EmptyState {
                        title: "No posts yet",
                        description: "Publish one above to move an event through AllSource.",
                    }
                },
                Some(Ok(list)) => rsx! {
                    for post in list.clone() {
                        PostCard {
                            key: "{post.id}",
                            post: post.clone(),
                            actions: rsx! {
                                Link {
                                    to: Route::PostDetail { id: post.id },
                                    class: "text-xs text-slate-500 underline",
                                    "Open"
                                }
                            },
                        }
                    }
                },
            }
        }
    }
}

/// Fold-on-read for a single entity: `GET /posts/{id}`.
#[component]
pub fn PostDetail(id: Uuid) -> Element {
    let post = use_resource(move || async move { rv2_client::get_post(id).await });

    rsx! {
        match &*post.read() {
            None => rsx! { Skeleton { lines: 5 } },
            Some(Err(error)) => rsx! { ErrorBanner { message: error.to_string() } },
            Some(Ok(view)) => rsx! { PostBody { post: view.clone() } },
        }
    }
}

#[component]
fn PostBody(post: PostView) -> Element {
    rsx! {
        PageHeader { title: post.title.clone() }
        Card { p { class: "whitespace-pre-wrap text-sm text-slate-700", "{post.content}" } }
    }
}

/// Credential sign-in. Posts straight at better-auth's own route on `apps/api`,
/// which sets the HttpOnly session cookie (D17); the client never sees a token.
#[component]
pub fn Login() -> Element {
    rsx! {
        div { class: "mx-auto max-w-sm px-6 py-16",
            PageHeader { title: "Sign in" }
            Card {
                form {
                    class: "space-y-3",
                    method: "post",
                    action: "{rv2_client::api_base()}/auth/sign-in/email",
                    TextField {
                        label: "Email",
                        r#type: "email",
                        value: String::new(),
                        oninput: move |_: FormEvent| {},
                    }
                    TextField {
                        label: "Password",
                        r#type: "password",
                        value: String::new(),
                        oninput: move |_: FormEvent| {},
                    }
                    Button { r#type: "submit", variant: Variant::Primary, "Sign in" }
                }
            }
            p { class: "mt-4 text-xs text-slate-500",
                // §5.4 / SEAM in apps/api: OAuth is deliberately not wired in
                // this scaffold, and saying so beats a button that fails.
                "Google sign-in is not wired in this scaffold — see the OAuth SEAM note in apps/api."
            }
        }
    }
}
