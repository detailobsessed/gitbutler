use crate::utils::{CommandExt as _, Sandbox};

const MANAGED_HOOK_NAMES: [&str; 3] = ["pre-commit", "post-checkout", "pre-push"];

fn assert_hooks_installed(env: &Sandbox) {
    let hooks_dir = env.projects_root().join(".git/hooks");
    for name in MANAGED_HOOK_NAMES {
        let path = hooks_dir.join(name);
        assert!(path.exists(), "{name} should exist before uninstall");
        let content = std::fs::read_to_string(&path).expect("read hook");
        assert!(
            content.contains("GITBUTLER_MANAGED_HOOK_V1"),
            "{name} should be GitButler-managed before uninstall"
        );
    }
}

fn assert_managed_hooks_gone(env: &Sandbox) {
    let hooks_dir = env.projects_root().join(".git/hooks");
    for name in MANAGED_HOOK_NAMES {
        let path = hooks_dir.join(name);
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            assert!(
                !content.contains("GITBUTLER_MANAGED_HOOK_V1"),
                "{name} should no longer be GitButler-managed after uninstall"
            );
        }
    }
}

#[test]
fn uninstall_removes_signed_hooks() -> anyhow::Result<()> {
    let env = Sandbox::open_with_default_settings("repo-no-remote")?;

    // Set up managed hooks via the install command.
    env.but("hook install").assert().success();
    assert_hooks_installed(&env);

    // Uninstall removes them.
    env.but("hook uninstall").assert().success();
    assert_managed_hooks_gone(&env);

    Ok(())
}

#[test]
fn uninstall_leaves_user_hooks_alone() -> anyhow::Result<()> {
    let env = Sandbox::open_with_default_settings("repo-no-remote")?;

    // Plain user hook (no signature). Uninstall must leave it intact.
    env.invoke_bash(
        "mkdir -p .git/hooks && \
         printf '#!/bin/sh\\necho user hook\\n' > .git/hooks/pre-commit && \
         chmod +x .git/hooks/pre-commit",
    );

    env.but("hook uninstall").assert().success();

    let pre_commit = env.projects_root().join(".git/hooks/pre-commit");
    let content = std::fs::read_to_string(&pre_commit)?;
    assert!(content.contains("user hook"));
    assert!(!content.contains("GITBUTLER_MANAGED_HOOK_V1"));

    Ok(())
}

#[test]
fn uninstall_is_a_noop_when_no_managed_hooks_exist() -> anyhow::Result<()> {
    let env = Sandbox::open_with_default_settings("repo-no-remote")?;

    env.but("hook uninstall").assert().success();
    assert_managed_hooks_gone(&env);

    Ok(())
}

#[test]
fn uninstall_emits_json_report() -> anyhow::Result<()> {
    let env = Sandbox::open_with_default_settings("repo-no-remote")?;

    env.but("hook install").assert().success();

    let output = env.but("--json hook uninstall").allow_json().output()?;
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;

    // Top-level shape: { removed, restored, warnings, externalManager }
    assert!(json["removed"].is_array(), "removed should be an array");
    assert!(json["restored"].is_array(), "restored should be an array");
    assert!(json["warnings"].is_array(), "warnings should be an array");
    assert!(
        json["externalManager"].is_null(),
        "externalManager should be null when none detected"
    );

    let removed = json["removed"].as_array().unwrap();
    assert_eq!(
        removed.len(),
        3,
        "expected 3 removed hooks (pre-commit, post-checkout, pre-push)"
    );

    Ok(())
}
