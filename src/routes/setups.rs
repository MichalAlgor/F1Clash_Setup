use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{delete, get};
use axum::{Form, Router};
use maud::html;
use serde::Deserialize;
use sqlx::PgPool;

use crate::models::part::{Part, PartCategory, Stats};
use crate::models::setup::{Setup, SetupWithStats};
use crate::templates;

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/setups", get(list).post(create))
        .route("/setups/new", get(new))
        .route("/setups/{id}", get(show).post(update))
        .route("/setups/{id}", delete(destroy))
}

async fn list(State(pool): State<PgPool>) -> impl IntoResponse {
    let setups = sqlx::query_as::<_, Setup>("SELECT * FROM setups ORDER BY name")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    let mut with_stats = Vec::new();
    for setup in setups {
        let stats = compute_setup_stats(&pool, &setup).await;
        with_stats.push(SetupWithStats { setup, stats });
    }

    templates::setups::list_page(&with_stats)
}

async fn new(State(pool): State<PgPool>) -> impl IntoResponse {
    let parts_by_category = load_parts_by_category(&pool).await;
    templates::setups::form_page(&parts_by_category, None)
}

#[derive(Deserialize)]
pub struct SetupForm {
    pub name: String,
    #[serde(rename = "Engine")]
    pub engine_id: i32,
    #[serde(rename = "Front Wing")]
    pub front_wing_id: i32,
    #[serde(rename = "Rear Wing")]
    pub rear_wing_id: i32,
    #[serde(rename = "Sidepod")]
    pub sidepod_id: i32,
    #[serde(rename = "Underbody")]
    pub underbody_id: i32,
    #[serde(rename = "Suspension")]
    pub suspension_id: i32,
    #[serde(rename = "Brakes")]
    pub brakes_id: i32,
}

async fn create(State(pool): State<PgPool>, Form(form): Form<SetupForm>) -> impl IntoResponse {
    sqlx::query(
        "INSERT INTO setups (name, engine_id, front_wing_id, rear_wing_id, sidepod_id, underbody_id, suspension_id, brakes_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(&form.name)
    .bind(form.engine_id)
    .bind(form.front_wing_id)
    .bind(form.rear_wing_id)
    .bind(form.sidepod_id)
    .bind(form.underbody_id)
    .bind(form.suspension_id)
    .bind(form.brakes_id)
    .execute(&pool)
    .await
    .unwrap();

    Redirect::to("/setups")
}

async fn show(State(pool): State<PgPool>, Path(id): Path<i32>) -> impl IntoResponse {
    let setup = sqlx::query_as::<_, Setup>("SELECT * FROM setups WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let stats = compute_setup_stats(&pool, &setup).await;
    let s = SetupWithStats { setup, stats };

    templates::layout::page(
        &s.setup.name,
        html! {
            h1 { (&s.setup.name) }
            figure {
                table {
                    thead {
                        tr { th { "Stat" } th { "Value" } }
                    }
                    tbody {
                        tr { td { "Speed" } td { (s.stats.speed) } }
                        tr { td { "Cornering" } td { (s.stats.cornering) } }
                        tr { td { "Power Unit" } td { (s.stats.power_unit) } }
                        tr { td { "Qualifying" } td { (s.stats.qualifying) } }
                        tr { td { "Pit Stop Time" } td { (format!("{:.2}s", s.stats.pit_stop_time)) } }
                        tr { td { strong { "Total Performance" } } td { strong { (s.stats.total_performance()) } } }
                    }
                }
            }
            a href="/setups" role="button" class="outline" { "← Back to setups" }
        },
    )
}

async fn update(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Form(form): Form<SetupForm>,
) -> impl IntoResponse {
    sqlx::query(
        "UPDATE setups SET name=$1, engine_id=$2, front_wing_id=$3, rear_wing_id=$4, sidepod_id=$5, underbody_id=$6, suspension_id=$7, brakes_id=$8
         WHERE id=$9",
    )
    .bind(&form.name)
    .bind(form.engine_id)
    .bind(form.front_wing_id)
    .bind(form.rear_wing_id)
    .bind(form.sidepod_id)
    .bind(form.underbody_id)
    .bind(form.suspension_id)
    .bind(form.brakes_id)
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();

    Redirect::to("/setups")
}

async fn destroy(State(pool): State<PgPool>, Path(id): Path<i32>) -> impl IntoResponse {
    sqlx::query("DELETE FROM setups WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    html! {}
}

async fn compute_setup_stats(pool: &PgPool, setup: &Setup) -> Stats {
    let part_ids = [
        setup.engine_id,
        setup.front_wing_id,
        setup.rear_wing_id,
        setup.sidepod_id,
        setup.underbody_id,
        setup.suspension_id,
        setup.brakes_id,
    ];

    let parts = sqlx::query_as::<_, Part>(
        "SELECT * FROM parts WHERE id = ANY($1)",
    )
    .bind(&part_ids[..])
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    parts.iter().fold(Stats::default(), |acc, p| acc.add(&p.stats()))
}

async fn load_parts_by_category(pool: &PgPool) -> Vec<(String, Vec<Part>)> {
    let parts = sqlx::query_as::<_, Part>("SELECT * FROM parts ORDER BY category, name")
        .fetch_all(pool)
        .await
        .unwrap_or_default();

    PartCategory::all()
        .iter()
        .map(|cat| {
            let cat_parts: Vec<Part> = parts.iter().filter(|p| p.category == *cat).cloned().collect();
            (cat.display_name().to_string(), cat_parts)
        })
        .collect()
}
