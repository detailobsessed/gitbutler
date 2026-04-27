pub mod rebase;

mod commands;
mod traversal;

pub use traversal::{
    commit_ids_excluding_reachable_from_with_graph, first_parent_commit_ids_until,
    first_parent_commit_ids_until_with_graph,
};

pub use commands::{FileInfo, RepoCommands};
pub use remote::GitRemote;

mod repository_ext;
pub use repository_ext::{commit_with_signature_gix, commit_without_signature_gix};

pub use but_hooks::hook_manager;
pub mod hooks;
pub use but_hooks::managed_hooks;
mod remote;
pub mod staging;

pub mod commit_message;

pub const GITBUTLER_COMMIT_AUTHOR_NAME: &str = "GitButler";
pub const GITBUTLER_COMMIT_AUTHOR_EMAIL: &str = "gitbutler@gitbutler.com";

pub enum SignaturePurpose {
    Author,
    Committer,
}

/// Provide a `gix` signature with the GitButler author and the current or overridden time.
pub fn signature_gix(purpose: SignaturePurpose) -> gix::actor::Signature {
    gix::actor::Signature {
        name: GITBUTLER_COMMIT_AUTHOR_NAME.into(),
        email: GITBUTLER_COMMIT_AUTHOR_EMAIL.into(),
        time: but_core::commit_time(match purpose {
            SignaturePurpose::Author => "GIT_AUTHOR_DATE",
            SignaturePurpose::Committer => "GIT_COMMITTER_DATE",
        }),
    }
}
