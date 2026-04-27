use std::sync::Arc;

use bstr::BString;
use but_core::HunkHeader;
use gitbutler_stack::StackId;
use ratatui::style::Color;
use ratatui_textarea::TextArea;

use crate::{
    CliId,
    command::legacy::status::tui::MessageOnDrop,
    id::{ShortId, UncommittedCliId},
    theme::Theme,
};

#[derive(Debug, Default, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::EnumIter, Hash))]
#[strum_discriminants(name(ModeDiscriminant))]
pub(super) enum Mode {
    #[default]
    Normal,
    Rub(RubMode),
    InlineReword(InlineRewordMode),
    Command(CommandMode),
    Commit(CommitMode),
    Move(MoveMode),
    Details,
}

impl Mode {
    pub(super) fn bg(&self, theme: &'static Theme) -> Color {
        match self {
            Mode::Normal => theme.tui_mode_normal.bg.unwrap_or(Color::DarkGray),
            Mode::Commit(_) => theme.tui_mode_commit.bg.unwrap_or(Color::Green),
            Mode::Rub(_) => theme.tui_mode_rub.bg.unwrap_or(Color::Blue),
            Mode::InlineReword(_) => theme.tui_mode_inline_reword.bg.unwrap_or(Color::Magenta),
            Mode::Command(_) => theme.tui_mode_command.bg.unwrap_or(Color::Yellow),
            Mode::Move(..) => theme.tui_mode_move.bg.unwrap_or(Color::Cyan),
            Mode::Details => theme
                .tui_mode_details
                .bg
                .unwrap_or(Color::Rgb(255, 165, 0) /* orange */),
        }
    }

    pub(super) fn fg(&self, theme: &'static Theme) -> Color {
        match self {
            Mode::Normal => theme.tui_mode_normal.fg.unwrap_or(Color::White),
            Mode::Commit(_) => theme.tui_mode_commit.fg.unwrap_or(Color::Black),
            Mode::Rub(_) => theme.tui_mode_rub.fg.unwrap_or(Color::Black),
            Mode::InlineReword(_) => theme.tui_mode_inline_reword.fg.unwrap_or(Color::Black),
            Mode::Command(_) => theme.tui_mode_command.fg.unwrap_or(Color::Black),
            Mode::Move(..) => theme.tui_mode_move.fg.unwrap_or(Color::Black),
            Mode::Details => theme.tui_mode_details.fg.unwrap_or(Color::Black),
        }
    }
}

#[derive(Debug)]
pub(super) struct RubMode {
    pub(super) source: RubSource,
    pub(super) available_targets: Vec<Arc<CliId>>,
    pub(super) _unlock_details: Option<MessageOnDrop>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) enum RubSource {
    CliId(Arc<CliId>),
    CommittedHunk(CommittedHunk),
}

