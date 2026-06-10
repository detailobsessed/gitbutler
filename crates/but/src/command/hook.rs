//! Implementation of `but hook` subcommands.
//!
//! These commands provide GitButler's workspace guard and cleanup logic
//! as standalone CLI commands that any external hook manager can invoke.

use anyhow::Result;
use ratatui::style::Modifier;
use serde::Serialize;

use crate::theme::{self, Paint};
use crate::utils::OutputChannel;

/// Human-readable hint shown in all contexts where managed hooks are absent
/// (disabled by config or displaced by an external manager) to tell the user
/// how to re-enable them.
pub const FORCE_HOOKS_RECOVERY_HINT: &str =
    "To switch back to GitButler-managed hooks: but setup --force-hooks";

/// Information about a single installed hook.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookInfo {
    /// The name of the hook (e.g. "pre-commit")
    pub name: String,
    /// The status of the hook
    pub status: HookStatus,
}

/// Status of a hook installation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HookStatus {
    /// Hook was installed (may have backed up existing hook)
    Installed {
        /// Path to backup if one was created, None otherwise
        backup_path: Option<String>,
    },
    /// Hook was already configured
    AlreadyConfigured,
    /// Hook was skipped
    Skipped,
}

/// Details about an external hook manager detected during setup.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookManagerInfo {
    /// The name of the detected hook manager (e.g. "prek", "lefthook")
    pub name: String,
    /// Whether GitButler installed its own managed hooks
    pub hooks_installed: bool,
}

/// Convert a list of per-hook installation results into [`HookInfo`] entries for JSON output.
///
/// Each `(name, backup_status)` pair becomes a [`HookInfo`] with
/// [`HookStatus::Installed`], carrying the backup path (if any) from the status.
pub fn hook_results_to_hook_info(
    results: &[(String, but_hooks::managed_hooks::HookBackupStatus)],
) -> Vec<HookInfo> {
    results
        .iter()
        .map(|(name, backup_status)| HookInfo {
            name: name.clone(),
            status: HookStatus::Installed {
                backup_path: backup_status.to_backup_path().map(str::to_owned),
            },
        })
        .collect()
}

/// Build [`HookInfo`] entries for hooks that were already configured (no changes made).
///
/// Produces one entry per managed hook name, all with [`HookStatus::AlreadyConfigured`].
pub fn already_configured_hook_info() -> Vec<HookInfo> {
    but_hooks::hook_manager::MANAGED_HOOK_NAMES
        .iter()
        .map(|name| HookInfo {
            name: name.to_string(),
            status: HookStatus::AlreadyConfigured,
        })
        .collect()
}

/// Open a git repository for hook subcommands.
///
/// Uses [`gix::discover_with_environment_overrides()`] so that the repository
/// is found via a directory walk when `GIT_DIR` is not set (e.g. direct
/// invocation by prek or in tests), and the git-provided `GIT_DIR` /
/// `GIT_WORK_TREE` environment variables are respected when git itself
/// invokes this command as a hook subprocess.
fn open_repo(current_dir: &std::path::Path) -> anyhow::Result<gix::Repository> {
    Ok(gix::discover_with_environment_overrides(current_dir)?)
}

/// Return the short name of the currently checked-out branch, or `None`
/// on detached HEAD / error.
fn current_branch_name(repo: &gix::Repository) -> Option<String> {
    repo.head()
        .ok()?
        .referent_name()
        .map(|n| n.shorten().to_string())
}

/// Block an action when on the `gitbutler/workspace` branch.
///
/// Opens the repository, resolves the current branch, and if it is
/// `gitbutler/workspace` prints a coloured error with the supplied
/// `headline` and `help_lines`, then bails with `bail_msg`.
/// Returns `Ok(())` on any other branch, detached HEAD, or if the
/// repository cannot be opened.
fn workspace_guard(
    out: &mut OutputChannel,
    current_dir: &std::path::Path,
    headline: &str,
    help_lines: &[&str],
    bail_msg: &str,
) -> Result<()> {
    let repo = match open_repo(current_dir) {
        Ok(repo) => repo,
        Err(_) => return Ok(()),
    };

    let Some(branch_name) = current_branch_name(&repo) else {
        return Ok(());
    };

    if branch_name == "gitbutler/workspace" {
        if let Some(out) = out.for_human() {
            let t = theme::get();
            writeln!(out)?;
            writeln!(
                out,
                "{}",
                t.error.add_modifier(Modifier::BOLD).paint(headline)
            )?;
            writeln!(out)?;
            for line in help_lines {
                writeln!(out, "  {}", t.hint.paint(*line))?;
            }
            writeln!(out)?;
        }
        anyhow::bail!("{bail_msg}");
    }

    Ok(())
}

