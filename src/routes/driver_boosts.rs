use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::Form;
use axum::Router;
use sqlx::PgPool;

use crate::drivers_data;
use crate::models::driver::DriverBoost;
use crate::templates;

pub fn router() -> Router<PgPool> {
    Router::new().route("/driver-boosts", get(show).post(save))
}

async fn show(State(pool): State<PgPool>) -> impl IntoResponse {
    let boosts = sqlx::query_as::<_, DriverBoost>(
        "SELECT * FROM driver_boosts ORDER BY driver_name",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    templates::driver_boosts::page(&boosts)
}

async fn save(
    State(pool): State<PgPool>,
    Form(form): Form<Vec<(String, String)>>,
) -> impl IntoResponse {
    sqlx::query("DELETE FROM driver_boosts")
        .execute(&pool)
        .await
        .unwrap();

    for (key, value) in &form {
        let Some(rest) = key.strip_prefix("boost:") else { continue };
        let Some((name, rarity_str)) = rest.rsplit_once(':') else { continue };
        let percentage: i32 = value.parse().unwrap_or(0);
        if percentage == 0 { continue; }
        if drivers_data::find_driver_by_db(name, rarity_str).is_none() { continue; }

        sqlx::query("INSERT INTO driver_boosts (driver_name, rarity, percentage) VALUES ($1, $2, $3)")
            .bind(name)
            .bind(rarity_str)
            .bind(percentage)
            .execute(&pool)
            .await
            .unwrap();
    }

    Redirect::to("/driver-boosts")
}
