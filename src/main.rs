mod accounts;
mod config;
mod install;
mod metrics;
mod preflight;
mod prefs;
mod prompt;
mod run_cmd;
mod runner;
mod scheduler;
mod tui;

use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use tui::app::RunMode;

fn main() {
    // Check for `run` subcommand before entering TUI path
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("run") {
        if let Err(e) = run_scheduled(&args[2..]) {
            eprintln!("Error: {e:#}");
            std::process::exit(1);
        }
        return;
    }

    if let Err(e) = run() {
        eprintln!("Error: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    if !install::is_installed() {
        println!("First-time setup: linking rules and registering hooks in ~/.claude");
        install::run_install()?;
    }

    let cwd = std::env::current_dir().context("Failed to get current directory")?;

    // Load config (falls back to defaults if no .claude-launch.toml)
    let config = config::Config::load(&cwd)?;

    // Resolve workflow dir early — fail fast if it can't be found
    let workflow_dir = prompt::resolve_workflow_dir()?;

    // Discover specs and teams
    let specs_dir = cwd.join(&config.specs_dir);
    let teams_dir = Path::new(&workflow_dir).join("prompts").join("teams");

    let spec_entries =
        config::discover_specs(&specs_dir).context("Failed to discover spec files")?;
    let teams = config::discover_teams(&teams_dir).context("Failed to discover team files")?;

    if teams.is_empty() {
        anyhow::bail!("No team files found in {}", teams_dir.display());
    }

    // Load accounts (empty vec if no config file)
    let accounts = accounts::load_accounts();

    // Run TUI — clears and restores terminal on exit
    let selection = tui::ui::run_tui(spec_entries, teams, &config.default_team, &cwd, accounts)?;
    let selection = match selection {
        Some(s) => s,
        None => {
            println!("Cancelled.");
            return Ok(());
        }
    };

    if selection.mode == RunMode::DraftRun {
        let input_file = format!("{}/{}", config.specs_dir, selection.spec);
        let drafter_template = Path::new(&workflow_dir)
            .join("prompts")
            .join("agents")
            .join("drafter.md");
        let rendered_prompt = prompt::render_drafter_prompt(
            &drafter_template,
            &input_file,
            &config.specs_dir,
            &workflow_dir,
        )?;
        let date = chrono::Local::now().format("%Y%m%d").to_string();
        let log_path = runner::build_log_path("drafter", &date);
        let oauth_token = match &selection.account {
            Some(label) => accounts::load_token_for_account(label),
            None => runner::load_oauth_token(),
        };
        if oauth_token.is_none() {
            eprintln!("Warning: Could not load OAuth token from Keychain — proceeding without it.");
        }
        runner::run_claude(
            &rendered_prompt,
            selection.headless,
            &log_path,
            oauth_token.as_deref(),
        )?;
        println!(
            "Drafter run complete. Check {} for the new spec.",
            config.specs_dir
        );
        return Ok(());
    }

    // Derive feature slug from spec filename
    let feature_slug = selection
        .spec
        .strip_suffix(".md")
        .unwrap_or(&selection.spec)
        .to_string();

    // Build full relative spec path for the prompt template (e.g. docs/specs/my-feature.md)
    let spec_file_path = format!("{}/{}", config.specs_dir, selection.spec);

    // Preflight: clean check, checkout base, pull, create branch
    let branch =
        preflight::run_preflight(&config.base_branch, &feature_slug).context("Preflight failed")?;

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

    // Load OAuth token — use account-specific token if account selected
    let oauth_token = match &selection.account {
        Some(label) => accounts::load_token_for_account(label),
        None => runner::load_oauth_token(),
    };
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

    // Print summary
    let metrics_status = if metrics_written {
        "metrics written"
    } else {
        "metrics collection failed"
    };
    println!("Branch: {branch} | {metrics_status}");

    Ok(())
}

fn run_scheduled(args: &[String]) -> Result<()> {
    eprintln!("[claude-launch] run_scheduled: starting");
    let run_args = run_cmd::parse_run_args(args)?;
    eprintln!("[claude-launch] run_scheduled: spec={} team={} headless={}", run_args.spec, run_args.team, run_args.headless);

    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    eprintln!("[claude-launch] run_scheduled: cwd={}", cwd.display());
    let config = config::Config::load(&cwd)?;
    eprintln!("[claude-launch] run_scheduled: config loaded");
    let workflow_dir = prompt::resolve_workflow_dir()?;
    eprintln!("[claude-launch] run_scheduled: workflow_dir={workflow_dir}");

    let feature_slug = run_args
        .spec
        .strip_suffix(".md")
        .unwrap_or(&run_args.spec)
        .to_string();
    let spec_file_path = format!("{}/{}", config.specs_dir, run_args.spec);

    // Preflight: clean check, checkout base, pull, create branch
    eprintln!("[claude-launch] run_scheduled: running preflight");
    let _branch =
        preflight::run_preflight(&config.base_branch, &feature_slug).context("Preflight failed")?;
    eprintln!("[claude-launch] run_scheduled: preflight done");

    // Render prompt template
    eprintln!("[claude-launch] run_scheduled: rendering prompt");
    let template_path = Path::new(&workflow_dir)
        .join("prompts")
        .join("teams")
        .join(format!("{}.md", run_args.team));
    let rendered_prompt = prompt::render_prompt(
        &template_path,
        &spec_file_path,
        &feature_slug,
        &workflow_dir,
        &run_args.team,
    )?;

    let started_at = Utc::now().to_rfc3339();

    // Load OAuth token — use account-specific token if account set in plist args
    let oauth_token = match &run_args.account {
        Some(label) => accounts::load_token_for_account(label),
        None => runner::load_oauth_token(),
    };
    if oauth_token.is_none() {
        eprintln!("Warning: Could not load OAuth token from Keychain — proceeding without it.");
    }

    let date = chrono::Local::now().format("%Y%m%d").to_string();
    let log_path = runner::build_log_path(&feature_slug, &date);

    let exit_code = runner::run_claude(
        &rendered_prompt,
        run_args.headless,
        &log_path,
        oauth_token.as_deref(),
    )?;

    let completed_at = Utc::now().to_rfc3339();

    collect_metrics(
        &cwd,
        &run_args.team,
        &feature_slug,
        &started_at,
        &completed_at,
        exit_code,
    );

    // Self-cleanup: remove the plist if this was a scheduled invocation
    if let Some(plist_path) = &run_args.cleanup_plist {
        scheduler::cleanup_plist(plist_path)
            .context("Fatal: Failed to clean up scheduled run plist")?;
    }

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
        let project_dir = cwd_str.replace('/', "-");

        let jsonl_files = metrics::parser::discover_jsonl_files(&project_dir)?;
        let usages = metrics::parser::collect_agent_usage(&jsonl_files, started_at)?;

        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let db_path = Path::new(&home)
            .join(".claude")
            .join("claude-launch-metrics.db");
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