/// Return the branch name from which the most recent checkout originated.
///
/// Walks the `HEAD` reflog in reverse (most-recent-first) looking for the
/// first `"checkout: moving from <src> to <dst>"` entry, and returns `<src>`.
///
/// # Why the reflog and not the hook arguments?
///
/// Git's `post-checkout` hook receives `$1` (previous HEAD commit SHA), `$2`
/// (new HEAD commit SHA), and `$3` (branch-checkout flag) — it does **not**
/// receive branch names. Recovering the previous *branch name* from the commit
/// SHA alone would require a reverse ref-lookup that is unreliable when the
/// SHA is reachable from multiple branches. The reflog entry written by Git at
/// checkout time is the authoritative, stable source for this information; its
/// `"checkout: moving from <src> to <dst>"` message format has been stable
/// since Git 1.8 and is relied upon by many tools.
///
/// When reflogs are disabled (`log.keepBackupRefs=false`, `--no-log`, or a
/// freshly-cloned repo before any checkout) `previous_checkout_branch_name`
/// returns `None` and the caller silently skips the workspace-guard check.
/// This is the safe default: better to miss a notification than to fire
/// incorrectly.
///
/// Returns `None` when:
/// - the reflog is absent or empty (e.g. freshly-cloned repo with reflogs disabled),
/// - no checkout entry exists yet, or
/// - the reflog entry cannot be parsed.
fn previous_checkout_branch_name(repo: &gix::Repository) -> Option<String> {
    use gix::bstr::ByteSlice;

    let head_ref = repo.find_reference("HEAD").ok()?;
    let mut log_iter = head_ref.log_iter();
    let mut reverse = log_iter.rev().ok()??;

    for entry in reverse.by_ref() {
        let Some(line) = entry.ok() else { continue };
        let Some(msg) = line.message.to_str().ok() else {
            continue;
        };
        if let Some(rest) = msg.strip_prefix("checkout: moving from ") {
            // Use rsplit_once so branch names containing " to " are parsed
            // correctly — the *last* " to " is always the separator.
            let (from_branch, _to) = rest.rsplit_once(" to ")?;
            return (!from_branch.is_empty()).then(|| from_branch.to_owned());
        }
    }
    None
}

/// Run the pre-commit workspace guard.
///
/// Blocks direct `git commit` on the `gitbutler/workspace` branch with a
/// helpful error message. Exits 0 (allow) on any other branch or if the
/// repository cannot be opened.
///
/// Respects git-provided environment variables (`GIT_DIR`, etc.) when
/// invoked as a hook subprocess.
pub fn pre_commit(out: &mut OutputChannel, current_dir: &std::path::Path) -> Result<()> {
    workspace_guard(
        out,
        current_dir,
        "GITBUTLER_ERROR: Cannot commit directly to gitbutler/workspace branch.",
        &[
            "GitButler manages commits on this branch. Please use GitButler to commit your changes:",
            "- Use the GitButler app to create commits",
            "- Or run 'but commit' from the command line",
            "",
            "If you want to exit GitButler mode and use normal git:",
            "- Run 'but teardown' to switch to a regular branch",
            "- Or directly checkout another branch: git checkout <branch>",
        ],
        "Cannot commit directly to gitbutler/workspace branch",
    )
}

/// Run the post-checkout notification logic for hook-manager users.
///
/// When leaving the `gitbutler/workspace` branch (branch checkout only),
/// prints an informational message directing the user to `but setup`.
/// On file checkouts or when staying on workspace, does nothing.
///
/// # Difference from the shell-based post-checkout hook
///
/// The shell hook installed by `install_managed_hooks` also **uninstalls**
/// GitButler's managed hooks when leaving workspace (since GitButler owns
/// them). This command intentionally does **not** uninstall anything because
/// it runs inside a hook manager (prek, husky, etc.) that owns the hook
/// files — there are no GitButler-managed hooks to remove.
///
/// Respects git-provided environment variables (`GIT_DIR`, etc.) when
/// invoked as a hook subprocess.
pub fn post_checkout(
    out: &mut OutputChannel,
    current_dir: &std::path::Path,
    _prev_head: &str,
    _new_head: &str,
    is_branch_checkout: &str,
) -> Result<()> {
    // Only act on branch checkouts (not file checkouts)
    if is_branch_checkout != "1" {
        return Ok(());
    }

    let repo = match open_repo(current_dir) {
        Ok(repo) => repo,
        Err(_) => return Ok(()),
    };

    let prev_branch = previous_checkout_branch_name(&repo);
    if prev_branch.as_deref() != Some("gitbutler/workspace") {
        return Ok(());
    }

    let new_branch = current_branch_name(&repo).unwrap_or_default();

    // If we're still on gitbutler/workspace (e.g. same-branch checkout), nothing to report
    if new_branch == "gitbutler/workspace" {
        return Ok(());
    }

    // When called via `but hook post-checkout`, the hook manager owns the
    // hooks — GitButler didn't install managed hooks, so there's nothing
    // to uninstall. Just inform the user they left workspace mode.
    if let Some(out) = out.for_human() {
        let t = theme::get();
        writeln!(out)?;
        writeln!(
            out,
            "{}",
            t.info
                .paint("NOTE: You have left GitButler's managed workspace branch.")
        )?;
        writeln!(
            out,
            "{}",
            t.command_suggestion
                .paint("To return to GitButler mode, run: but setup")
        )?;
        writeln!(out)?;
    }

    Ok(())
}

