mod config;
mod metrics;
mod mr;
mod preflight;
mod prompt;
mod runner;
mod tui;

use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cwd = std::env::current_dir().context("Failed to get current directory")?;

    // Load config (falls back to defaults if no .claude-agent-team.toml)
    let config = config::Config::load(&cwd)?;

    // Resolve workflow dir early — fail fast if it can't be found
    let workflow_dir = prompt::resolve_workflow_dir()?;

    // Discover specs and teams
    let specs_dir = cwd.join(&config.specs_dir);
    let teams_dir = Path::new(&workflow_dir).join("prompts").join("teams");

    let specs = config::discover_specs(&specs_dir).context("Failed to discover spec files")?;
    let teams = config::discover_teams(&teams_dir).context("Failed to discover team files")?;

    if specs.is_empty() {
        anyhow::bail!("No spec files found in {}", specs_dir.display());
    }
    if teams.is_empty() {
        anyhow::bail!("No team files found in {}", teams_dir.display());
    }

    // Run TUI — clears and restores terminal on exit
    let selection = tui::ui::run_tui(specs, teams, &config.default_team)?;
    let selection = match selection {
        Some(s) => s,
        None => {
            println!("Cancelled.");
            return Ok(());
        }
    };

    // Derive feature slug from spec filename
    let feature_slug = selection
        .spec
        .strip_suffix(".md")
        .unwrap_or(&selection.spec)
        .to_string();

    // Build full relative spec path for the prompt template (e.g. docs/specs/my-feature.md)
    let spec_file_path = format!("{}/{}", config.specs_dir, selection.spec);

    // Preflight: clean check, checkout base, pull, create branch
    let branch = preflight::run_preflight(&config.base_branch, &feature_slug)
        .context("Preflight failed")?;

    // Render prompt template
    let template_path = Path::new(&workflow_dir)
        .join("prompts")
        .join("teams")
        .join(format!("{}.md", selection.team));
    let rendered_prompt = prompt::render_prompt(
        &template_path,
        &spec_file_path,
        &feature_slug,
        &workflow_dir,
        &selection.team,
    )?;

    // Record start time
    let started_at = Utc::now().to_rfc3339();

    // Load OAuth token
    let oauth_token = runner::load_oauth_token();
    if oauth_token.is_none() {
        eprintln!("Warning: Could not load OAuth token from Keychain — proceeding without it.");
    }

    // Build log path
    let date = chrono::Local::now().format("%Y%m%d").to_string();
    let log_path = runner::build_log_path(&feature_slug, &date);

    // Spawn claude and wait
    let exit_code = runner::run_claude(
        &rendered_prompt,
        selection.headless,
        &log_path,
        oauth_token.as_deref(),
    )?;

    // Record completion time
    let completed_at = Utc::now().to_rfc3339();

    // Collect metrics — non-fatal
    let metrics_written = collect_metrics(
        &cwd,
        &selection.team,
        &feature_slug,
        &started_at,
        &completed_at,
        exit_code,
    );

    // Create MR
    let mr_created = match mr::create_mr(&feature_slug, &config.base_branch, exit_code) {
        Ok(()) => true,
        Err(e) => {
            eprintln!("Warning: MR creation failed: {e:#}");
            false
        }
    };

    // Print summary
    println!("{}", mr::format_summary(&branch, mr_created, metrics_written));

    Ok(())
}

fn collect_metrics(
    cwd: &Path,
    team: &str,
    feature_slug: &str,
    started_at: &str,
    completed_at: &str,
    exit_code: i32,
) -> bool {
    let result = (|| -> Result<()> {
        let cwd_str = cwd.to_str().context("CWD path is not valid UTF-8")?;
        let project_dir = cwd_str.trim_start_matches('/').replace('/', "-");

        let jsonl_files = metrics::parser::discover_jsonl_files(&project_dir)?;
        let usages = metrics::parser::collect_agent_usage(&jsonl_files, started_at)?;

        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let db_path = Path::new(&home)
            .join(".claude")
            .join("claude-agent-team-metrics.db");
        let conn =
            rusqlite::Connection::open(&db_path).context("Failed to open metrics database")?;
        metrics::db::init_db(&conn)?;

        let run_id = metrics::db::insert_run(
            &conn,
            feature_slug,
            team,
            &project_dir,
            started_at,
            completed_at,
            exit_code,
        )?;

        for usage in &usages {
            metrics::db::insert_agent_usage(
                &conn,
                run_id,
                &usage.role,
                usage.input_tokens,
                usage.output_tokens,
                usage.cache_creation_tokens,
                usage.cache_read_tokens,
            )?;
        }

        Ok(())
    })();

    match result {
        Ok(()) => true,
        Err(e) => {
            eprintln!("Warning: Metrics collection failed: {e:#}");
            false
        }
    }
}
