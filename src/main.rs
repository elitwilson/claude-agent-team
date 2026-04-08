#[cfg(not(target_os = "macos"))]
compile_error!("claude-launch only supports macOS (scheduler requires launchd and Keychain)");

mod accounts;
mod config;
mod install;
mod metrics;
mod new_team;
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

    if args.get(1).map(|s| s.as_str()) == Some("new-team") {
        if let Err(e) = run_new_team(&args[2..]) {
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
    let builtin_teams_dir = Path::new(&workflow_dir).join("prompts").join("teams");
    let (user_dir, project_dir, project_teams_dir) =
        build_dirs(&workflow_dir, config.custom_dir.as_deref(), &cwd);
    let user_teams_dir = Path::new(&user_dir).join("teams");

    let spec_entries =
        config::discover_specs(&specs_dir).context("Failed to discover spec files")?;
    let team_entries = config::discover_teams(
        &builtin_teams_dir,
        &user_teams_dir,
        project_teams_dir.as_deref(),
    )
    .context("Failed to discover team files")?;
    let team_names: Vec<String> = team_entries.iter().map(|e| e.name.clone()).collect();

    if team_entries.is_empty() {
        anyhow::bail!("No team files found in {}", builtin_teams_dir.display());
    }

    // Load accounts (empty vec if no config file)
    let accounts = accounts::load_accounts();

    // Run TUI — clears and restores terminal on exit
    let selection =
        tui::ui::run_tui(spec_entries, team_names, &config.default_team, &cwd, accounts)?;
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

    // Read base_branch from spec frontmatter (required field)
    let spec_path = specs_dir.join(&selection.spec);
    let base_branch =
        config::read_base_branch(&spec_path).context("Failed to read base_branch from spec")?;

    // Preflight: clean check, checkout base, pull, create branch
    let branch =
        preflight::run_preflight(&base_branch, &feature_slug).context("Preflight failed")?;

    // Look up selected team entry to get its absolute path
    let team_entry = find_team_entry(&team_entries, &selection.team)
        .with_context(|| format!("Selected team '{}' not found in discovered entries", selection.team))?;

    // Render prompt template using the entry's path directly (works for built-in, user, and project teams)
    let rendered_prompt = prompt::render_prompt(
        &team_entry.path,
        &spec_file_path,
        &feature_slug,
        &workflow_dir,
        &selection.team,
        &user_dir,
        &project_dir,
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

    let (user_dir, project_dir, project_teams_dir) =
        build_dirs(&workflow_dir, config.custom_dir.as_deref(), &cwd);

    let feature_slug = run_args
        .spec
        .strip_suffix(".md")
        .unwrap_or(&run_args.spec)
        .to_string();
    let spec_file_path = format!("{}/{}", config.specs_dir, run_args.spec);

    // Read base_branch from spec frontmatter (required field)
    let spec_path = cwd.join(&config.specs_dir).join(&run_args.spec);
    let base_branch = config::read_base_branch(&spec_path).context("Failed to read base_branch from spec")?;
    eprintln!("[claude-launch] run_scheduled: base_branch={base_branch}");

    // Spec hash integrity check: if --spec-hash was provided, verify the file hasn't changed
    if let Some(ref expected_hash) = run_args.spec_hash {
        let actual_hash = scheduler::hash_spec_file(&spec_path)
            .context("Failed to hash spec file for integrity check")?;
        if actual_hash != *expected_hash {
            eprintln!(
                "Error: Spec '{}' has changed since it was scheduled (hash mismatch). \
                 Re-schedule to run the updated spec.",
                run_args.spec
            );
            std::process::exit(1);
        }
    }

    // Preflight: clean check, checkout base, pull, create branch
    eprintln!("[claude-launch] run_scheduled: running preflight");
    let _branch =
        preflight::run_preflight(&base_branch, &feature_slug).context("Preflight failed")?;
    eprintln!("[claude-launch] run_scheduled: preflight done");

    // Render prompt template — discover teams to get the correct path for the selected team
    eprintln!("[claude-launch] run_scheduled: rendering prompt");
    let builtin_teams_dir = Path::new(&workflow_dir).join("prompts").join("teams");
    let user_teams_dir = Path::new(&user_dir).join("teams");
    let team_entries = config::discover_teams(
        &builtin_teams_dir,
        &user_teams_dir,
        project_teams_dir.as_deref(),
    )
    .context("Failed to discover team files")?;
    let team_entry = find_team_entry(&team_entries, &run_args.team)
        .with_context(|| format!("Scheduled team '{}' not found in discovered entries", run_args.team))?;
    let rendered_prompt = prompt::render_prompt(
        &team_entry.path,
        &spec_file_path,
        &feature_slug,
        &workflow_dir,
        &run_args.team,
        &user_dir,
        &project_dir,
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

fn run_new_team(args: &[String]) -> Result<()> {
    new_team::run(args)
}

/// Compute `user_dir` and `project_dir` strings plus the optional `project_teams_dir` path
/// from the resolved `workflow_dir` and config's `custom_dir`.
///
/// - `user_dir`  → `<workflow_dir>/user`
/// - `project_dir` → `<cwd>/<custom_dir>` if configured, empty string otherwise
/// - `project_teams_dir` → `Some(<cwd>/<custom_dir>/teams)` if configured, `None` otherwise
fn build_dirs(
    workflow_dir: &str,
    custom_dir: Option<&str>,
    cwd: &Path,
) -> (String, String, Option<std::path::PathBuf>) {
    let user_dir = format!("{}/user", workflow_dir);
    let (project_dir, project_teams_dir) = match custom_dir {
        Some(d) => {
            let base = cwd.join(d);
            let teams = base.join("teams");
            (base.to_string_lossy().into_owned(), Some(teams))
        }
        None => (String::new(), None),
    };
    (user_dir, project_dir, project_teams_dir)
}

/// Find a `TeamEntry` by name from a slice of entries.
fn find_team_entry<'a>(entries: &'a [config::TeamEntry], name: &str) -> Option<&'a config::TeamEntry> {
    entries.iter().find(|e| e.name == name)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // --- build_dirs tests ---

    #[test]
    fn test_build_dirs_no_custom_dir() {
        let (user_dir, project_dir, project_teams_dir) =
            build_dirs("/home/user/.claude-launch", None, Path::new("/project"));
        assert_eq!(user_dir, "/home/user/.claude-launch/user");
        assert_eq!(project_dir, "");
        assert!(project_teams_dir.is_none());
    }

    #[test]
    fn test_build_dirs_with_custom_dir() {
        let cwd = PathBuf::from("/project/root");
        let (user_dir, project_dir, project_teams_dir) =
            build_dirs("/home/user/.claude-launch", Some("custom-teams"), &cwd);
        assert_eq!(user_dir, "/home/user/.claude-launch/user");
        assert_eq!(project_dir, "/project/root/custom-teams");
        assert_eq!(
            project_teams_dir,
            Some(PathBuf::from("/project/root/custom-teams/teams"))
        );
    }

    // --- find_team_entry tests ---

    #[test]
    fn test_find_team_entry_returns_matching_entry() {
        let entries = vec![
            config::TeamEntry {
                name: "alpha".to_string(),
                path: PathBuf::from("/some/alpha.md"),
                source: config::TeamSource::BuiltIn,
            },
            config::TeamEntry {
                name: "beta".to_string(),
                path: PathBuf::from("/some/beta.md"),
                source: config::TeamSource::User,
            },
        ];

        let found = find_team_entry(&entries, "beta").unwrap();
        assert_eq!(found.name, "beta");
        assert!(matches!(found.source, config::TeamSource::User));
    }

    #[test]
    fn test_find_team_entry_returns_none_for_unknown() {
        let entries: Vec<config::TeamEntry> = vec![];
        assert!(find_team_entry(&entries, "ghost").is_none());
    }

}