/// Run the pre-push workspace guard.
///
/// Blocks `git push` when on the `gitbutler/workspace` branch with a
/// helpful error message. Exits 0 (allow) on any other branch or if the
/// repository cannot be opened.
///
/// The `_remote_name` and `_remote_url` parameters match git's pre-push
/// hook signature but are not inspected — the guard decision is based
/// solely on the current branch name.
///
/// Respects git-provided environment variables (`GIT_DIR`, etc.) when
/// invoked as a hook subprocess.
pub fn pre_push(
    out: &mut OutputChannel,
    current_dir: &std::path::Path,
    _remote_name: &str,
    _remote_url: &str,
) -> Result<()> {
    workspace_guard(
        out,
        current_dir,
        "GITBUTLER_ERROR: Cannot push the gitbutler/workspace branch.",
        &[
            "The workspace branch is a synthetic branch managed by GitButler.",
            "Pushing it to a remote would publish GitButler's internal state.",
            "",
            "To push your branches, use:",
            "- The GitButler app to push branches",
            "- Or run 'but push' from the command line",
            "",
            "If you want to exit GitButler mode and push normally:",
            "- Run 'but teardown' to switch to a regular branch",
        ],
        "Cannot push the gitbutler/workspace branch",
    )
}

/// The integration mode for GitButler hooks in this repository.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum HookMode {
    /// GitButler manages hooks directly in the hooks directory.
    Managed,
    /// An external hook manager (e.g. prek) owns the hook files.
    External,
    /// Hook installation is disabled via `gitbutler.installHooks = false`.
    Disabled,
    /// No hooks are installed and no configuration has been set.
    Unconfigured,
}

impl std::fmt::Display for HookMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Managed => write!(f, "GitButler-managed"),
            Self::External => write!(f, "external hook manager"),
            Self::Disabled => write!(f, "disabled"),
            Self::Unconfigured => write!(f, "unconfigured"),
        }
    }
}

/// Ownership classification for a single hook file.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum HookOwner {
    /// The hook file is managed by GitButler (contains the V1 signature).
    Gitbutler,
    /// The hook file is owned by a known external hook manager.
    External,
    /// The hook file exists but is not recognized as GitButler or a known manager.
    User,
    /// No hook file exists at this path.
    Missing,
}

impl HookOwner {
    /// Human-readable label, optionally annotated with the manager name.
    fn display(&self, manager: Option<&str>) -> String {
        match self {
            Self::Gitbutler => "GitButler-managed".to_owned(),
            Self::External => format!("external ({})", manager.unwrap_or("unknown")),
            Self::User => "user hook".to_owned(),
            Self::Missing => "not installed".to_owned(),
        }
    }
}

impl std::fmt::Display for HookOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display(None))
    }
}

/// Status of a single hook file.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SingleHookStatus {
    /// The hook name (e.g. "pre-commit").
    name: String,
    /// Whether the hook file exists on disk.
    exists: bool,
    /// Who owns this hook file.
    owner: HookOwner,
    /// Name of the external hook manager, when `owner` is `external`.
    #[serde(skip_serializing_if = "Option::is_none")]
    manager: Option<String>,
}

/// Full result of the `but hook status` diagnostic command.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HookStatusResult {
    /// The resolved hooks directory path.
    hooks_path: String,
    /// Whether `core.hooksPath` is set (non-default).
    custom_hooks_path: bool,
    /// Value of `gitbutler.installHooks` config key (`true` when not set).
    config_enabled: bool,
    /// The detected hook integration mode.
    mode: HookMode,
    /// Name of the detected external hook manager, if any.
    external_manager: Option<String>,
    /// Per-hook status for each managed hook type.
    hooks: Vec<SingleHookStatus>,
    /// Warning messages (e.g. orphaned hooks).
    warnings: Vec<String>,
    /// Recommended next actions.
    recommendations: Vec<String>,
}

/// Human-readable description of the consequence when a GitButler hook is missing or displaced.
fn hook_missing_description(hook_name: &str) -> &'static str {
    match hook_name {
        "pre-commit" => {
            "workspace guard won't run — commits to the workspace branch won't be blocked"
        }
        "post-checkout" => {
            "workspace cleanup won't run — stale state may persist when switching branches"
        }
        "pre-push" => "push guard won't run — pushes from the workspace branch won't be blocked",
        _ => "GitButler hook won't run",
    }
}

