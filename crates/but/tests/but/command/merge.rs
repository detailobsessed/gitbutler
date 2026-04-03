use bstr::{BString, ByteSlice as _};
use gix::{actor::Signature, date::Time};
use snapbox::str;

use crate::utils::{CommandExt, Sandbox};

#[test]
fn evo_merge_simple() -> anyhow::Result<()> {
    // Simple case: swap order of 2 commits

    let env = Sandbox::open_with_default_settings("merge-gb-local-two-branches")?;
    env.but("setup").assert().success();

    env.but("branch new remote-branch").assert().success();
    env.file("I.txt", "");
    env.but("commit remote-branch -m II").assert().success();
    env.file("J.txt", "");
    env.but("commit remote-branch -m JJ").assert().success();

    env.but("branch new local-branch").assert().success();
    env.file("Q.txt", "");
    env.but("commit local-branch -m QQ").assert().success();
    env.file("P.txt", "");
    env.but("commit local-branch -m PP").assert().success();

    insta::assert_snapshot!(env.git_log()?, @r"
    *   f4d6458 (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    |\  
    | * 096070c (local-branch) PP
    | * 64848c3 QQ
    * | 35c5451 (remote-branch) JJ
    * | 324d9c3 II
    |/  
    * 85efbe4 (gb-local/main, gb-local/HEAD, main, gitbutler/target) M
    ");

    // The graph is interpreted as chunks of 2 characters, parent->child
    // (e.g. AI means AA evolved into II; AA is a commit from some time before
    // and might have already been garbage collected).
    let output = env
        .but("merge --graph=AIAPBJBQ --local=local-branch --remote=remote-branch ''")
        .assert()
        .success()
        .stdout_eq(str![[r#"
910726e00b12d9aba380fa3c347232a28c354196

"#]])
        .get_output()
        .stdout
        .clone();
    let output = env.invoke_git(&format!("log --oneline --graph {}", output.as_bstr()));
    insta::assert_snapshot!(output, @"
    * 910726e merge remote JJ + local QQ
    * 073a8f4 merge remote II + local PP
    * 85efbe4 M
    ");

    Ok(())
}

fn commit<const N: usize>(
    repo: &gix::Repository,
    message: &str,
    parent_ids: [gix::ObjectId; N],
) -> anyhow::Result<gix::ObjectId> {
    let signature = Signature {
        name: BString::from("Someone"),
        email: BString::from("someone@example.com"),
        time: Time {
            seconds: 1675176957,
            offset: 0,
        },
    };
    let commit = gix::objs::Commit {
        tree: repo.empty_tree().id,
        parents: parent_ids.to_vec().into(),
        author: signature.clone(),
        committer: signature,
        encoding: None,
        message: BString::from(message),
        extra_headers: Vec::new(),
    };
    Ok(repo.write_object(commit)?.detach())
}

#[test]
fn evo_merge_complex() -> anyhow::Result<()> {
    // Complex case: non-linear on remote. One commit (A, not shown) was independently split
    // by both remote (into II, JJ) and local (into PP, QQ). Remote has one novel
    // commit (KK).

    let env = Sandbox::open_with_default_settings("merge-gb-local-two-branches")?;
    env.but("setup").assert().success();

    let repo = env.open_repo()?;
    let base = repo.rev_parse_single("main")?.detach();

    let ii = commit(&repo, "II", [base])?;
    let jj = commit(&repo, "JJ", [ii])?;
    let kk = commit(&repo, "KK", [ii])?;
    let ll = commit(&repo, "LL", [jj, kk])?;
    let output = env.invoke_git(&format!("log --oneline --graph {}", ll.to_hex()));
    insta::assert_snapshot!(output, @r"
    *   5895ebf LL
    |\  
    | * 9375d62 KK
    * | dda220e JJ
    |/  
    * 1ad0fa8 II
    * 85efbe4 M
    ");

    let ss = commit(&repo, "SS", [base])?;
    let pp = commit(&repo, "PP", [ss])?;
    let qq = commit(&repo, "QQ", [pp])?;
    let output = env.invoke_git(&format!("log --oneline --graph {}", qq.to_hex()));
    insta::assert_snapshot!(output, @"
    * 56bd561 QQ
    * de4c4e7 PP
    * a4ae5b7 SS
    * 85efbe4 M
    ");

    // II, JJ, PP, QQ have common progenitor (AA, not shown in commit graph).
    // LL, SS have common progenitor (DD, not shown in commit graph).
    let output = env
        .but(&format!(
            "merge --graph=AIAJAPAQDLDS --local={} --remote={} ''",
            qq.to_hex(),
            ll.to_hex()
        ))
        .assert()
        .success()
        .stdout_eq(str![[r#"
ef65b54a5409b216930cc1b3932f2ba3f16cb414

"#]])
        .get_output()
        .stdout
        .clone();
    let output = env.invoke_git(&format!("log --oneline --graph {}", output.as_bstr()));
    insta::assert_snapshot!(output, @r"
    *   ef65b54 merge remote LL + local SS
    |\  
    | * 2ccec8f KK
    * | d138883 merge remote JJ + local PP,QQ
    |/  
    * ad95fdc merge remote II + local PP,QQ
    * 85efbe4 M
    ");

    Ok(())
}

#[test]
fn evo_merge_disappearing_local_merge_rebases_children_onto_first_parent() -> anyhow::Result<()> {
    let env = Sandbox::open_with_default_settings("merge-gb-local-two-branches")?;
    env.but("setup").assert().success();

    let repo = env.open_repo()?;
    let base = repo.rev_parse_single("main")?.detach();

    let ii = commit(&repo, "II", [base])?;
    let jj = commit(&repo, "JJ", [ii])?;
    let kk = commit(&repo, "KK", [ii])?;
    let ll = commit(&repo, "LL", [jj, kk])?;
    let output = env.invoke_git(&format!("log --oneline --graph {}", ll.to_hex()));
    insta::assert_snapshot!(output, @r"
    *   5895ebf LL
    |\  
    | * 9375d62 KK
    * | dda220e JJ
    |/  
    * 1ad0fa8 II
    * 85efbe4 M
    ");

    let pp = commit(&repo, "PP", [base])?;
    let qq = commit(&repo, "QQ", [pp])?;
    let rr = commit(&repo, "RR", [pp])?;
    let tt = commit(&repo, "TT", [qq, rr])?;
    let uu = commit(&repo, "UU", [tt])?;
    let ss = commit(&repo, "SS", [uu])?;
    let output = env.invoke_git(&format!("log --oneline --graph {}", ss.to_hex()));
    insta::assert_snapshot!(output, @r"
    * 05969ac SS
    * 9a1f997 UU
    *   57193fc TT
    |\  
    | * feb2b3c RR
    * | a0a3f7f QQ
    |/  
    * 115a360 PP
    * 85efbe4 M
    ");

    let output = env
        .but(&format!(
            "merge --graph=AIAPBJBQBTCKCRDLDS --local={} --remote={} ''",
            ss.to_hex(),
            ll.to_hex()
        ))
        .assert()
        .success()
        .stdout_eq(str![[r#"
850c8312a80985e390d77f31bb7d582e7cd46bd5

"#]])
        .get_output()
        .stdout
        .clone();
    let output = env.invoke_git(&format!("log --oneline --graph {}", output.as_bstr()));
    insta::assert_snapshot!(output, @r"
    *   850c831 merge remote LL + local SS
    |\  
    | * b6b5916 merge remote KK + local RR
    * | b63528d UU
    * | 92a2454 merge remote JJ + local QQ,TT
    |/  
    * 9e12b54 merge remote II + local PP
    * 85efbe4 M
    ");

    Ok(())
}

#[test]
fn evo_merge_local_tips_before_remote_tips() -> anyhow::Result<()> {
    let env = Sandbox::open_with_default_settings("merge-gb-local-two-branches")?;
    env.but("setup").assert().success();

    let repo = env.open_repo()?;
    let base = repo.rev_parse_single("main")?.detach();

    let ii = commit(&repo, "II", [base])?;
    let jj = commit(&repo, "JJ", [ii])?;
    let output = env.invoke_git(&format!("log --oneline --graph {}", jj.to_hex()));
    insta::assert_snapshot!(output, @"
    * dda220e JJ
    * 1ad0fa8 II
    * 85efbe4 M
    ");

    let pp = commit(&repo, "PP", [base])?;
    let rr = commit(&repo, "RR", [pp])?;
    let output = env.invoke_git(&format!("log --oneline --graph {}", rr.to_hex()));
    insta::assert_snapshot!(output, @"
    * feb2b3c RR
    * 115a360 PP
    * 85efbe4 M
    ");

    let output = env
        .but(&format!(
            "merge --graph=AIAP --local={} --remote={} ''",
            rr.to_hex(),
            jj.to_hex()
        ))
        .assert()
        .success()
        .stdout_eq(str![[r#"
363345fc7def8ac0bc83bfd01a63fd5169ed7189

"#]])
        .get_output()
        .stdout
        .clone();
    let output = env.invoke_git(&format!("log --oneline --graph {}", output.as_bstr()));
    insta::assert_snapshot!(output, @"
    * 363345f JJ
    * 7e2498e RR
    * 9e12b54 merge remote II + local PP
    * 85efbe4 M
    ");

    Ok(())
}

#[test]
fn merge_first_branch_into_gb_local_and_verify_rebase() -> anyhow::Result<()> {
    let env = Sandbox::open_with_default_settings("merge-gb-local-two-branches")?;

    // Run setup to create gb-local remote
    env.but("setup").assert().success();

    // Verify we're on gitbutler/workspace
    let output = env.invoke_git("rev-parse --abbrev-ref HEAD");
    assert_eq!(output, "gitbutler/workspace");

    // Create first branch
    env.but("branch new first-branch").assert().success();

    // Create first commit on first branch
    env.file("file1.txt", "content1");
    env.but("commit first-branch -m 'first commit on branch A'")
        .assert()
        .success();

    let first_branch = "first-branch";

    // Create second branch with a different commit
    env.but("branch new second-branch").assert().success();

    env.file("file2.txt", "content2");
    env.but("commit second-branch -m 'second commit on branch B'")
        .assert()
        .success();

    // Verify git log shows both branches before merge
    insta::assert_snapshot!(env.git_log()?, @r"
    *   945f3cf (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    |\  
    | * edca1cd (second-branch) second commit on branch B
    * | 549e10c (first-branch) first commit on branch A
    |/  
    * 85efbe4 (gb-local/main, gb-local/HEAD, main, gitbutler/target) M
    ");

    // Get the current main branch commit (should be the initial commit M)
    let main_before_hash = env.invoke_git("rev-parse main");

    // Merge the first branch
    env.but(format!("merge {first_branch}"))
        .assert()
        .success()
        .stdout_eq(str![[r#"

Found 2 upstream commits on gb-local/main
   61888c9 Merge branch 'first-branch'
   549e10c first commit on branch A

Updating 2 active branches...

Rebase of second-branch successful

Branch first-branch has been integrated upstream and removed locally

Summary
────────
  second-branch - rebased
  first-branch - integrated

To undo this operation:
  Run `but undo`

"#]]);

    // Verify that main has been updated with the merge commit
    let main_after_hash = env.invoke_git("rev-parse main");

    // Main should have changed
    assert_ne!(
        main_before_hash, main_after_hash,
        "main branch should have been updated"
    );

    // Verify the merge commit has both parents
    let parents = env.invoke_git("rev-list --parents -n 1 main");
    let parent_count = parents.split_whitespace().count() - 1; // Subtract 1 for the commit itself
    assert_eq!(parent_count, 2, "Merge commit should have 2 parents");

    // Verify file1.txt exists on main now
    let file1_content = std::fs::read_to_string(env.projects_root().join("file1.txt"))?;
    assert_eq!(file1_content, "content1");

    // Verify that only the second branch remains in the workspace
    let status_after = env
        .but("status --json")
        .allow_json()
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status_after_str = String::from_utf8_lossy(&status_after);
    let status_after_json: serde_json::Value = serde_json::from_str(&status_after_str)?;

    // Should only have one stack now (second-branch)
    assert_eq!(
        status_after_json["stacks"].as_array().unwrap().len(),
        1,
        "Only second-branch should remain after merge"
    );

    // Verify the second branch is rebased on top of the updated main
    let second_branch_base_hash = env.invoke_git("merge-base main second-branch");

    // The merge base should be the new main (the second branch was rebased)
    assert_eq!(
        second_branch_base_hash, main_after_hash,
        "second-branch should be rebased on top of the merged main"
    );

    // Verify git log shows the rebased structure
    insta::assert_snapshot!(env.git_log()?, @r"
    * c7f0f9d (HEAD -> gitbutler/workspace) GitButler Workspace Commit
    * e8d7818 (second-branch) second commit on branch B
    *   61888c9 (gb-local/main, gb-local/HEAD, main) Merge branch 'first-branch'
    |\  
    | | * 945f3cf (gb-local/gitbutler/workspace) GitButler Workspace Commit
    | |/| 
    |/| | 
    | | * edca1cd (gb-local/second-branch) second commit on branch B
    | |/  
    * / 549e10c (gb-local/first-branch) first commit on branch A
    |/  
    * 85efbe4 (gb-local/gitbutler/target, gitbutler/target) M
    ");

    Ok(())
}
