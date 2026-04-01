use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::Connection;

/// Initialize the SQLite database with the required schema.
pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS runs (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            feature_slug        TEXT NOT NULL,
            team                TEXT NOT NULL,
            project             TEXT NOT NULL,
            started_at          TEXT NOT NULL,
            completed_at        TEXT NOT NULL,
            agent_exit_code     INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS agent_usage (
            id                          INTEGER PRIMARY KEY AUTOINCREMENT,
            run_id                      INTEGER NOT NULL REFERENCES runs(id),
            agent_role                  TEXT NOT NULL,
            input_tokens                INTEGER NOT NULL DEFAULT 0,
            output_tokens               INTEGER NOT NULL DEFAULT 0,
            cache_creation_tokens       INTEGER NOT NULL DEFAULT 0,
            cache_read_tokens           INTEGER NOT NULL DEFAULT 0
        );",
    )
    .context("Failed to initialize database schema")?;
    Ok(())
}

/// Insert a run record and return the run ID.
pub fn insert_run(
    conn: &Connection,
    feature_slug: &str,
    team: &str,
    project: &str,
    started_at: &str,
    completed_at: &str,
    exit_code: i32,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO runs (feature_slug, team, project, started_at, completed_at, agent_exit_code)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            feature_slug,
            team,
            project,
            started_at,
            completed_at,
            exit_code
        ],
    )
    .context("Failed to insert run")?;
    Ok(conn.last_insert_rowid())
}

/// Insert an agent usage record.
pub fn insert_agent_usage(
    conn: &Connection,
    run_id: i64,
    role: &str,
    input_tokens: u64,
    output_tokens: u64,
    cache_creation_tokens: u64,
    cache_read_tokens: u64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO agent_usage (run_id, agent_role, input_tokens, output_tokens, cache_creation_tokens, cache_read_tokens)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![run_id, role, input_tokens as i64, output_tokens as i64, cache_creation_tokens as i64, cache_read_tokens as i64],
    )
    .context("Failed to insert agent usage")?;
    Ok(())
}

/// The most recent completed run for a spec in a project.
pub struct LastRun {
    pub team: String,
    pub completed_at: DateTime<Utc>,
}

/// Return the most recent completed run per `feature_slug` for the given project.
///
/// Rows with unparseable `completed_at` values are skipped silently.
pub fn last_run_for_project(conn: &Connection, project: &str) -> Result<HashMap<String, LastRun>> {
    let mut stmt = conn.prepare(
        "SELECT feature_slug, team, MAX(completed_at) AS completed_at
         FROM runs
         WHERE project = ?1
         GROUP BY feature_slug",
    )?;

    let mut map = HashMap::new();
    let mut rows = stmt.query([project])?;
    while let Some(row) = rows.next()? {
        let slug: String = row.get(0)?;
        let team: String = row.get(1)?;
        let completed_at_str: String = row.get(2)?;
        let completed_at = match completed_at_str.parse::<DateTime<Utc>>() {
            Ok(dt) => dt,
            Err(_) => continue,
        };
        map.insert(slug, LastRun { team, completed_at });
    }

    Ok(map)
}

#[cfg(test)]
mod tests;