/// Escape a string for safe inclusion inside a single-quoted shell literal.
///
/// Uses the POSIX-portable `'\''` idiom: close the quote, emit an escaped
/// single quote, reopen the quote. The returned string is safe to place
/// between single quotes in `--format shell` output, including paths with
/// embedded apostrophes (valid on Unix filesystems) and any future
/// user-controlled manager-name content.
fn shell_single_quote(value: &str) -> String {
    value.replace('\'', r"'\''")
}

/// Show hook ownership and integration state for the current repository.
///
/// Inspects the hooks directory, `gitbutler.installHooks` config, and
/// hook file contents to report a diagnostic summary. Produces human,
/// JSON, and shell output.
///
/// Respects git-provided environment variables (`GIT_DIR`, etc.) when
/// invoked as a hook subprocess.
pub fn status(out: &mut OutputChannel, current_dir: &std::path::Path) -> Result<()> {
    let repo = open_repo(current_dir)?;

    let hooks_dir = but_hooks::managed_hooks::get_hooks_dir_gix(&repo);
    let default_hooks_dir = repo.git_dir().join("hooks");
    let custom_hooks_path = hooks_dir != default_hooks_dir;
    let config_enabled = but_hooks::managed_hooks::install_managed_hooks_enabled(&repo);

    let workdir = repo
        .workdir()
        .unwrap_or_else(|| repo.git_dir())
        .to_path_buf();

    // Per-hook status
    let mut hooks = Vec::new();
    let mut has_gitbutler_hook = false;
    let mut has_user_hook = false;
    let mut external_manager_name: Option<String> = None;
    let mut has_real_external_hook = false;

    // Detect config-only manager once (no hook files present yet, e.g. prek.toml exists
    // and binary is on PATH, but `prek install` hasn't run). Hoisted outside the loop to
    // avoid redundant disk I/O (config stat + which call) for each of the three hooks.
    let config_only_manager = but_hooks::hook_manager::detect_hook_manager("", &workdir);

    for hook_name in but_hooks::hook_manager::MANAGED_HOOK_NAMES {
        let hook_path = hooks_dir.join(hook_name);
        let (exists, owner, manager) = if hook_path.exists() {
            let content = std::fs::read_to_string(&hook_path).unwrap_or_default();
            if but_hooks::managed_hooks::is_gitbutler_managed_hook(&hook_path) {
                has_gitbutler_hook = true;
                (true, HookOwner::Gitbutler, None)
            } else if let Some(manager) =
                but_hooks::hook_manager::detect_hook_manager(&content, &workdir)
            {
                let name = manager.name().to_owned();
                external_manager_name = Some(name.clone());
                has_real_external_hook = true;
                (true, HookOwner::External, Some(name))
            } else {
                has_user_hook = true;
                (true, HookOwner::User, None)
            }
        } else if let Some(manager) = config_only_manager {
            // Hook file doesn't exist, but a manager is configured + available
            // (e.g. prek.toml present, binary on PATH, hooks not yet installed).
            let name = manager.name().to_owned();
            external_manager_name = Some(name.clone());
            (false, HookOwner::External, Some(name))
        } else {
            (false, HookOwner::Missing, None)
        };

        hooks.push(SingleHookStatus {
            name: (*hook_name).to_owned(),
            exists,
            owner,
            manager,
        });
    }

    // If we detected an external manager only via config (no real hook files owned by it)
    // but there ARE user-owned hook files, the config-only detection is misleading —
    // downgrade those slots to Missing so we don't report contradictory diagnostics.
    if has_user_hook && !has_real_external_hook && external_manager_name.is_some() {
        external_manager_name = None;
        for hook in &mut hooks {
            if hook.owner == HookOwner::External {
                hook.owner = HookOwner::Missing;
                hook.manager = None;
                hook.exists = false;
            }
        }
    }

    // Determine mode
    // External check comes first: `but setup` sets `installHooks = false` when it
    // detects an external manager, so `!config_enabled` alone would mask the real reason.
    let mode = if external_manager_name.is_some() {
        HookMode::External
    } else if !config_enabled {
        HookMode::Disabled
    } else if has_gitbutler_hook {
        HookMode::Managed
    } else {
        HookMode::Unconfigured
    };

    // Warnings
    let mut warnings = Vec::new();
    if custom_hooks_path && but_hooks::managed_hooks::has_managed_hooks_in(&default_hooks_dir) {
        warnings.push(format!(
            "Orphaned GitButler-managed hooks found in {} (core.hooksPath points elsewhere)",
            default_hooks_dir.display()
        ));
    }
    // Warn about each GitButler-required hook that the external manager hasn't installed.
    // A missing hook means the corresponding GitButler guard won't fire, which can lead
    // to silent data-integrity issues (e.g. committing to the workspace branch unguarded).
    if mode == HookMode::External {
        for hook in &hooks {
            if !hook.exists {
                let description = hook_missing_description(&hook.name);
                let mgr = hook.manager.as_deref().unwrap_or("external manager");
                warnings.push(format!(
                    "{} is not installed by {mgr} — {description}",
                    hook.name
                ));
            }
        }
    }
    // In Managed mode, warn about any slot occupied by a user hook instead of a GB hook.
    // The mode label says "GitButler-managed" but the guard for that slot won't fire,
    // which can silently break workspace protection (e.g. the pre-commit workspace guard).
    if mode == HookMode::Managed {
        for hook in &hooks {
            if matches!(hook.owner, HookOwner::User) {
                let description = hook_missing_description(&hook.name);
                warnings.push(format!("{}: user hook present — {description}", hook.name));
            }
        }
    }

    // Recommendations
    let mut recommendations = Vec::new();
    match &mode {
        HookMode::Disabled => {
            recommendations
                .push("Run `but setup --force-hooks` to re-enable GitButler-managed hooks.".into());
        }
        HookMode::External => {
            if let Some(ref mgr) = external_manager_name {
                recommendations.push(format!(
                    "Hooks are managed by {mgr}. Use `but hook pre-commit` etc. in your {mgr} config."
                ));
            }
        }
        HookMode::Unconfigured => {
            recommendations.push("Run `but setup` to install GitButler hooks.".into());
        }
        HookMode::Managed => {}
    }
    // Add a single setup hint if any external-manager hooks are missing.
    let has_missing_external_hooks = mode == HookMode::External && hooks.iter().any(|h| !h.exists);
    if has_missing_external_hooks {
        recommendations.push("Run `but setup` to see integration instructions.".into());
    }
    if custom_hooks_path && but_hooks::managed_hooks::has_managed_hooks_in(&default_hooks_dir) {
        recommendations.push(format!(
            "Remove orphaned hooks: {}",
            but_hooks::hook_manager::orphaned_hooks_remove_command(&default_hooks_dir)
        ));
    }

    let result = HookStatusResult {
        hooks_path: hooks_dir.display().to_string(),
        custom_hooks_path,
        config_enabled,
        mode,
        external_manager: external_manager_name,
        hooks,
        warnings,
        recommendations,
    };

    // Human output
    if let Some(out) = out.for_human() {
        let t = theme::get();
        writeln!(out)?;
        writeln!(
            out,
            "{}",
            t.info.add_modifier(Modifier::BOLD).paint("Hook status")
        )?;
        writeln!(out)?;
        writeln!(
            out,
            "  {:<20} {}",
            t.hint.paint("Hooks path:"),
            result.hooks_path
        )?;
        if result.custom_hooks_path {
            writeln!(
                out,
                "  {:<20} {}",
                t.hint.paint(""),
                t.hint.paint("(set via core.hooksPath)")
            )?;
        }
        writeln!(
            out,
            "  {:<20} gitbutler.installHooks = {}",
            t.hint.paint("Config:"),
            if result.config_enabled {
                "true"
            } else {
                "false"
            }
        )?;
        writeln!(out, "  {:<20} {}", t.hint.paint("Mode:"), result.mode)?;
        if let Some(ref mgr) = result.external_manager {
            writeln!(out, "  {:<20} {}", t.hint.paint("Hook manager:"), mgr)?;
        }
        writeln!(out)?;

        for hook in &result.hooks {
            let status_str = if hook.exists {
                match &hook.owner {
                    HookOwner::Gitbutler => {
                        t.success.paint(format!("✓ {}", hook.owner)).to_string()
                    }
                    HookOwner::External => t
                        .info
                        .paint(format!("● {}", hook.owner.display(hook.manager.as_deref())))
                        .to_string(),
                    HookOwner::User => t.attention.paint(format!("○ {}", hook.owner)).to_string(),
                    HookOwner::Missing => unreachable!(),
                }
            } else {
                let label = match &hook.owner {
                    HookOwner::External => format!(
                        "not configured ({})",
                        hook.manager.as_deref().unwrap_or("unknown")
                    ),
                    other => other.display(hook.manager.as_deref()),
                };
                t.hint.paint(format!("✗ {label}")).to_string()
            };
            writeln!(
                out,
                "  {:<20} {}",
                t.hint.paint(format!("{}:", hook.name)),
                status_str
            )?;
        }
        writeln!(out)?;

        for warning in &result.warnings {
            writeln!(out, "  {}", t.attention.paint(format!("⚠ {warning}")))?;
        }
        for rec in &result.recommendations {
            writeln!(out, "  {}", t.hint.paint(format!("→ {rec}")))?;
        }
        if !result.warnings.is_empty() || !result.recommendations.is_empty() {
            writeln!(out)?;
        }
    }

    // Shell output
    //
    // Values that may contain user-controlled content (paths, manager names)
    // are wrapped through `shell_single_quote` so that an embedded apostrophe
    // cannot break out of the surrounding single-quoted literal and produce
    // syntactically invalid shell when the output is sourced/evaled.
    //
    // `mode` is skipped because `HookMode`'s `Display` impl only emits a
    // closed set of hardcoded labels with no apostrophes.
    if let Some(out) = out.for_shell() {
        writeln!(
            out,
            "hooks_path='{}'",
            shell_single_quote(&result.hooks_path)
        )?;
        writeln!(out, "custom_hooks_path={}", result.custom_hooks_path)?;
        writeln!(out, "config_enabled={}", result.config_enabled)?;
        writeln!(out, "mode='{}'", result.mode)?;
        if let Some(ref mgr) = result.external_manager {
            writeln!(out, "external_manager='{}'", shell_single_quote(mgr))?;
        }
        for hook in &result.hooks {
            writeln!(
                out,
                "hook_{}='{}'",
                hook.name.replace('-', "_"),
                shell_single_quote(&hook.owner.display(hook.manager.as_deref()))
            )?;
        }
    }

    // JSON output
    if let Some(json_out) = out.for_json() {
        json_out.write_value(&result)?;
    }

    Ok(())
}