impl PartialEq<CliId> for RubSource {
    fn eq(&self, other: &CliId) -> bool {
        match self {
            RubSource::CliId(source) => &**source == other,
            RubSource::CommittedHunk { .. } => false,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct CommittedHunk {
    pub(super) commit_id: gix::ObjectId,
    pub(super) header: HunkHeader,
    pub(super) path: Arc<BString>,
}

#[derive(Debug)]
pub(super) enum InlineRewordMode {
    Commit {
        commit_id: gix::ObjectId,
        textarea: Box<TextArea<'static>>,
    },
    Branch {
        name: String,
        stack_id: StackId,
        textarea: Box<TextArea<'static>>,
    },
}

impl InlineRewordMode {
    pub(super) fn textarea(&self) -> &TextArea<'static> {
        match self {
            InlineRewordMode::Commit { textarea, .. }
            | InlineRewordMode::Branch { textarea, .. } => textarea,
        }
    }

    pub(super) fn textarea_mut(&mut self) -> &mut TextArea<'static> {
        match self {
            InlineRewordMode::Commit { textarea, .. }
            | InlineRewordMode::Branch { textarea, .. } => textarea,
        }
    }
}

#[derive(Debug)]
pub(super) struct CommandMode {
    pub(super) textarea: Box<TextArea<'static>>,
    pub(super) kind: CommandModeKind,
}

#[derive(Debug, Copy, Clone)]
pub(super) enum CommandModeKind {
    But,
    Shell,
}

#[derive(Debug)]
pub(super) struct CommitMode {
    pub(super) source: Arc<CommitSource>,
    /// If set, then the commit must be made on this stack
    ///
    /// Used when committing changes staged to a specific stack
    pub(super) scope_to_stack: Option<StackId>,
    /// How to compose the commit message.
    pub(super) message_composer: CommitMessageComposer,
}

/// A subset of [`CliId`] that supports being committed
#[derive(Debug)]
pub(super) enum CommitSource {
    Unassigned(UnassignedCommitSource),
    Uncommitted(Box<UncommittedCliId>),
    Stack(StackCommitSource),
}

#[derive(Debug, Copy, Clone, Default)]
pub(super) enum CommitMessageComposer {
    /// Open an editor to compose the commit message.
    #[default]
    Editor,
    /// Use an inline editor to compose the commit message.
    Inline,
    /// Create the commit with an empty message.
    Empty,
}

#[derive(Debug)]
pub(super) struct UnassignedCommitSource {
    pub(super) id: ShortId,
}

#[derive(Debug)]
pub(super) struct StackCommitSource {
    pub(super) stack_id: StackId,
}

impl CommitSource {
    pub(super) fn try_new(id: CliId) -> Option<Self> {
        match id {
            CliId::Unassigned { id } => Some(Self::Unassigned(UnassignedCommitSource { id })),
            CliId::Uncommitted(uncommitted_cli_id) => {
                Some(Self::Uncommitted(Box::new(uncommitted_cli_id)))
            }
            CliId::Stack { stack_id, .. } => Some(Self::Stack(StackCommitSource { stack_id })),
            CliId::PathPrefix { .. }
            | CliId::CommittedFile { .. }
            | CliId::Branch { .. }
            | CliId::Commit { .. } => None,
        }
    }
}

impl PartialEq<CliId> for CommitSource {
    fn eq(&self, other: &CliId) -> bool {
        match self {
            CommitSource::Unassigned(UnassignedCommitSource { id: lhs_id }) => {
                if let CliId::Unassigned { id: rhs_id } = other {
                    lhs_id == rhs_id
                } else {
                    false
                }
            }
            CommitSource::Uncommitted(lhs) => {
                if let CliId::Uncommitted(rhs) = other {
                    &**lhs == rhs
                } else {
                    false
                }
            }
            CommitSource::Stack(StackCommitSource {
                stack_id: stack_id_lhs,
            }) => {
                if let CliId::Stack {
                    stack_id: stack_id_rhs,
                    ..
                } = other
                {
                    stack_id_lhs == stack_id_rhs
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug)]
pub(super) struct MoveMode {
    pub(super) source: Arc<MoveSource>,
}

/// A subset of [`CliId`] that supports being moved
#[derive(Debug)]
pub(super) enum MoveSource {
    Commit {
        commit_id: gix::ObjectId,
        id: ShortId,
    },
    Branch {
        name: String,
        id: ShortId,
        stack_id: Option<StackId>,
    },
}

impl MoveSource {
    pub(super) fn is_commit(&self) -> bool {
        matches!(self, Self::Commit { .. })
    }
}

impl TryFrom<CliId> for MoveSource {
    type Error = anyhow::Error;

    fn try_from(id: CliId) -> Result<Self, Self::Error> {
        match id {
            CliId::Branch { name, id, stack_id } => Ok(Self::Branch { name, id, stack_id }),
            CliId::Commit { commit_id, id } => Ok(Self::Commit { commit_id, id }),
            CliId::Uncommitted(uncommitted_cli_id) => {
                anyhow::bail!("cannot move: {:?}", uncommitted_cli_id.id)
            }
            CliId::PathPrefix { id, .. }
            | CliId::CommittedFile { id, .. }
            | CliId::Unassigned { id }
            | CliId::Stack { id, .. } => {
                anyhow::bail!("cannot move: {id:?}")
            }
        }
    }
}

impl PartialEq<CliId> for MoveSource {
    fn eq(&self, other: &CliId) -> bool {
        match self {
            MoveSource::Commit {
                commit_id: commit_id_lhs,
                id: id_lhs,
            } => {
                if let CliId::Commit {
                    commit_id: commit_id_rhs,
                    id: id_rhs,
                } = other
                {
                    commit_id_lhs == commit_id_rhs && id_lhs == id_rhs
                } else {
                    false
                }
            }
            MoveSource::Branch {
                name: name_lhs,
                id: id_lhs,
                stack_id: stack_id_lhs,
            } => {
                if let CliId::Branch {
                    name: name_rhs,
                    id: id_rhs,
                    stack_id: stack_id_rhs,
                } = other
                {
                    name_lhs == name_rhs && id_lhs == id_rhs && stack_id_lhs == stack_id_rhs
                } else {
                    false
                }
            }
        }
    }
}
