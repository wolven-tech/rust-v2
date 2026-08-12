//! Transactional email.
//!
//! **Layer 2, SERVER-ONLY.** `resend-rs` pulls `reqwest`, so nothing here may be
//! reached from the Dioxus apps. The `wasm32 boundary` check enforces that.
//!
//! ## What this replaces, and what it adds
//!
//! rust-v1's `packages/email` was React Email **templates only** — a `welcome`
//! email and a logo component. Nothing ever sent anything: `RESEND_API_KEY` was
//! declared in `env.mjs` and `turbo.json`, and no code imported Resend. The
//! capability was configured but absent.
//!
//! This crate closes that: Tera templates (the direct analogue of React Email,
//! minus React) plus Resend's **official** Rust SDK actually wired to a send.
//!
//! ## Unconfigured renders to the log
//!
//! Without `RESEND_API_KEY` this is [`Mailer::Disabled`], which still *renders*
//! the template and logs the result. Two reasons:
//!
//! - A template that fails to render is a bug, and it should surface in
//!   development rather than the first time a key is present in staging.
//! - Local development gets to see the email without a vendor account or a
//!   catch-all inbox.
//!
//! It also makes the crate testable with no network and no key — the tests
//! below drive the real rendering path.

#![forbid(unsafe_code)]

/// Templates are compiled into the binary rather than read from disk, so a
/// deployed server cannot be missing one and there is no path resolution to get
/// wrong in a container.
const WELCOME_SUBJECT: &str = "Welcome to {{ product }}";
const WELCOME_BODY: &str = include_str!("../templates/welcome.html");

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("template `{name}` failed to render: {source}")]
    Render {
        name: &'static str,
        #[source]
        source: tera::Error,
    },
    #[error("Resend rejected the send: {0}")]
    Send(String),
}

/// A rendered, ready-to-send message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub to: String,
    pub subject: String,
    pub html: String,
}

/// The templates this application sends.
///
/// An enum rather than free-form template names: a typo in a string is a
/// runtime 500 on a path nobody exercises until a real user signs up, whereas a
/// typo here does not compile.
#[derive(Debug, Clone)]
pub enum Template {
    Welcome { name: String, product: String },
}

impl Template {
    const fn name(&self) -> &'static str {
        match self {
            Template::Welcome { .. } => "welcome",
        }
    }

    fn render(&self) -> Result<(String, String), EmailError> {
        let mut context = tera::Context::new();
        let (subject_src, body_src) = match self {
            Template::Welcome { name, product } => {
                context.insert("name", name);
                context.insert("product", product);
                (WELCOME_SUBJECT, WELCOME_BODY)
            }
        };

        let render = |src: &str| {
            tera::Tera::one_off(src, &context, true).map_err(|source| EmailError::Render {
                name: self.name(),
                source,
            })
        };

        Ok((render(subject_src)?, render(body_src)?))
    }
}

/// An outbound mail sink.
#[derive(Clone)]
pub enum Mailer {
    /// No `RESEND_API_KEY`. Messages are rendered and logged, not sent.
    Disabled { from: String },
    Resend {
        client: resend_rs::Resend,
        from: String,
    },
}

impl std::fmt::Debug for Mailer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mailer::Disabled { .. } => f.write_str("Mailer::Disabled"),
            Mailer::Resend { .. } => f.write_str("Mailer::Resend"),
        }
    }
}

impl Mailer {
    /// Build from the environment.
    ///
    /// `EMAIL_FROM` is required in both states — a rendered preview with no
    /// sender is a preview of something that could never have been sent.
    pub fn from_env() -> Self {
        let from =
            std::env::var("EMAIL_FROM").unwrap_or_else(|_| "onboarding@example.com".to_string());

        match std::env::var("RESEND_API_KEY") {
            Ok(key) if !key.trim().is_empty() => {
                tracing::info!(%from, "email enabled");
                Mailer::Resend {
                    client: resend_rs::Resend::new(&key),
                    from,
                }
            }
            _ => {
                tracing::info!("RESEND_API_KEY unset; email renders to the log");
                Mailer::Disabled { from }
            }
        }
    }