/// Result of a `but hook install` invocation, suitable for embedding in a
/// parent command's JSON output (e.g. the `hooks` field of `but setup`'s
/// `SetupResult`).
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookInstallReport {
    /// Per-hook outcome, one entry per managed hook name.
    pub hooks: Vec<HookInfo>,
    /// External hook manager detected (and respected), if any.
    pub hook_manager: Option<HookManagerInfo>,
    /// Whether GitButler-managed hooks now exist on disk after this run.
    pub hooks_installed: bool,
    /// Non-fatal warnings (orphaned-hooks sweep, partial install, etc.).
    pub warnings: Vec<String>,
}

/// Result of a `but hook uninstall` invocation.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookUninstallReport {
    /// Names of GitButler-managed hooks removed from the hooks directory.
    pub removed: Vec<String>,
    /// Names of hooks where a backed-up user hook was restored.
    pub restored: Vec<String>,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
    /// External hook manager detected in the hooks directory after uninstall,
    /// if any (e.g. `prek`, `lefthook`).
    pub external_manager: Option<String>,
}

/// Install GitButler-managed git hooks.
///
/// Runs the shared orchestrator [`but_hooks::managed_hooks::ensure_managed_hooks`],
/// dispatches the seven possible outcomes, sweeps for orphaned managed hooks
/// when `core.hooksPath` redirects, and emits human + JSON output. Returns
/// a structured report for callers that want to embed the result in their
/// own JSON output (e.g. `but setup`).
///
/// `force` re-enables the persisted opt-out (`gitbutler.installHooks=true`)
/// and instructs the orchestrator to skip external-manager detection.
pub fn install(
    out: &mut OutputChannel,
    current_dir: &std::path::Path,
    force: bool,
) -> Result<HookInstallReport> {
    let report = install_internal(out, current_dir, force)?;
    if let Some(json_out) = out.for_json() {
        json_out.write_value(&report)?;
    }
    Ok(report)
}

