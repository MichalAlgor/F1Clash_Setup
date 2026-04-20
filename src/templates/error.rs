use axum::http::StatusCode;
use maud::{Markup, html};

use super::layout;
use crate::auth::AuthStatus;

pub fn error_page(status: StatusCode, message: &str) -> Markup {
    let title = format!("{} Error", status.as_u16());
    let auth = AuthStatus {
        enabled: false,
        logged_in: false,
    };
    layout::page(
        &title,
        &auth,
        html! {
            hgroup {
                h1 { (status.as_u16()) }
                p { (message) }
            }
            a href="/" role="button" class="outline" { "← Back to home" }
        },
    )
}
