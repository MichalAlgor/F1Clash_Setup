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
    #[serde(default)]
    additional_stat_name: Option<String>,
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
    /// Legacy field — ignored; data migrated to additional_stat_value by migration 005.
    #[serde(default)]
    #[allow(dead_code)]
    drs: i32,
    #[serde(default)]
    additional_stat_value: i32,
    #[serde(default)]
    additional_stat_details: HashMap<String, i32>,
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
    additional_stat_name: Option<String>,
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
    additional_stat_value: i32,
    additional_stat_details: serde_json::Value,
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
                r#"INSERT INTO part_catalog (name, season, category, series, rarity, sort_order, additional_stat_name)
                   VALUES ($1, $2, $3::part_category, $4, $5, $6, $7)
                   ON CONFLICT (name, season) DO UPDATE
                     SET category             = EXCLUDED.category,
                         series               = EXCLUDED.series,
                         rarity               = EXCLUDED.rarity,
                         sort_order           = EXCLUDED.sort_order,
                         additional_stat_name = EXCLUDED.additional_stat_name
                   RETURNING id"#,
            )
            .bind(&part.name)
            .bind(season)
            .bind(part.category.slug())
            .bind(part.series)
            .bind(&part.rarity)
            .bind(part.sort_order)
            .bind(&part.additional_stat_name)
            .fetch_one(pool)
            .await
            .unwrap_or_else(|e| panic!("Failed to upsert part '{}': {e}", part.name));

            for lvl in &part.levels {
                let details = serde_json::to_value(&lvl.additional_stat_details)
                    .unwrap_or(serde_json::json!({}));
                sqlx::query(
                    r#"INSERT INTO part_level_stats
                       (part_id, level, speed, cornering, power_unit, qualifying, pit_stop_time,
                        additional_stat_value, additional_stat_details)
                       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                       ON CONFLICT (part_id, level) DO UPDATE
                         SET speed                  = EXCLUDED.speed,
                             cornering              = EXCLUDED.cornering,
                             power_unit             = EXCLUDED.power_unit,
                             qualifying             = EXCLUDED.qualifying,
                             pit_stop_time          = EXCLUDED.pit_stop_time,
                             additional_stat_value  = EXCLUDED.additional_stat_value,
                             additional_stat_details = EXCLUDED.additional_stat_details"#,
                )
                .bind(part_id)
                .bind(lvl.level)
                .bind(lvl.speed)
                .bind(lvl.cornering)
                .bind(lvl.power_unit)
                .bind(lvl.qualifying)
                .bind(lvl.pit_stop_time)
                .bind(lvl.additional_stat_value)
                .bind(details)
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
        "SELECT id, name, season, category, series, rarity, sort_order, additional_stat_name
         FROM part_catalog
         ORDER BY season, sort_order",
    )
    .fetch_all(pool)
    .await
    .expect("Failed to load part catalog");

    let level_rows = sqlx::query_as::<_, LevelRow>(
        "SELECT part_id, level, speed, cornering, power_unit, qualifying, pit_stop_time,
                additional_stat_value, additional_stat_details
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
                .map(|l| {
                    let additional_stat_details: HashMap<String, i32> =
                        serde_json::from_value(l.additional_stat_details.clone())
                            .unwrap_or_default();
                    OwnedLevelStats {
                        level: l.level,
                        speed: l.speed,
                        cornering: l.cornering,
                        power_unit: l.power_unit,
                        qualifying: l.qualifying,
                        pit_stop_time: l.pit_stop_time,
                        additional_stat_value: l.additional_stat_value,
                        additional_stat_details,
                    }
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
                additional_stat_name: p.additional_stat_name,
                levels,
            }
        })
        .collect()
}

/// Load all season→category mappings from the DB.
pub async fn load_season_categories(pool: &PgPool) -> HashMap<String, Vec<PartCategory>> {
    #[derive(sqlx::FromRow)]
    struct Row {
        season: String,
        category: PartCategory,
    }

    let rows = sqlx::query_as::<_, Row>(
        "SELECT season, category FROM season_categories ORDER BY season",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut map: HashMap<String, Vec<PartCategory>> = HashMap::new();
    for row in rows {
        map.entry(row.season).or_default().push(row.category);
    }
    map
}