/// Same as [`install`] but never writes its own JSON object to `out`. Used
/// when the caller (e.g. `but setup`) embeds the report in a larger JSON
/// envelope and wants to avoid duplicate top-level objects on stdout.
pub(crate) fn install_internal(
    out: &mut OutputChannel,
    current_dir: &std::path::Path,
    force: bool,
) -> Result<HookInstallReport> {
    let repo = open_repo(current_dir)?;
    let t = theme::get();

    if force {
        but_hooks::managed_hooks::set_install_managed_hooks_enabled(&repo, true)?;
    }

    let hooks_dir = but_hooks::managed_hooks::get_hooks_dir_gix(&repo);
    let outcome = but_hooks::managed_hooks::ensure_managed_hooks(
        &repo,
        &hooks_dir,
        force,
        std::time::SystemTime::now(),
    );

    let mut report = HookInstallReport::default();

    match outcome {
        but_hooks::managed_hooks::HookSetupOutcome::Installed { ref hook_results } => {
            report.hooks_installed = true;
            if let Some(out) = out.for_human() {
                for (name, backup_status) in hook_results {
                    writeln!(
                        out,
                        "  {}",
                        t.success
                            .add_modifier(Modifier::DIM)
                            .paint(backup_status.format_install_line(name))
                    )?;
                }
            }
            report.hooks.extend(hook_results_to_hook_info(hook_results));
        }
        but_hooks::managed_hooks::HookSetupOutcome::AlreadyInstalled => {
            report.hooks_installed = true;
            if let Some(out) = out.for_human() {
                for hook_name in but_hooks::hook_manager::MANAGED_HOOK_NAMES {
                    writeln!(
                        out,
                        "  {}",
                        t.success.paint(format!("✓ {hook_name} (already managed)"))
                    )?;
                }
            }
            report.hooks.extend(already_configured_hook_info());
        }
        but_hooks::managed_hooks::HookSetupOutcome::PartialSuccess {
            ref installed_hooks,
            ref hook_results,
            ref warnings,
        } => {
            report.hooks_installed = true;
            if let Some(out) = out.for_human() {
                for name in installed_hooks {
                    writeln!(out, "  {}", t.success.paint(format!("✓ Installed {name}")))?;
                }
                for w in warnings {
                    writeln!(out, "  {}", t.attention.paint(format!("Warning: {w}")))?;
                }
            }
            // Populate installed hooks with real backup data from hook_results.
            report.hooks.extend(hook_results_to_hook_info(hook_results));
            // Populate skipped hooks (those in MANAGED_HOOK_NAMES but not in hook_results).
            let installed_names: std::collections::HashSet<&str> =
                hook_results.iter().map(|(n, _)| n.as_str()).collect();
            report.hooks.extend(
                but_hooks::hook_manager::MANAGED_HOOK_NAMES
                    .iter()
                    .filter(|name| !installed_names.contains(*name))
                    .map(|name| HookInfo {
                        name: name.to_string(),
                        status: HookStatus::Skipped,
                    }),
            );
        }
        but_hooks::managed_hooks::HookSetupOutcome::DisabledByConfig => {
            // Previously configured opt-out (e.g. via `but setup --no-hooks`).
            if let Some(out) = out.for_human() {
                writeln!(
                    out,
                    "  {}",
                    t.hint.paint(
                        "Skipping hook installation (--no-hooks is configured for this repository)"
                    )
                )?;
                writeln!(out, "  {}", t.hint.paint(FORCE_HOOKS_RECOVERY_HINT))?;
            }
        }
        but_hooks::managed_hooks::HookSetupOutcome::ExternalManagerDetected {
            ref manager_name,
            instructions,
        } => {
            report.hook_manager = Some(HookManagerInfo {
                name: manager_name.clone(),
                hooks_installed: false,
            });
            if let Some(out) = out.for_human() {
                writeln!(out)?;
                writeln!(
                    out,
                    "  {}",
                    t.info
                        .paint(format!("Detected {manager_name} managing your git hooks."))
                )?;
                writeln!(
                    out,
                    "  {}",
                    t.hint
                        .paint("GitButler will not overwrite hooks owned by your hook manager.")
                )?;
                writeln!(
                    out,
                    "  {}",
                    t.hint
                        .paint("This repository is now configured for externally-managed hooks.")
                )?;
                writeln!(out)?;
                writeln!(
                    out,
                    "  {}",
                    t.attention
                        .paint("To integrate GitButler's workspace guard with your hook manager:")
                )?;
                for line in instructions.lines() {
                    if line.is_empty() {
                        writeln!(out)?;
                    } else {
                        writeln!(out, "  {}", t.hint.paint(line))?;
                    }
                }
                writeln!(out)?;
                writeln!(out, "  {}", t.hint.paint(FORCE_HOOKS_RECOVERY_HINT))?;
                writeln!(out)?;
            }
        }
        but_hooks::managed_hooks::HookSetupOutcome::HookSkipped { ref hook_names } => {
            if let Some(out) = out.for_human() {
                writeln!(
                    out,
                    "  {}",
                    t.attention.paint(format!(
                        "Warning: Skipped {} — hook(s) exist and are not GitButler-managed. \
                         Use --force-hooks to override.",
                        hook_names.join(", ")
                    ))
                )?;
            }
            report.hooks.extend(hook_names.iter().map(|name| HookInfo {
                name: name.clone(),
                status: HookStatus::Skipped,
            }));
        }
        but_hooks::managed_hooks::HookSetupOutcome::Failed { ref error } => {
            if let Some(out) = out.for_human() {
                writeln!(
                    out,
                    "  {}",
                    t.attention.paint(format!(
                        "Warning: Failed to install GitButler managed hooks: {error}"
                    ))
                )?;
            }
        }
    }

    // Warn about orphaned hooks if core.hooksPath redirects to a different directory.
    // Only relevant when hooks were installed or attempted (not when an external
    // manager was detected, since those hooks are intentionally elsewhere).
    if matches!(
        outcome,
        but_hooks::managed_hooks::HookSetupOutcome::Installed { .. }
            | but_hooks::managed_hooks::HookSetupOutcome::AlreadyInstalled
            | but_hooks::managed_hooks::HookSetupOutcome::PartialSuccess { .. }
            | but_hooks::managed_hooks::HookSetupOutcome::HookSkipped { .. }
    ) {
        let default_hooks_dir = repo.git_dir().join("hooks");
        if hooks_dir != default_hooks_dir
            && but_hooks::managed_hooks::has_managed_hooks_in(&default_hooks_dir)
        {
            let warning = format!(
                "GitButler-managed hooks found in old hooks directory ({}).",
                default_hooks_dir.display()
            );
            report.warnings.push(warning.clone());
            if let Some(out) = out.for_human() {
                writeln!(
                    out,
                    "  {}",
                    t.attention.paint(format!("Warning: {warning}"))
                )?;
                writeln!(
                    out,
                    "  {}",
                    t.hint.paint(format!(
                        "core.hooksPath is now set to {}, so those hooks are orphaned.",
                        hooks_dir.display()
                    ))
                )?;
                writeln!(
                    out,
                    "  {}",
                    t.hint.paint(format!(
                        "You can safely remove them with: {}",
                        but_hooks::hook_manager::orphaned_hooks_remove_command(&default_hooks_dir)
                    ))
                )?;
            }
        }
    }

    Ok(report)
}