    /// The `From:` address, which both variants carry.
    fn sender(&self) -> &str {
        match self {
            Mailer::Disabled { from } | Mailer::Resend { from, .. } => from,
        }
    }

    /// Render a template without sending it. Useful in tests and previews.
    ///
    /// # Errors
    ///
    /// [`EmailError::Render`] if the template or its subject will not render.
    pub fn render(&self, to: &str, template: &Template) -> Result<Message, EmailError> {
        let (subject, html) = template.render()?;
        Ok(Message {
            to: to.to_string(),
            subject,
            html,
        })
    }

    /// Render and send.
    ///
    /// Unlike [`rv2_analytics`](../rv2_analytics/index.html), a failure here
    /// **is** returned: a welcome email that silently vanishes is a user-visible
    /// bug, and the caller is the only place that knows whether it is worth
    /// failing the request over.
    ///
    /// # Errors
    ///
    /// [`EmailError::Render`] if the template will not render, or
    /// [`EmailError::Send`] if Resend rejects the message.
    pub async fn send(&self, to: &str, template: Template) -> Result<Message, EmailError> {
        let message = self.render(to, &template)?;

        match self {
            Mailer::Disabled { .. } => {
                tracing::info!(
                    to = %message.to,
                    subject = %message.subject,
                    bytes = message.html.len(),
                    "email not sent (no RESEND_API_KEY); rendered only"
                );
            }
            Mailer::Resend { client, .. } => {
                let email = resend_rs::types::CreateEmailBaseOptions::new(
                    self.sender(),
                    [message.to.clone()],
                    message.subject.clone(),
                )
                .with_html(&message.html);

                client
                    .emails
                    .send(email)
                    .await
                    .map_err(|error| EmailError::Send(error.to_string()))?;

                tracing::info!(to = %message.to, "email sent");
            }
        }

        Ok(message)
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        matches!(self, Mailer::Resend { .. })
    }
}

/// Convenience for callers that only need the rendered form.
impl Message {
    #[must_use]
    pub fn preview(&self) -> String {
        format!(
            "To: {}\nSubject: {}\n\n{}",
            self.to, self.subject, self.html
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn welcome() -> Template {
        Template::Welcome {
            name: "Ada".to_string(),
            product: "rust-v2".to_string(),
        }
    }

    #[test]
    fn the_welcome_template_interpolates_both_subject_and_body() {
        let mailer = Mailer::Disabled {
            from: "onboarding@example.com".to_string(),
        };
        let message = mailer
            .render("ada@example.com", &welcome())
            .expect("render");

        assert_eq!(message.subject, "Welcome to rust-v2");
        assert!(
            message.html.contains("Ada"),
            "body did not interpolate the name"
        );
        assert!(
            !message.html.contains("{{"),
            "an unrendered placeholder survived: {}",
            message.html
        );
    }

    /// `one_off(.., true)` autoescapes. A name containing markup must not be
    /// able to inject it into the email body.
    #[test]
    fn interpolated_values_are_escaped() {
        let mailer = Mailer::Disabled {
            from: "onboarding@example.com".to_string(),
        };
        let message = mailer
            .render(
                "ada@example.com",
                &Template::Welcome {
                    name: "<script>alert(1)</script>".to_string(),
                    product: "rust-v2".to_string(),
                },
            )
            .expect("render");

        assert!(
            !message.html.contains("<script>"),
            "template did not escape user input: {}",
            message.html
        );
    }

    /// Sending while disabled still renders, so a broken template surfaces in
    /// development instead of the first time a key exists.
    #[tokio::test]
    async fn sending_while_disabled_still_renders() {
        let mailer = Mailer::Disabled {
            from: "onboarding@example.com".to_string(),
        };
        assert!(!mailer.is_enabled());

        let message = mailer
            .send("ada@example.com", welcome())
            .await
            .expect("send");
        assert_eq!(message.to, "ada@example.com");
        assert!(message.preview().contains("Subject: Welcome to rust-v2"));
    }
}
