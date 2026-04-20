use async_trait::async_trait;
use sqlx::PgPool;

use super::domain::{
    AnalyticsError, AnalyticsQuery, AnalyticsSink, CountryCount, DailyCount, DayCount, DeviceCount,
    EngagementStats, FeatureCount, FeatureEvent, FunnelStats, HourCount, PageEvent, PathCount,
    ReferrerCount, Summary,
};

#[derive(Clone)]
pub struct PostgresAnalytics {
    pool: PgPool,
}

impl PostgresAnalytics {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Delete events older than `days`. Safe to call concurrently.
    pub async fn prune(&self, days: u32) -> Result<u64, sqlx::Error> {
        let cutoff = days as i32;
        let r1 = sqlx::query(
            "DELETE FROM page_events WHERE ts < now() - ($1::int || ' days')::interval",
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?
        .rows_affected();

        let r2 = sqlx::query(
            "DELETE FROM feature_events WHERE ts < now() - ($1::int || ' days')::interval",
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(r1 + r2)
    }
}

#[async_trait]
impl AnalyticsSink for PostgresAnalytics {
    async fn record(&self, e: PageEvent) -> Result<(), AnalyticsError> {
        sqlx::query(
            r#"
            INSERT INTO page_events
                (path, canonical_path, kind, method, status, referrer,
                 device, country, response_ms, ts, session_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(&e.path)
        .bind(&e.canonical_path)
        .bind(&e.kind)
        .bind(&e.method)
        .bind(e.status as i16)
        .bind(&e.referrer)
        .bind(e.device.as_str())
        .bind(&e.country)
        .bind(e.response_ms as i32)
        .bind(e.ts)
        .bind(&e.session_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn record_feature(&self, e: FeatureEvent) -> Result<(), AnalyticsError> {
        sqlx::query(
            "INSERT INTO feature_events (session_id, event, properties) VALUES ($1, $2, $3)",
        )
        .bind(&e.session_id)
        .bind(&e.event)
        .bind(&e.properties)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[async_trait]
impl AnalyticsQuery for PostgresAnalytics {
    async fn visits_per_day(&self, days: u32) -> Result<Vec<DailyCount>, AnalyticsError> {
        let rows = sqlx::query_as::<_, (chrono::NaiveDate, i64)>(
            r#"
            SELECT (ts AT TIME ZONE 'UTC')::date AS day, COUNT(*)
            FROM page_events
            WHERE ts > now() - ($1::int || ' days')::interval
              AND device <> 'bot'
              AND kind = 'page'
            GROUP BY day
            ORDER BY day
            "#,
        )
        .bind(days as i32)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(day, count)| DailyCount { day, count })
            .collect())
    }

    async fn top_paths(&self, days: u32, limit: u32) -> Result<Vec<PathCount>, AnalyticsError> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT COALESCE(canonical_path, path) AS p, COUNT(*) AS c
            FROM page_events
            WHERE ts > now() - ($1::int || ' days')::interval
              AND device <> 'bot'
              AND kind = 'page'
            GROUP BY p
            ORDER BY c DESC
            LIMIT $2
            "#,
        )
        .bind(days as i32)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(path, count)| PathCount { path, count })
            .collect())
    }

    async fn top_referrers(
        &self,
        days: u32,
        limit: u32,
    ) -> Result<Vec<ReferrerCount>, AnalyticsError> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT referrer, COUNT(*) AS c
            FROM page_events
            WHERE ts > now() - ($1::int || ' days')::interval
              AND device <> 'bot'
              AND referrer IS NOT NULL
            GROUP BY referrer
            ORDER BY c DESC
            LIMIT $2
            "#,
        )
        .bind(days as i32)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(referrer, count)| ReferrerCount { referrer, count })
            .collect())
    }

    async fn top_countries(
        &self,
        days: u32,
        limit: u32,
    ) -> Result<Vec<CountryCount>, AnalyticsError> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT country, COUNT(*) AS c
            FROM page_events
            WHERE ts > now() - ($1::int || ' days')::interval
              AND device <> 'bot'
              AND country IS NOT NULL
            GROUP BY country
            ORDER BY c DESC
            LIMIT $2
            "#,
        )
        .bind(days as i32)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(country, count)| CountryCount { country, count })
            .collect())
    }

    async fn device_breakdown(&self, days: u32) -> Result<Vec<DeviceCount>, AnalyticsError> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT device, COUNT(*) AS c
            FROM page_events
            WHERE ts > now() - ($1::int || ' days')::interval
            GROUP BY device
            ORDER BY c DESC
            "#,
        )
        .bind(days as i32)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(device, count)| DeviceCount { device, count })
            .collect())
    }

    async fn summary(&self, days: u32) -> Result<Summary, AnalyticsError> {
        let row: (i64, i64, i64, Option<f64>, Option<f64>) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*)::bigint                                                  AS total,
                COUNT(DISTINCT session_id)::bigint                                AS visitors,
                COUNT(DISTINCT COALESCE(canonical_path, path))::bigint            AS paths,
                AVG(response_ms)::float8                                          AS avg_ms,
                (100.0 * SUM(CASE WHEN device = 'bot' THEN 1 ELSE 0 END)::float8
                    / NULLIF(COUNT(*), 0)::float8)                                AS bot_pct
            FROM page_events
            WHERE ts > now() - ($1::int || ' days')::interval
            "#,
        )
        .bind(days as i32)
        .fetch_one(&self.pool)
        .await?;

        Ok(Summary {
            total_events: row.0,
            unique_visitors: row.1,
            unique_paths: row.2,
            avg_response_ms: row.3.unwrap_or(0.0),
            bot_percentage: row.4.unwrap_or(0.0),
        })
    }

    // ── Stage 2: feature event counts ─────────────────────────────────────────

    async fn feature_counts(&self, days: u32) -> Result<Vec<FeatureCount>, AnalyticsError> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT event, COUNT(*) AS c
            FROM feature_events
            WHERE ts > now() - ($1::int || ' days')::interval
            GROUP BY event
            ORDER BY c DESC
            "#,
        )
        .bind(days as i32)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(event, count)| FeatureCount { event, count })
            .collect())
    }

    // ── Stage 3: session-level engagement ────────────────────────────────────

    async fn engagement(&self, days: u32) -> Result<EngagementStats, AnalyticsError> {
        // Bounce rate and average session depth from page views per session
        let depth_row: (Option<f64>, Option<f64>, Option<f64>) = sqlx::query_as(
            r#"
            SELECT
                AVG(page_count)::float8,
                (100.0 * COUNT(*) FILTER (WHERE page_count = 1)::float8
                    / NULLIF(COUNT(*), 0)::float8),
                NULL  -- placeholder for return visitor rate computed below
            FROM (
                SELECT session_id, COUNT(*) AS page_count
                FROM page_events
                WHERE ts > now() - ($1::int || ' days')::interval
                  AND device <> 'bot'
                  AND kind = 'page'
                  AND session_id IS NOT NULL
                GROUP BY session_id
            ) s
            "#,
        )
        .bind(days as i32)
        .fetch_one(&self.pool)
        .await?;

        // Return visitor rate: sessions seen on ≥2 distinct calendar days
        let return_row: (Option<f64>,) = sqlx::query_as(
            r#"
            SELECT
                (100.0 * COUNT(*) FILTER (WHERE day_count > 1)::float8
                    / NULLIF(COUNT(*), 0)::float8)
            FROM (
                SELECT session_id, COUNT(DISTINCT (ts AT TIME ZONE 'UTC')::date) AS day_count
                FROM page_events
                WHERE ts > now() - ($1::int || ' days')::interval
                  AND device <> 'bot'
                  AND session_id IS NOT NULL
                GROUP BY session_id
            ) s
            "#,
        )
        .bind(days as i32)
        .fetch_one(&self.pool)
        .await?;

        Ok(EngagementStats {
            bounce_rate: depth_row.1.unwrap_or(0.0),
            return_visitor_rate: return_row.0.unwrap_or(0.0),
            avg_session_depth: depth_row.0.unwrap_or(0.0),
        })
    }

    // ── Stage 4: time & day patterns ──────────────────────────────────────────

    async fn hourly_distribution(&self, days: u32) -> Result<Vec<HourCount>, AnalyticsError> {
        let rows = sqlx::query_as::<_, (i32, i64)>(
            r#"
            SELECT EXTRACT(HOUR FROM ts AT TIME ZONE 'UTC')::int AS hour, COUNT(*) AS c
            FROM page_events
            WHERE ts > now() - ($1::int || ' days')::interval
              AND device <> 'bot'
              AND kind = 'page'
            GROUP BY hour
            ORDER BY hour
            "#,
        )
        .bind(days as i32)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(hour, count)| HourCount { hour, count })
            .collect())
    }

    async fn day_of_week_distribution(&self, days: u32) -> Result<Vec<DayCount>, AnalyticsError> {
        let rows = sqlx::query_as::<_, (i32, i64)>(
            r#"
            SELECT EXTRACT(DOW FROM ts AT TIME ZONE 'UTC')::int AS dow, COUNT(*) AS c
            FROM page_events
            WHERE ts > now() - ($1::int || ' days')::interval
              AND device <> 'bot'
              AND kind = 'page'
            GROUP BY dow
            ORDER BY dow
            "#,
        )
        .bind(days as i32)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(dow, count)| DayCount { dow, count })
            .collect())
    }

    // ── Stage 5: funnel ───────────────────────────────────────────────────────

    async fn funnel(&self, days: u32) -> Result<FunnelStats, AnalyticsError> {
        let row: (i64, i64, i64, i64) = sqlx::query_as(
            r#"
            WITH
              inv AS (
                SELECT DISTINCT session_id FROM page_events
                WHERE COALESCE(canonical_path, path) = '/inventory'
                  AND kind = 'page'
                  AND device <> 'bot'
                  AND ts > now() - ($1::int || ' days')::interval
              ),
              opt AS (
                SELECT DISTINCT session_id FROM feature_events
                WHERE event = 'optimizer_run'
                  AND ts > now() - ($1::int || ' days')::interval
              ),
              sav AS (
                SELECT DISTINCT session_id FROM feature_events
                WHERE event = 'optimizer_save'
                  AND ts > now() - ($1::int || ' days')::interval
              ),
              shr AS (
                SELECT DISTINCT session_id FROM feature_events
                WHERE event = 'share_create'
                  AND ts > now() - ($1::int || ' days')::interval
              )
            SELECT
                (SELECT COUNT(*) FROM inv)::bigint,
                (SELECT COUNT(*) FROM opt)::bigint,
                (SELECT COUNT(*) FROM sav)::bigint,
                (SELECT COUNT(*) FROM shr)::bigint
            "#,
        )
        .bind(days as i32)
        .fetch_one(&self.pool)
        .await?;

        Ok(FunnelStats {
            visited_inventory: row.0,
            ran_optimizer: row.1,
            saved_setup: row.2,
            created_share: row.3,
        })
    }
}