/// Uninstall GitButler-managed git hooks.
///
/// Best-effort: signature-checks each managed hook before touching it, so
/// user/external hooks are left intact. Restores any pre-existing user hook
/// from `<hook>.gitbutler-backup`. After cleanup, detects whether an external
/// hook manager (prek, lefthook, husky) is now responsible for the hooks
/// directory and prints a hint if so.
pub fn uninstall(
    out: &mut OutputChannel,
    current_dir: &std::path::Path,
) -> Result<HookUninstallReport> {
    let report = uninstall_internal(out, current_dir)?;
    if let Some(json_out) = out.for_json() {
        json_out.write_value(&report)?;
    }
    Ok(report)
}

/// Same as [`uninstall`] but never writes its own JSON object to `out`. Used
/// when the caller (e.g. `but teardown`) embeds the report in a larger JSON
/// envelope.
pub(crate) fn uninstall_internal(
    out: &mut OutputChannel,
    current_dir: &std::path::Path,
) -> Result<HookUninstallReport> {
    let mut report = HookUninstallReport::default();

    let repo = match open_repo(current_dir) {
        Ok(r) => r,
        Err(_) => return Ok(report),
    };

    let hooks_dir = but_hooks::managed_hooks::get_hooks_dir_gix(&repo);
    let t = theme::get();

    match but_hooks::managed_hooks::uninstall_managed_hooks(&hooks_dir) {
        Ok(summary) => {
            let hook_output_printed = !summary.removed.is_empty()
                || !summary.restored.is_empty()
                || !summary.warnings.is_empty();
            if let Some(out) = out.for_human() {
                for name in &summary.removed {
                    writeln!(out, "  {}", t.success.paint(format!("✓ Removed {name}")))?;
                }
                for name in &summary.restored {
                    writeln!(
                        out,
                        "  {}",
                        t.success
                            .paint(format!("✓ Restored {name} (your original hook is back)"))
                    )?;
                }
                for w in &summary.warnings {
                    writeln!(out, "  {}", t.attention.paint(format!("Warning: {w}")))?;
                }
                if hook_output_printed {
                    writeln!(out)?;
                }
            }
            report.removed = summary.removed;
            report.restored = summary.restored;
            report.warnings = summary.warnings;

            // Only show the "externally managed" message when an external hook
            // manager is actually present in the hooks dir. Checking only the
            // config flag is misleading: it can be `false` after `--no-hooks` or a
            // manual edit even when no external manager exists.
            let workdir = repo
                .workdir()
                .unwrap_or_else(|| repo.git_dir())
                .to_path_buf();
            if let Some((manager_name, _)) =
                but_hooks::hook_manager::detect_hook_manager_in_hooks_dir(&hooks_dir, &workdir)
            {
                report.external_manager = Some(manager_name.to_owned());
                if let Some(out) = out.for_human() {
                    writeln!(
                        out,
                        "  {}",
                        t.hint.paint(format!(
                            "Hooks are managed by {manager_name} — leaving them untouched."
                        ))
                    )?;
                    writeln!(
                        out,
                        "  {}",
                        t.hint.paint(format!(
                            "If you wired 'but hook' commands into {manager_name}, remove those \
                             entries to fully disable GitButler's workspace guards."
                        ))
                    )?;
                    writeln!(out)?;
                }
            }
        }
        Err(e) => {
            let msg = format!("Failed to uninstall Git hooks: {e}");
            if let Some(out) = out.for_human() {
                writeln!(out, "  {}", t.attention.paint(format!("Warning: {msg}")))?;
            }
            report.warnings.push(msg);
        }
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_missing_description_covers_all_managed_hooks() {
        assert!(hook_missing_description("pre-commit").contains("workspace guard"));
        assert!(hook_missing_description("post-checkout").contains("workspace cleanup"));
        assert!(hook_missing_description("pre-push").contains("push guard"));
        assert_eq!(
            hook_missing_description("unknown-hook"),
            "GitButler hook won't run"
        );
    }
}
