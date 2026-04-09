use bstr::BStr;
use but_core::commit::message_is_conflicted;
use but_core::{ChangeId, commit::Headers};

/// Extension trait for `gix::Commit`.
///
/// For now, it collects useful methods from `gitbutler-core::git::Commit`
pub trait CommitExt {
    fn change_id(&self) -> Option<ChangeId>;
    fn is_signed(&self) -> bool;
    fn is_conflicted(&self) -> bool;
}

pub trait CommitMessageBstr {
    /// Obtain the commit-message as bytes, but without assuming any encoding.
    fn message_bstr(&self) -> &BStr;
}

impl CommitExt for gix::Commit<'_> {
    fn change_id(&self) -> Option<ChangeId> {
        let commit = self.decode().ok()?;
        Headers::try_from_commit_headers(|| commit.extra_headers())?.change_id
    }

    fn is_signed(&self) -> bool {
        self.decode()
            .is_ok_and(|decoded| decoded.extra_headers().pgp_signature().is_some())
    }

    fn is_conflicted(&self) -> bool {
        // Check commit message first (new style), fall back to header (legacy).
        if let Ok(commit) = self.decode() {
            if message_is_conflicted(commit.message) {
                return true;
            }
            Headers::try_from_commit_headers(|| commit.extra_headers())
                .is_some_and(|hdr| hdr.is_conflicted())
        } else {
            false
        }
    }
}

impl CommitMessageBstr for gix::Commit<'_> {
    fn message_bstr(&self) -> &BStr {
        self.message_raw()
            .expect("valid commit that can be parsed: TODO - allow it to return errors?")
    }
}
