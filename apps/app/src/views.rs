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

/// Credential sign-in.
///
/// `rv2_client::sign_in` posts JSON at better-auth's own route on `apps/api`,
/// which sets the HttpOnly session cookie (D17); the client never sees a token.
///
/// ## This was a form that could not submit
///
/// It used to be a **native HTML form POST** — `method="post"`,
/// `action="…/auth/sign-in/email"` — with `value: String::new()` hardcoded and
/// no-op `oninput` handlers. Clicking Sign in navigated the browser off the SPA
/// to the API origin and got a `400`: a native submit is form-encoded, and
/// better-auth answers JSON-or-400. Even had the encoding matched, the fields
/// held no state and `TextField` rendered no `name`, so the body was empty.
///
/// The API was never at fault, and the vertical slice passed throughout —
/// because it drives the API with reqwest and JSON and never touches this
/// screen. The whole dashboard was unreachable in a browser: `/posts` correctly
/// redirects here, and there was no way back out. Found by signing in with a
/// real browser, which is the only thing that would have found it.
#[component]
pub fn Login() -> Element {
    let navigator = use_navigator();

    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    let submit = move |_| async move {
        submitting.set(true);
        error.set(None);
        match rv2_client::sign_in(&email(), &password()).await {
            // Straight to the dashboard rather than `Shell`'s bootstrap being
            // re-run in place: `Shell` re-reads `GET /me` on mount, so the
            // navigation *is* the session refresh. `replace`, not `push`, so
            // Back does not return to a login screen the user has passed.
            Ok(()) => {
                password.set(String::new());
                navigator.replace(Route::Dashboard {});
            }
            // Deliberately not "invalid email or password": better-auth already
            // answers with a message that does not distinguish the two, and
            // re-wording it here would only risk saying something it did not.
            Err(error_response) => error.set(Some(error_response.to_string())),
        }
        submitting.set(false);
    };

    rsx! {
        div { class: "mx-auto max-w-sm px-6 py-16",
            PageHeader { title: "Sign in" }
            Card {
                form {
                    class: "space-y-3",
                    // `prevent_default` is what stops the browser doing its own
                    // navigation to `action`. Without it the handler runs and
                    // the page leaves anyway.
                    onsubmit: move |event| {
                        event.prevent_default();
                        submit(())
                    },
                    TextField {
                        label: "Email",
                        r#type: "email",
                        name: "email",
                        autocomplete: "email",
                        required: true,
                        value: email(),
                        oninput: move |event: FormEvent| email.set(event.value()),
                    }
                    TextField {
                        label: "Password",
                        r#type: "password",
                        name: "password",
                        autocomplete: "current-password",
                        required: true,
                        value: password(),
                        oninput: move |event: FormEvent| password.set(event.value()),
                    }
                    if let Some(message) = error() {
                        ErrorBanner { message }
                    }
                    Button {
                        r#type: "submit",
                        variant: Variant::Primary,
                        disabled: submitting(),
                        "Sign in"
                    }
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
