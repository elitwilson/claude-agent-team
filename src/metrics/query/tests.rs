use super::*;
use crate::metrics::db::{init_db, insert_agent_usage, insert_run};
use rusqlite::Connection;

fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();
    conn
}

/// Helper: insert a run with multiple agents and return the run ID.
fn seed_run(
    conn: &Connection,
    slug: &str,
    team: &str,
    started_at: &str,
    exit_code: i32,
    agents: &[(&str, u64, u64, u64, u64)],
) -> i64 {
    let run_id = insert_run(
        conn,
        slug,
        team,
        "/project",
        started_at,
        "2026-03-27T12:00:00Z",
        exit_code,
    )
    .unwrap();
    for &(role, input, output, cache_create, cache_read) in agents {
        insert_agent_usage(conn, run_id, role, input, output, cache_create, cache_read).unwrap();
    }
    run_id
}

// --- fetch_runs: token summing ---

#[test]
fn test_fetch_runs_sums_tokens_across_agents() {
    let conn = setup_db();
    seed_run(
        &conn,
        "my-feature",
        "feature-dev",
        "2026-03-27T10:00:00Z",
        0,
        &[
            ("orchestrator", 1000, 500, 200, 100),
            ("coder", 2000, 1000, 300, 150),
            ("reviewer", 500, 250, 0, 50),
        ],
    );

    let runs = fetch_runs(&conn).unwrap();
    assert_eq!(runs.len(), 1);

    let run = &runs[0];
    assert_eq!(run.total_input, 3500); // 1000 + 2000 + 500
    assert_eq!(run.total_output, 1750); // 500 + 1000 + 250
    assert_eq!(run.total_cache, 800); // (200+100) + (300+150) + (0+50)
}

// --- fetch_runs: field mapping ---

#[test]
fn test_fetch_runs_maps_fields_correctly() {
    let conn = setup_db();
    seed_run(
        &conn,
        "auth-refactor",
        "platform",
        "2026-03-20T09:30:00Z",
        1,
        &[("coder", 100, 50, 10, 5)],
    );

    let runs = fetch_runs(&conn).unwrap();
    assert_eq!(runs.len(), 1);

    let run = &runs[0];
    assert_eq!(run.run_date, "2026-03-20");
    assert_eq!(run.feature_slug, "auth-refactor");
    assert_eq!(run.team, "platform");
    assert_eq!(run.exit_code, 1);
}

// --- fetch_runs: ordering ---

#[test]
fn test_fetch_runs_ordered_by_started_at_desc() {
    let conn = setup_db();
    seed_run(
        &conn,
        "oldest",
        "team",
        "2026-03-01T10:00:00Z",
        0,
        &[("a", 10, 10, 0, 0)],
    );
    seed_run(
        &conn,
        "middle",
        "team",
        "2026-03-15T10:00:00Z",
        0,
        &[("a", 10, 10, 0, 0)],
    );
    seed_run(
        &conn,
        "newest",
        "team",
        "2026-03-27T10:00:00Z",
        0,
        &[("a", 10, 10, 0, 0)],
    );

    let runs = fetch_runs(&conn).unwrap();
    assert_eq!(runs.len(), 3);
    assert_eq!(runs[0].feature_slug, "newest");
    assert_eq!(runs[1].feature_slug, "middle");
    assert_eq!(runs[2].feature_slug, "oldest");
}

// --- fetch_runs: empty database ---

#[test]
fn test_fetch_runs_returns_empty_vec_when_no_rows() {
    let conn = setup_db();
    let runs = fetch_runs(&conn).unwrap();
    assert!(runs.is_empty());
}

// --- fetch_runs: run with no agent usage ---

#[test]
fn test_fetch_runs_includes_run_with_no_agent_usage() {
    let conn = setup_db();
    // Insert a run with no agent_usage rows
    insert_run(
        &conn,
        "no-agents",
        "team",
        "/project",
        "2026-03-27T10:00:00Z",
        "2026-03-27T11:00:00Z",
        0,
    )
    .unwrap();

    let runs = fetch_runs(&conn).unwrap();
    assert_eq!(runs.len(), 1);

    let run = &runs[0];
    assert_eq!(run.feature_slug, "no-agents");
    assert_eq!(run.total_input, 0);
    assert_eq!(run.total_output, 0);
    assert_eq!(run.total_cache, 0);
}

// --- fetch_runs: multiple runs with different agents ---

#[test]
fn test_fetch_runs_does_not_mix_tokens_between_runs() {
    let conn = setup_db();
    seed_run(
        &conn,
        "run-a",
        "team",
        "2026-03-27T10:00:00Z",
        0,
        &[("coder", 1000, 500, 100, 50)],
    );
    seed_run(
        &conn,
        "run-b",
        "team",
        "2026-03-26T10:00:00Z",
        0,
        &[("coder", 2000, 1000, 200, 100)],
    );

    let runs = fetch_runs(&conn).unwrap();
    assert_eq!(runs.len(), 2);

    // run-a is newer, should be first
    assert_eq!(runs[0].feature_slug, "run-a");
    assert_eq!(runs[0].total_input, 1000);
    assert_eq!(runs[0].total_output, 500);

    assert_eq!(runs[1].feature_slug, "run-b");
    assert_eq!(runs[1].total_input, 2000);
    assert_eq!(runs[1].total_output, 1000);
}
