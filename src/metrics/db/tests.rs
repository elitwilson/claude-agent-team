use super::*;
use rusqlite::Connection;

fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();
    conn
}

// --- init_db tests ---

#[test]
fn test_init_db_creates_tables() {
    let conn = setup_db();
    // Verify runs table exists
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM runs", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);

    // Verify agent_usage table exists
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM agent_usage", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_init_db_is_idempotent() {
    let conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();
    // Calling again should not error
    init_db(&conn).unwrap();
}

// --- insert_run tests ---

#[test]
fn test_insert_run_returns_id() {
    let conn = setup_db();
    let id = insert_run(
        &conn,
        "my-feature",
        "feature-dev",
        "/Users/charlo/dev/project",
        "2026-03-27T10:00:00Z",
        "2026-03-27T11:00:00Z",
        0,
    )
    .unwrap();
    assert!(id > 0);
}

#[test]
fn test_insert_run_stores_correct_data() {
    let conn = setup_db();
    let id = insert_run(
        &conn,
        "my-feature",
        "feature-dev",
        "/Users/charlo/dev/project",
        "2026-03-27T10:00:00Z",
        "2026-03-27T11:00:00Z",
        1,
    )
    .unwrap();

    let (slug, team, project, exit_code): (String, String, String, i32) = conn
        .query_row(
            "SELECT feature_slug, team, project, agent_exit_code FROM runs WHERE id = ?",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();

    assert_eq!(slug, "my-feature");
    assert_eq!(team, "feature-dev");
    assert_eq!(project, "/Users/charlo/dev/project");
    assert_eq!(exit_code, 1);
}

// --- insert_agent_usage tests ---

#[test]
fn test_insert_agent_usage_stores_data() {
    let conn = setup_db();
    let run_id = insert_run(
        &conn,
        "feat",
        "team",
        "/project",
        "2026-03-27T10:00:00Z",
        "2026-03-27T11:00:00Z",
        0,
    )
    .unwrap();

    insert_agent_usage(&conn, run_id, "orchestrator", 1000, 500, 200, 100).unwrap();

    let (role, input, output, cache_create, cache_read): (String, i64, i64, i64, i64) = conn
        .query_row(
            "SELECT agent_role, input_tokens, output_tokens, cache_creation_tokens, cache_read_tokens FROM agent_usage WHERE run_id = ?",
            [run_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .unwrap();

    assert_eq!(role, "orchestrator");
    assert_eq!(input, 1000i64);
    assert_eq!(output, 500i64);
    assert_eq!(cache_create, 200i64);
    assert_eq!(cache_read, 100i64);
}

#[test]
fn test_insert_multiple_agents_for_one_run() {
    let conn = setup_db();
    let run_id = insert_run(
        &conn,
        "feat",
        "team",
        "/project",
        "2026-03-27T10:00:00Z",
        "2026-03-27T11:00:00Z",
        0,
    )
    .unwrap();

    insert_agent_usage(&conn, run_id, "orchestrator", 100, 50, 0, 0).unwrap();
    insert_agent_usage(&conn, run_id, "coder", 200, 100, 0, 0).unwrap();
    insert_agent_usage(&conn, run_id, "reviewer", 150, 75, 0, 0).unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM agent_usage WHERE run_id = ?",
            [run_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 3);
}
