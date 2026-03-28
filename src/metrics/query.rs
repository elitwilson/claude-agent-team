use anyhow::{Context, Result};
use rusqlite::Connection;

/// Summary of a single run's token usage across all agents.
#[derive(Debug, Clone, PartialEq)]
pub struct RunSummary {
    pub run_date: String,
    pub feature_slug: String,
    pub team: String,
    pub total_input: u64,
    pub total_output: u64,
    pub total_cache: u64,
    pub exit_code: i32,
}

/// Fetch all runs with aggregated token usage, ordered by started_at DESC.
pub fn fetch_runs(conn: &Connection) -> Result<Vec<RunSummary>> {
    let mut stmt = conn
        .prepare(
            "SELECT
                SUBSTR(r.started_at, 1, 10) AS run_date,
                r.feature_slug,
                r.team,
                COALESCE(SUM(a.input_tokens), 0) AS total_input,
                COALESCE(SUM(a.output_tokens), 0) AS total_output,
                COALESCE(SUM(a.cache_creation_tokens) + SUM(a.cache_read_tokens), 0) AS total_cache,
                r.agent_exit_code
            FROM runs r
            LEFT JOIN agent_usage a ON a.run_id = r.id
            GROUP BY r.id
            ORDER BY r.started_at DESC",
        )
        .context("Failed to prepare fetch_runs query")?;

    let rows = stmt
        .query_map([], |row| {
            Ok(RunSummary {
                run_date: row.get(0)?,
                feature_slug: row.get(1)?,
                team: row.get(2)?,
                total_input: row.get::<_, i64>(3)? as u64,
                total_output: row.get::<_, i64>(4)? as u64,
                total_cache: row.get::<_, i64>(5)? as u64,
                exit_code: row.get(6)?,
            })
        })
        .context("Failed to execute fetch_runs query")?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.context("Failed to read run summary row")?);
    }
    Ok(results)
}

#[cfg(test)]
mod tests;
