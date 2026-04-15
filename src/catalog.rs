use std::collections::HashMap;

use serde::Deserialize;
use sqlx::PgPool;

use crate::models::part::{OwnedLevelStats, OwnedPartDefinition, PartCategory};

/// Shape of one part entry in parts.json
#[derive(Debug, Deserialize)]
struct SeedPart {
    name: String,
    category: PartCategory,
    series: i32,
    rarity: String,
    sort_order: i32,
    levels: Vec<SeedLevel>,
}

#[derive(Debug, Deserialize)]
struct SeedLevel {
    level: i32,
    speed: i32,
    cornering: i32,
    power_unit: i32,
    qualifying: i32,
    pit_stop_time: f64,
    drs: i32,
}

/// Flat DB row for part_catalog — used by load_catalog.
#[derive(sqlx::FromRow)]
struct PartRow {
    id: i32,
    name: String,
    season: String,
    category: PartCategory,
    series: i32,
    rarity: String,
    sort_order: i32,
}

/// Flat DB row for part_level_stats — used by load_catalog.
#[derive(sqlx::FromRow)]
struct LevelRow {
    part_id: i32,
    level: i32,
    speed: i32,
    cornering: i32,
    power_unit: i32,
    qualifying: i32,
    pit_stop_time: f64,
    drs: i32,
}

/// Read parts.json and upsert all parts+levels into the DB.
/// Adds new rows and updates existing ones; never deletes.
pub async fn seed_catalog(pool: &PgPool) {
    let json = match std::fs::read_to_string("parts.json") {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("parts.json not found, skipping catalog seed: {e}");
            return;
        }
    };

    let seasons: HashMap<String, Vec<SeedPart>> = match serde_json::from_str(&json) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to parse parts.json: {e}");
            return;
        }
    };

    for (season, parts) in &seasons {
        for part in parts {
            let part_id: i32 = sqlx::query_scalar(
                r#"INSERT INTO part_catalog (name, season, category, series, rarity, sort_order)
                   VALUES ($1, $2, $3::part_category, $4, $5, $6)
                   ON CONFLICT (name, season) DO UPDATE
                     SET category   = EXCLUDED.category,
                         series     = EXCLUDED.series,
                         rarity     = EXCLUDED.rarity,
                         sort_order = EXCLUDED.sort_order
                   RETURNING id"#,
            )
            .bind(&part.name)
            .bind(season)
            .bind(part.category.slug())
            .bind(part.series)
            .bind(&part.rarity)
            .bind(part.sort_order)
            .fetch_one(pool)
            .await
            .unwrap_or_else(|e| panic!("Failed to upsert part '{}': {e}", part.name));

            for lvl in &part.levels {
                sqlx::query(
                    r#"INSERT INTO part_level_stats
                       (part_id, level, speed, cornering, power_unit, qualifying, pit_stop_time, drs)
                       VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                       ON CONFLICT (part_id, level) DO UPDATE
                         SET speed         = EXCLUDED.speed,
                             cornering     = EXCLUDED.cornering,
                             power_unit    = EXCLUDED.power_unit,
                             qualifying    = EXCLUDED.qualifying,
                             pit_stop_time = EXCLUDED.pit_stop_time,
                             drs           = EXCLUDED.drs"#,
                )
                .bind(part_id)
                .bind(lvl.level)
                .bind(lvl.speed)
                .bind(lvl.cornering)
                .bind(lvl.power_unit)
                .bind(lvl.qualifying)
                .bind(lvl.pit_stop_time)
                .bind(lvl.drs)
                .execute(pool)
                .await
                .unwrap_or_else(|e| {
                    panic!("Failed to upsert level {} for '{}': {e}", lvl.level, part.name)
                });
            }
        }
    }

    tracing::info!("Catalog seeded from parts.json");
}

/// Load the full catalog (all seasons) from the DB into memory.
pub async fn load_catalog(pool: &PgPool) -> Vec<OwnedPartDefinition> {
    let part_rows = sqlx::query_as::<_, PartRow>(
        "SELECT id, name, season, category, series, rarity, sort_order
         FROM part_catalog
         ORDER BY season, sort_order",
    )
    .fetch_all(pool)
    .await
    .expect("Failed to load part catalog");

    let level_rows = sqlx::query_as::<_, LevelRow>(
        "SELECT part_id, level, speed, cornering, power_unit, qualifying, pit_stop_time, drs
         FROM part_level_stats
         ORDER BY part_id, level",
    )
    .fetch_all(pool)
    .await
    .expect("Failed to load part level stats");

    part_rows
        .into_iter()
        .map(|p| {
            let levels = level_rows
                .iter()
                .filter(|l| l.part_id == p.id)
                .map(|l| OwnedLevelStats {
                    level: l.level,
                    speed: l.speed,
                    cornering: l.cornering,
                    power_unit: l.power_unit,
                    qualifying: l.qualifying,
                    pit_stop_time: l.pit_stop_time,
                    drs: l.drs,
                })
                .collect();

            OwnedPartDefinition {
                id: p.id,
                name: p.name,
                season: p.season,
                category: p.category,
                series: p.series,
                rarity: p.rarity,
                sort_order: p.sort_order,
                levels,
            }
        })
        .collect()
}
