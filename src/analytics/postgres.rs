use async_trait::async_trait;
use sqlx::PgPool;

use super::domain::{
    AnalyticsError, AnalyticsQuery, AnalyticsSink, CountryCount, DailyCount, DeviceCount,
    PageEvent, PathCount, ReferrerCount, Summary,
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
        let res = sqlx::query(
            "DELETE FROM page_events WHERE ts < now() - ($1::int || ' days')::interval",
        )
        .bind(days as i32)
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected())
    }
}

#[async_trait]
impl AnalyticsSink for PostgresAnalytics {
    async fn record(&self, e: PageEvent) -> Result<(), AnalyticsError> {
        sqlx::query(
            r#"
            INSERT INTO page_events
                (path, method, status, referrer, device, country, response_ms, ts)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&e.path)
        .bind(&e.method)
        .bind(e.status as i16)
        .bind(&e.referrer)
        .bind(e.device.as_str())
        .bind(&e.country)
        .bind(e.response_ms as i32)
        .bind(e.ts)
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
            SELECT path, COUNT(*) AS c
            FROM page_events
            WHERE ts > now() - ($1::int || ' days')::interval
              AND device <> 'bot'
            GROUP BY path
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
        let row: (i64, i64, Option<f64>, Option<f64>) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*)::bigint                                                  AS total,
                COUNT(DISTINCT path)::bigint                                      AS paths,
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
            unique_paths: row.1,
            avg_response_ms: row.2.unwrap_or(0.0),
            bot_percentage: row.3.unwrap_or(0.0),
        })
    }
}
