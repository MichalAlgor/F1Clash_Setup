use std::collections::HashMap;

use serde::Deserialize;
use sqlx::PgPool;

use crate::models::driver::{OwnedDriverDefinition, OwnedDriverLevelStats};
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

/// Parse a parts.json string and upsert all parts+levels into the DB.
/// Adds new rows and updates existing ones; never deletes.
pub async fn seed_parts_from_str(pool: &PgPool, json: &str) -> Result<(), anyhow::Error> {
    let seasons: HashMap<String, Vec<SeedPart>> = serde_json::from_str(json)?;

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
            .map_err(|e| anyhow::anyhow!("Failed to upsert part '{}': {e}", part.name))?;

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
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to upsert level {} for '{}': {e}",
                        lvl.level,
                        part.name
                    )
                })?;
            }
        }
    }

    tracing::info!("Catalog seeded from parts JSON");
    Ok(())
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

    if let Err(e) = seed_parts_from_str(pool, &json).await {
        tracing::error!("Failed to seed parts catalog: {e}");
    }
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

// ── Driver catalog ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SeedDriver {
    name: String,
    rarity: String,
    series: String,
    sort_order: i32,
    levels: Vec<SeedDriverLevel>,
}

#[derive(Debug, Deserialize)]
struct SeedDriverLevel {
    level: i32,
    overtaking: i32,
    defending: i32,
    qualifying: i32,
    race_start: i32,
    tyre_management: i32,
    #[serde(default)]
    cards_required: i32,
    #[serde(default)]
    coins_cost: i64,
    #[serde(default)]
    legacy_points: i32,
}

/// Parse a drivers.json string and upsert all drivers+levels into the DB.
/// Adds new rows and updates existing ones; never deletes.
pub async fn seed_drivers_from_str(pool: &PgPool, json: &str) -> Result<(), anyhow::Error> {
    let seasons: HashMap<String, Vec<SeedDriver>> = serde_json::from_str(json)?;
    for (season, drivers) in &seasons {
        seed_driver_season(pool, season, drivers).await;
    }
    tracing::info!("Driver catalog seeded from drivers JSON");
    Ok(())
}

/// Read drivers.json and upsert all drivers+levels into the DB.
/// Falls back to seeding from built-in static data (season "2025") when
/// drivers.json is absent and the table is empty.
pub async fn seed_drivers_catalog(pool: &PgPool) {
    if let Ok(json) = std::fs::read_to_string("drivers.json") {
        if let Err(e) = seed_drivers_from_str(pool, &json).await {
            tracing::error!("Failed to seed drivers catalog: {e}");
        }
        return;
    }

    tracing::warn!("drivers.json not found — driver catalog not updated");
}

async fn seed_driver_season(pool: &PgPool, season: &str, drivers: &[SeedDriver]) {
    for driver in drivers {
        let driver_id: i32 = sqlx::query_scalar(
            r#"INSERT INTO driver_catalog (name, season, rarity, series, sort_order)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (name, rarity, season) DO UPDATE
                 SET series     = EXCLUDED.series,
                     sort_order = EXCLUDED.sort_order
               RETURNING id"#,
        )
        .bind(&driver.name)
        .bind(season)
        .bind(&driver.rarity)
        .bind(&driver.series)
        .bind(driver.sort_order)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|e| panic!("Failed to upsert driver '{}': {e}", driver.name));

        for lvl in &driver.levels {
            sqlx::query(
                r#"INSERT INTO driver_level_stats
                   (driver_id, level, overtaking, defending, qualifying, race_start, tyre_management,
                    cards_required, coins_cost, legacy_points)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                   ON CONFLICT (driver_id, level) DO UPDATE
                     SET overtaking      = EXCLUDED.overtaking,
                         defending       = EXCLUDED.defending,
                         qualifying      = EXCLUDED.qualifying,
                         race_start      = EXCLUDED.race_start,
                         tyre_management = EXCLUDED.tyre_management,
                         cards_required  = EXCLUDED.cards_required,
                         coins_cost      = EXCLUDED.coins_cost,
                         legacy_points   = EXCLUDED.legacy_points"#,
            )
            .bind(driver_id)
            .bind(lvl.level)
            .bind(lvl.overtaking)
            .bind(lvl.defending)
            .bind(lvl.qualifying)
            .bind(lvl.race_start)
            .bind(lvl.tyre_management)
            .bind(lvl.cards_required)
            .bind(lvl.coins_cost)
            .bind(lvl.legacy_points)
            .execute(pool)
            .await
            .unwrap_or_else(|e| {
                panic!("Failed to upsert level {} for driver '{}': {e}", lvl.level, driver.name)
            });
        }
    }
}

/// Load the full driver catalog (all seasons) from the DB into memory.
pub async fn load_drivers_catalog(pool: &PgPool) -> Vec<OwnedDriverDefinition> {
    #[derive(sqlx::FromRow)]
    struct DriverRow {
        id: i32,
        name: String,
        season: String,
        rarity: String,
        series: String,
        sort_order: i32,
    }

    #[derive(sqlx::FromRow)]
    struct DriverLevelRow {
        driver_id: i32,
        level: i32,
        overtaking: i32,
        defending: i32,
        qualifying: i32,
        race_start: i32,
        tyre_management: i32,
        cards_required: i32,
        coins_cost: i64,
        legacy_points: i32,
    }

    let driver_rows = sqlx::query_as::<_, DriverRow>(
        "SELECT id, name, season, rarity, series, sort_order
         FROM driver_catalog
         ORDER BY season, sort_order",
    )
    .fetch_all(pool)
    .await
    .expect("Failed to load driver catalog");

    let level_rows = sqlx::query_as::<_, DriverLevelRow>(
        "SELECT driver_id, level, overtaking, defending, qualifying, race_start, tyre_management,
                cards_required, coins_cost, legacy_points
         FROM driver_level_stats
         ORDER BY driver_id, level",
    )
    .fetch_all(pool)
    .await
    .expect("Failed to load driver level stats");

    driver_rows
        .into_iter()
        .map(|d| {
            let levels = level_rows
                .iter()
                .filter(|l| l.driver_id == d.id)
                .map(|l| OwnedDriverLevelStats {
                    level: l.level,
                    overtaking: l.overtaking,
                    defending: l.defending,
                    qualifying: l.qualifying,
                    race_start: l.race_start,
                    tyre_management: l.tyre_management,
                    cards_required: l.cards_required,
                    coins_cost: l.coins_cost,
                    legacy_points: l.legacy_points,
                })
                .collect();
            OwnedDriverDefinition {
                id: d.id,
                name: d.name,
                season: d.season,
                rarity: d.rarity,
                series: d.series,
                sort_order: d.sort_order,
                levels,
            }
        })
        .collect()
}

// ── Season categories ─────────────────────────────────────────────────────────

/// Load all season→category mappings from the DB.
pub async fn load_season_categories(pool: &PgPool) -> HashMap<String, Vec<PartCategory>> {
    #[derive(sqlx::FromRow)]
    struct Row {
        season: String,
        category: PartCategory,
    }

    let rows =
        sqlx::query_as::<_, Row>("SELECT season, category FROM season_categories ORDER BY season")
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    let mut map: HashMap<String, Vec<PartCategory>> = HashMap::new();
    for row in rows {
        map.entry(row.season).or_default().push(row.category);
    }
    for cats in map.values_mut() {
        cats.sort();
    }
    map
}
