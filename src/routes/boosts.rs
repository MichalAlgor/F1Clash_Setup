use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::Form;
use axum::Router;
use sqlx::PgPool;

use crate::data;
use crate::drivers_data;
use crate::models::driver::DriverBoost;
use crate::models::setup::Boost;
use crate::templates;

pub fn router() -> Router<PgPool> {
    Router::new().route("/boosts", get(show).post(save))
}

async fn show(State(pool): State<PgPool>) -> impl IntoResponse {
    let part_boosts = sqlx::query_as::<_, Boost>("SELECT * FROM boosts ORDER BY part_name")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    let driver_boosts = sqlx::query_as::<_, DriverBoost>(
        "SELECT * FROM driver_boosts ORDER BY driver_name",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    templates::boosts::page(&part_boosts, &driver_boosts)
}

/// Single save handler — form fields prefixed with `part:` or `driver:`
async fn save(
    State(pool): State<PgPool>,
    Form(form): Form<Vec<(String, String)>>,
) -> impl IntoResponse {
    // Clear both tables
    sqlx::query("DELETE FROM boosts").execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM driver_boosts").execute(&pool).await.unwrap();

    for (key, value) in &form {
        let percentage: i32 = value.parse().unwrap_or(0);
        if percentage == 0 {
            continue;
        }

        if let Some(part_name) = key.strip_prefix("part:") {
            if data::find_part(part_name).is_none() { continue; }
            sqlx::query("INSERT INTO boosts (part_name, percentage) VALUES ($1, $2)")
                .bind(part_name)
                .bind(percentage)
                .execute(&pool)
                .await
                .unwrap();
        } else if let Some(rest) = key.strip_prefix("driver:") {
            let Some((name, rarity_str)) = rest.rsplit_once(':') else { continue };
            if drivers_data::find_driver_by_db(name, rarity_str).is_none() { continue; }
            sqlx::query("INSERT INTO driver_boosts (driver_name, rarity, percentage) VALUES ($1, $2, $3)")
                .bind(name)
                .bind(rarity_str)
                .bind(percentage)
                .execute(&pool)
                .await
                .unwrap();
        }
    }

    Redirect::to("/boosts")
}
