use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::Form;
use axum::Router;
use sqlx::PgPool;

use crate::data;
use crate::models::setup::Boost;
use crate::templates;

pub fn router() -> Router<PgPool> {
    Router::new().route("/boosts", get(show).post(save))
}

async fn show(State(pool): State<PgPool>) -> impl IntoResponse {
    let boosts = sqlx::query_as::<_, Boost>("SELECT * FROM boosts ORDER BY part_name")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    templates::boosts::page(&boosts)
}

async fn save(
    State(pool): State<PgPool>,
    Form(form): Form<Vec<(String, String)>>,
) -> impl IntoResponse {
    sqlx::query("DELETE FROM boosts")
        .execute(&pool)
        .await
        .unwrap();

    for (key, value) in &form {
        let Some(part_name) = key.strip_prefix("boost:") else {
            continue;
        };
        let percentage: i32 = value.parse().unwrap_or(0);
        if percentage == 0 {
            continue;
        }
        if data::find_part(part_name).is_none() {
            continue;
        }
        sqlx::query("INSERT INTO boosts (part_name, percentage) VALUES ($1, $2)")
            .bind(part_name)
            .bind(percentage)
            .execute(&pool)
            .await
            .unwrap();
    }

    Redirect::to("/boosts")
}
