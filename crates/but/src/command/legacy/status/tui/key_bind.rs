use std::{borrow::Cow, collections::HashMap};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use strum::IntoEnumIterator;

use crate::command::legacy::status::tui::{
    BranchPickerMessage, CommandMessage, CommitMessageComposer, ConfirmMessage, Message, Mode,
    RewordMessage, RubMessage, mode::ModeDiscriminant,
};

use super::{CommandModeKind, CommitMessage, DetailsMessage, FilesMessage, MoveMessage};

pub(super) fn default_key_binds() -> KeyBinds {
    let mut key_binds = KeyBinds::new();

    for mode in ModeDiscriminant::iter() {
        match mode {
            ModeDiscriminant::Normal => {
                register_global_key_binds(&mut key_binds, Vec::from([mode]));
                register_long_jump_key_binds(&mut key_binds, Vec::from([mode]));
                register_branch_picker_key_binds(&mut key_binds, Vec::from([mode]));
                register_normal_mode_key_binds(&mut key_binds);
            }
            ModeDiscriminant::Rub => {
                register_global_key_binds(&mut key_binds, Vec::from([mode]));
                register_long_jump_key_binds(&mut key_binds, Vec::from([mode]));
                register_branch_picker_key_binds(&mut key_binds, Vec::from([mode]));
                register_rub_mode_key_binds(&mut key_binds);
            }
            ModeDiscriminant::InlineReword => {
                register_inline_reword_mode_key_binds(&mut key_binds);
            }
            ModeDiscriminant::Command => {
                register_command_mode_key_binds(&mut key_binds);
            }
            ModeDiscriminant::Commit => {
                register_global_key_binds(&mut key_binds, Vec::from([mode]));
                register_long_jump_key_binds(&mut key_binds, Vec::from([mode]));
                register_branch_picker_key_binds(&mut key_binds, Vec::from([mode]));
                register_commit_mode_key_binds(&mut key_binds);
            }
            ModeDiscriminant::Move => {
                register_global_key_binds(&mut key_binds, Vec::from([mode]));
                register_long_jump_key_binds(&mut key_binds, Vec::from([mode]));
                register_branch_picker_key_binds(&mut key_binds, Vec::from([mode]));
                register_move_mode_key_binds(&mut key_binds);
            }
            ModeDiscriminant::Details => {
                register_detail_key_binds(&mut key_binds);
            }
        }
    }

    key_binds
}

pub(super) fn confirm_key_binds() -> KeyBinds {
    let mut key_binds = KeyBinds::new();

    let all_modes = ModeDiscriminant::iter().collect::<Vec<_>>();

    register_quit_key_binds(&mut key_binds, all_modes.clone());

    key_binds.register(KeyBindDef {
        short_description: "select",
        key_matcher: press().code(KeyCode::Enter),
        modes: all_modes.clone(),
        message: Message::Confirm(ConfirmMessage::Confirm),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "yes",
        key_matcher: press().code(KeyCode::Char('y')),
        modes: all_modes.clone(),
        message: Message::Confirm(ConfirmMessage::Yes),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "no",
        key_matcher: press().code(KeyCode::Char('n')).alt_code(KeyCode::Esc),
        modes: all_modes.clone(),
        message: Message::Confirm(ConfirmMessage::No),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "left",
        key_matcher: press().code(KeyCode::Char('h')).alt_code(KeyCode::Left),
        modes: all_modes.clone(),
        message: Message::Confirm(ConfirmMessage::Left),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "right",
        key_matcher: press().code(KeyCode::Char('l')).alt_code(KeyCode::Right),
        modes: all_modes.clone(),
        message: Message::Confirm(ConfirmMessage::Right),
        hide_from_hotbar: false,
    });

    key_binds
}

pub(super) fn branch_picker_key_binds() -> KeyBinds {
    let mut key_binds = KeyBinds::new();

    let all_modes = ModeDiscriminant::iter().collect::<Vec<_>>();

    key_binds.register(KeyBindDef {
        short_description: "down",
        key_matcher: press().alt_code(KeyCode::Down),
        modes: all_modes.clone(),
        message: Message::BranchPicker(BranchPickerMessage::MoveCursorDown),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "up",
        key_matcher: press().alt_code(KeyCode::Up),
        modes: all_modes.clone(),
        message: Message::BranchPicker(BranchPickerMessage::MoveCursorUp),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "down",
        key_matcher: press().control().code(KeyCode::Char('n')),
        modes: all_modes.clone(),
        message: Message::BranchPicker(BranchPickerMessage::MoveCursorDown),
        hide_from_hotbar: true,
    });

    key_binds.register(KeyBindDef {
        short_description: "up",
        key_matcher: press().control().code(KeyCode::Char('p')),
        modes: all_modes.clone(),
        message: Message::BranchPicker(BranchPickerMessage::MoveCursorUp),
        hide_from_hotbar: true,
    });

    key_binds.register(KeyBindDef {
        short_description: "confirm",
        key_matcher: press().code(KeyCode::Enter),
        modes: all_modes.clone(),
        message: Message::BranchPicker(BranchPickerMessage::Confirm),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "back",
        key_matcher: press().code(KeyCode::Esc),
        modes: all_modes.clone(),
        message: Message::BranchPicker(BranchPickerMessage::Close),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "back",
        key_matcher: press().control().code(KeyCode::Char('[')),
        modes: all_modes.clone(),
        message: Message::BranchPicker(BranchPickerMessage::Close),
        hide_from_hotbar: true,
    });

    key_binds
}

fn register_detail_key_binds(key_binds: &mut KeyBinds) {
    key_binds.register(KeyBindDef {
        short_description: "next hunk",
        key_matcher: press().code(KeyCode::Char('j')).alt_code(KeyCode::Down),
        modes: Vec::from([ModeDiscriminant::Details]),
        message: Message::Details(DetailsMessage::SelectNextSection),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "prev hunk",
        key_matcher: press().code(KeyCode::Char('k')).alt_code(KeyCode::Up),
        modes: Vec::from([ModeDiscriminant::Details]),
        message: Message::Details(DetailsMessage::SelectPrevSection),
        hide_from_hotbar: false,
    });

    let scroll_distance = 5;

    key_binds.register(KeyBindDef {
        short_description: "scroll down",
        key_matcher: press().shift().code(KeyCode::Char('J')),
        modes: Vec::from([ModeDiscriminant::Details]),
        message: Message::Details(DetailsMessage::ScrollDown(scroll_distance)),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "scroll up",
        key_matcher: press().shift().code(KeyCode::Char('K')),
        modes: Vec::from([ModeDiscriminant::Details]),
        message: Message::Details(DetailsMessage::ScrollUp(scroll_distance)),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "rub",
        key_matcher: press().code(KeyCode::Char('r')),
        modes: Vec::from([ModeDiscriminant::Details]),
        message: Message::Details(DetailsMessage::StartRub),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "focus status",
        key_matcher: press().code(KeyCode::Char('h')),
        modes: Vec::from([ModeDiscriminant::Details]),
        message: Message::EnterNormalMode,
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "top",
        key_matcher: press().code(KeyCode::Char('g')),
        modes: Vec::from([ModeDiscriminant::Details]),
        message: Message::Details(DetailsMessage::GotoTop),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "bottom",
        key_matcher: press().shift().code(KeyCode::Char('G')),
        modes: Vec::from([ModeDiscriminant::Details]),
        message: Message::Details(DetailsMessage::GotoBottom),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "hide details",
        key_matcher: press().code(KeyCode::Char('d')),
        modes: Vec::from([ModeDiscriminant::Details]),
        message: Message::Details(DetailsMessage::ToggleVisibility),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "command",
        key_matcher: press().code(KeyCode::Char(':')),
        modes: Vec::from([ModeDiscriminant::Details]),
        message: Message::Command(CommandMessage::Start(CommandModeKind::But)),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "shell command",
        key_matcher: press().code(KeyCode::Char('!')),
        modes: Vec::from([ModeDiscriminant::Details]),
        message: Message::Command(CommandMessage::Start(CommandModeKind::Shell)),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "normal mode",
        key_matcher: press().control().code(KeyCode::Char('[')),
        modes: Vec::from([ModeDiscriminant::Details]),
        message: Message::EnterNormalMode,
        hide_from_hotbar: true,
    });

    register_grow_shrink_details_key_binds(key_binds, Vec::from([ModeDiscriminant::Details]));

    register_quit_key_binds(key_binds, Vec::from([ModeDiscriminant::Details]));
}

fn register_global_key_binds(key_binds: &mut KeyBinds, modes: Vec<ModeDiscriminant>) {
    key_binds.register(KeyBindDef {
        short_description: "down",
        key_matcher: press().code(KeyCode::Char('j')).alt_code(KeyCode::Down),
        modes: modes.clone(),
        message: Message::MoveCursorDown,
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "up",
        key_matcher: press().code(KeyCode::Char('k')).alt_code(KeyCode::Up),
        modes: modes.clone(),
        message: Message::MoveCursorUp,
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "next section",
        key_matcher: press().shift().code(KeyCode::Char('J')),
        modes: modes.clone(),
        message: Message::MoveCursorNextSection,
        hide_from_hotbar: true,
    });

    key_binds.register(KeyBindDef {
        short_description: "prev section",
        key_matcher: press().shift().code(KeyCode::Char('K')),
        modes: modes.clone(),
        message: Message::MoveCursorPreviousSection,
        hide_from_hotbar: true,
    });

    register_quit_key_binds(key_binds, modes.clone());

    key_binds.register(KeyBindDef {
        short_description: "normal mode",
        key_matcher: press().control().code(KeyCode::Char('[')),
        modes: modes.clone(),
        message: Message::EnterNormalMode,
        hide_from_hotbar: true,
    });

    key_binds.register(KeyBindDef {
        short_description: "details",
        key_matcher: press().code(KeyCode::Char('d')),
        modes: modes.clone(),
        message: Message::Details(DetailsMessage::ToggleVisibility),
        hide_from_hotbar: false,
    });

    register_grow_shrink_details_key_binds(key_binds, modes);
}

fn register_long_jump_key_binds(key_binds: &mut KeyBinds, modes: Vec<ModeDiscriminant>) {
    key_binds.register(KeyBindDef {
        short_description: "unassigned",
        key_matcher: press()
            .code(KeyCode::Char('z'))
            .alt_code(KeyCode::Char('g')),
        modes: modes.clone(),
        message: Message::SelectUnassigned,
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "merge base",
        key_matcher: press().shift().code(KeyCode::Char('G')),
        modes: modes.clone(),
        message: Message::SelectMergeBase,
        hide_from_hotbar: true,
    });
}

fn register_branch_picker_key_binds(key_binds: &mut KeyBinds, modes: Vec<ModeDiscriminant>) {
    key_binds.register(KeyBindDef {
        short_description: "goto branch",
        key_matcher: press().code(KeyCode::Char('t')),
        modes: modes.clone(),
        message: Message::PickAndGotoBranch,
        hide_from_hotbar: false,
    });
}

fn register_grow_shrink_details_key_binds(key_binds: &mut KeyBinds, modes: Vec<ModeDiscriminant>) {
    key_binds.register(KeyBindDef {
        short_description: "grow details",
        key_matcher: press().code(KeyCode::Char('+')),
        modes: modes.clone(),
        message: Message::GrowDetails,
        hide_from_hotbar: true,
    });

    key_binds.register(KeyBindDef {
        short_description: "shrink details",
        key_matcher: press().code(KeyCode::Char('-')),
        modes: modes.clone(),
        message: Message::ShrinkDetails,
        hide_from_hotbar: true,
    });
}

fn register_quit_key_binds(key_binds: &mut KeyBinds, modes: Vec<ModeDiscriminant>) {
    key_binds.register(KeyBindDef {
        short_description: "quit",
        key_matcher: press().code(KeyCode::Char('q')),
        modes,
        message: Message::Quit,
        hide_from_hotbar: false,
    });
}

fn register_normal_mode_key_binds(key_binds: &mut KeyBinds) {
    key_binds.register(KeyBindDef {
        short_description: "rub",
        key_matcher: press().code(KeyCode::Char('r')),
        modes: Vec::from([ModeDiscriminant::Normal]),
        message: Message::Rub(RubMessage::Start),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "reverse rub",
        key_matcher: press().shift().code(KeyCode::Char('R')),
        modes: Vec::from([ModeDiscriminant::Normal]),
        message: Message::Rub(RubMessage::StartReverse),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "commit",
        key_matcher: press().code(KeyCode::Char('c')),
        modes: Vec::from([ModeDiscriminant::Normal]),
        message: Message::Commit(CommitMessage::Start),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "new commit",
        key_matcher: press().code(KeyCode::Char('n')),
        modes: Vec::from([ModeDiscriminant::Normal]),
        message: Message::Commit(CommitMessage::CreateEmpty),
        hide_from_hotbar: true,
    });

    key_binds.register(KeyBindDef {
        short_description: "move",
        key_matcher: press().code(KeyCode::Char('m')),
        modes: Vec::from([ModeDiscriminant::Normal]),
        message: Message::Move(MoveMessage::Start),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "branch",
        key_matcher: press().code(KeyCode::Char('b')),
        modes: Vec::from([ModeDiscriminant::Normal]),
        message: Message::NewBranch,
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "focus details",
        key_matcher: press().code(KeyCode::Char('l')),
        modes: Vec::from([ModeDiscriminant::Normal]),
        message: Message::EnterDetailsMode,
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "reword inline",
        key_matcher: press().code(KeyCode::Enter),
        modes: Vec::from([ModeDiscriminant::Normal]),
        message: Message::Reword(RewordMessage::InlineStart),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "reword",
        key_matcher: press().shift().code(KeyCode::Char('M')),
        modes: Vec::from([ModeDiscriminant::Normal]),
        message: Message::Reword(RewordMessage::WithEditor),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "files",
        key_matcher: press().code(KeyCode::Char('f')),
        modes: Vec::from([ModeDiscriminant::Normal]),
        message: Message::Files(FilesMessage::ToggleFilesForCommit),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "show all files",
        key_matcher: press().shift().code(KeyCode::Char('F')),
        modes: Vec::from([ModeDiscriminant::Normal]),
        message: Message::Files(FilesMessage::ToggleGlobalFilesList),
        hide_from_hotbar: true,
    });

    key_binds.register(KeyBindDef {
        short_description: "discard",
        key_matcher: press().code(KeyCode::Char('x')),
        modes: Vec::from([ModeDiscriminant::Normal]),
        message: Message::Discard,
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "command",
        key_matcher: press().code(KeyCode::Char(':')),
        modes: Vec::from([ModeDiscriminant::Normal]),
        message: Message::Command(CommandMessage::Start(CommandModeKind::But)),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "shell command",
        key_matcher: press().code(KeyCode::Char('!')),
        modes: Vec::from([ModeDiscriminant::Normal]),
        message: Message::Command(CommandMessage::Start(CommandModeKind::Shell)),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "reload",
        key_matcher: press().control().code(KeyCode::Char('r')),
        modes: Vec::from([ModeDiscriminant::Normal]),
        message: Message::Reload(None),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "copy",
        key_matcher: press().shift().code(KeyCode::Char('C')),
        modes: Vec::from([ModeDiscriminant::Normal]),
        message: Message::CopySelection,
        hide_from_hotbar: true,
    });

    key_binds.register(KeyBindDef {
        short_description: "back",
        key_matcher: press().code(KeyCode::Esc),
        modes: Vec::from([ModeDiscriminant::Normal]),
        message: Message::EnterNormalMode,
        hide_from_hotbar: true,
    });
}

fn register_rub_mode_key_binds(key_binds: &mut KeyBinds) {
    key_binds.register(KeyBindDef {
        short_description: "confirm",
        key_matcher: press().code(KeyCode::Enter),
        modes: Vec::from([ModeDiscriminant::Rub]),
        message: Message::Rub(RubMessage::Confirm),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "back",
        key_matcher: press().code(KeyCode::Esc),
        modes: Vec::from([ModeDiscriminant::Rub]),
        message: Message::EnterNormalMode,
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "back",
        key_matcher: press().code(KeyCode::Char('r')),
        modes: Vec::from([ModeDiscriminant::Rub]),
        message: Message::EnterNormalMode,
        hide_from_hotbar: true,
    });
}

fn register_inline_reword_mode_key_binds(key_binds: &mut KeyBinds) {
    key_binds.register(KeyBindDef {
        short_description: "confirm",
        key_matcher: press().code(KeyCode::Enter),
        modes: Vec::from([ModeDiscriminant::InlineReword]),
        message: Message::Reword(RewordMessage::InlineConfirm),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "open editor",
        key_matcher: press().alt().code(KeyCode::Char('e')),
        modes: Vec::from([ModeDiscriminant::InlineReword]),
        message: Message::Reword(RewordMessage::OpenEditor),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "back",
        key_matcher: press().code(KeyCode::Esc),
        modes: Vec::from([ModeDiscriminant::InlineReword]),
        message: Message::EnterNormalMode,
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "normal mode",
        key_matcher: press().control().code(KeyCode::Char('[')),
        modes: Vec::from([ModeDiscriminant::InlineReword]),
        message: Message::EnterNormalMode,
        hide_from_hotbar: true,
    });
}

fn register_command_mode_key_binds(key_binds: &mut KeyBinds) {
    key_binds.register(KeyBindDef {
        short_description: "run",
        key_matcher: press().code(KeyCode::Enter),
        modes: Vec::from([ModeDiscriminant::Command]),
        message: Message::Command(CommandMessage::Confirm),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "back",
        key_matcher: press().code(KeyCode::Esc),
        modes: Vec::from([ModeDiscriminant::Command]),
        message: Message::EnterNormalMode,
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "normal mode",
        key_matcher: press().control().code(KeyCode::Char('[')),
        modes: Vec::from([ModeDiscriminant::Command]),
        message: Message::EnterNormalMode,
        hide_from_hotbar: true,
    });
}

fn register_commit_mode_key_binds(key_binds: &mut KeyBinds) {
    key_binds.register(KeyBindDef {
        short_description: "commit",
        key_matcher: press().code(KeyCode::Enter),
        modes: Vec::from([ModeDiscriminant::Commit]),
        message: Message::Commit(CommitMessage::Confirm),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "empty message",
        key_matcher: press().code(KeyCode::Char('e')),
        modes: Vec::from([ModeDiscriminant::Commit]),
        message: Message::Commit(CommitMessage::ToggleMessageComposer(
            CommitMessageComposer::Empty,
        )),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "reword inline",
        key_matcher: press().code(KeyCode::Char('i')),
        modes: Vec::from([ModeDiscriminant::Commit]),
        message: Message::Commit(CommitMessage::ToggleMessageComposer(
            CommitMessageComposer::Inline,
        )),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "back",
        key_matcher: press().code(KeyCode::Char('c')),
        modes: Vec::from([ModeDiscriminant::Commit]),
        message: Message::EnterNormalMode,
        hide_from_hotbar: true,
    });

    key_binds.register(KeyBindDef {
        short_description: "back",
        key_matcher: press().code(KeyCode::Esc),
        modes: Vec::from([ModeDiscriminant::Commit]),
        message: Message::EnterNormalMode,
        hide_from_hotbar: false,
    });
}

fn register_move_mode_key_binds(key_binds: &mut KeyBinds) {
    key_binds.register(KeyBindDef {
        short_description: "move",
        key_matcher: press().code(KeyCode::Enter),
        modes: Vec::from([ModeDiscriminant::Move]),
        message: Message::Move(MoveMessage::Confirm),
        hide_from_hotbar: false,
    });

    key_binds.register(KeyBindDef {
        short_description: "back",
        key_matcher: press().code(KeyCode::Char('m')),
        modes: Vec::from([ModeDiscriminant::Move]),
        message: Message::EnterNormalMode,
        hide_from_hotbar: true,
    });

    key_binds.register(KeyBindDef {
        short_description: "back",
        key_matcher: press().code(KeyCode::Esc),
        modes: Vec::from([ModeDiscriminant::Move]),
        message: Message::EnterNormalMode,
        hide_from_hotbar: false,
    });
}

#[derive(Clone, Copy, Debug)]
struct KeyBindId(usize);

#[derive(Debug)]
pub(super) struct KeyBinds {
    /// All registered key binds.
    all_key_binds: Vec<Box<dyn KeyBind>>,
    /// Which key binds are available in which modes?
    mode_to_key_binds: HashMap<ModeDiscriminant, Vec<KeyBindId>>,
}

impl KeyBinds {
    fn new() -> Self {
        KeyBinds {
            mode_to_key_binds: Default::default(),
            all_key_binds: Default::default(),
        }
    }

    fn register<T>(&mut self, key_bind: T) -> KeyBindId
    where
        T: IntoKeyBind,
    {
        let key_bind = key_bind.into_key_bind();

        let id = KeyBindId(self.all_key_binds.len());

        for mode in ModeDiscriminant::iter() {
            if key_bind.available_in_mode(mode) {
                self.mode_to_key_binds.entry(mode).or_default().push(id);
            }
        }

        self.all_key_binds.push(Box::new(key_bind));

        id
    }

    pub(super) fn iter_key_binds_available_in_mode(
        &self,
        mode: &Mode,
    ) -> impl Iterator<Item = &dyn KeyBind> {
        let mode = ModeDiscriminant::from(mode);
        self.mode_to_key_binds
            .get(&mode)
            .into_iter()
            .flatten()
            .copied()
            .map(|KeyBindId(idx)| &*self.all_key_binds[idx])
    }
}

pub(super) trait IntoKeyBind {
    type KeyBind: KeyBind;

    fn into_key_bind(self) -> Self::KeyBind;
}

impl<T> IntoKeyBind for T
where
    T: KeyBind,
{
    type KeyBind = T;

    fn into_key_bind(self) -> Self::KeyBind {
        self
    }
}

pub(super) trait KeyBind: std::fmt::Debug + 'static {
    fn short_description(&self) -> &str;

    fn chord_display(&self) -> &str;

    fn hide_from_hotbar(&self) -> bool {
        false
    }

    fn available_in_mode(&self, mode: ModeDiscriminant) -> bool;

    fn matches(&self, ev: &KeyEvent) -> bool;

    fn message(&self) -> Message;
}

#[derive(Debug)]
struct KeyBindDef {
    short_description: &'static str,
    key_matcher: KeyMatcher,
    modes: Vec<ModeDiscriminant>,
    message: Message,
    hide_from_hotbar: bool,
}

impl IntoKeyBind for KeyBindDef {
    type KeyBind = StaticKeyBind;

    fn into_key_bind(self) -> Self::KeyBind {
        StaticKeyBind {
            short_description: self.short_description,
            chord_display: self.key_matcher.chord_display(),
            key_matcher: self.key_matcher,
            modes: self.modes,
            message: self.message,
            hide_from_hotbar: self.hide_from_hotbar,
        }
    }
}

#[derive(Debug)]
struct StaticKeyBind {
    short_description: &'static str,
    chord_display: Cow<'static, str>,
    key_matcher: KeyMatcher,
    modes: Vec<ModeDiscriminant>,
    message: Message,
    hide_from_hotbar: bool,
}

impl KeyBind for StaticKeyBind {
    fn short_description(&self) -> &str {
        self.short_description
    }

    fn chord_display(&self) -> &str {
        &self.chord_display
    }

    fn available_in_mode(&self, mode: ModeDiscriminant) -> bool {
        self.modes.contains(&mode)
    }

    fn matches(&self, ev: &KeyEvent) -> bool {
        self.key_matcher.matches(ev)
    }

    fn message(&self) -> Message {
        self.message.clone()
    }

    fn hide_from_hotbar(&self) -> bool {
        self.hide_from_hotbar
    }
}

#[inline]
fn press() -> KeyMatcher {
    KeyMatcher {
        kind: KeyEventKind::Press,
        modifiers: KeyModifiers::NONE,
        codes: [None, None],
    }
}

#[derive(Debug, Copy, Clone)]
struct KeyMatcher {
    kind: KeyEventKind,
    modifiers: KeyModifiers,
    codes: [Option<KeyCode>; 2],
}

impl KeyMatcher {
    #[inline]
    fn alt(self) -> Self {
        self.modifiers(KeyModifiers::ALT)
    }

    #[inline]
    fn shift(self) -> Self {
        self.modifiers(KeyModifiers::SHIFT)
    }

    #[inline]
    fn control(self) -> Self {
        self.modifiers(KeyModifiers::CONTROL)
    }

    #[inline]
    fn modifiers(mut self, modifiers: KeyModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    #[inline]
    fn code(mut self, code: KeyCode) -> Self {
        self.codes[0] = Some(code);
        self
    }

    #[inline]
    fn alt_code(mut self, code: KeyCode) -> Self {
        self.codes[1] = Some(code);
        self
    }

    /// Render this matcher into the hotbar chord display format.
    fn chord_display(&self) -> Cow<'static, str> {
        let mut codes = self.codes.into_iter().flatten().collect::<Vec<_>>();
        codes.sort_by_key(|code| self.display_sort_key(*code));

        let displays = codes
            .into_iter()
            .map(|code| self.format_code(code))
            .collect::<Vec<_>>();
        Cow::Owned(displays.join("/"))
    }

    /// Return the sort key used to produce a stable, user-facing display order.
    fn display_sort_key(&self, code: KeyCode) -> u8 {
        match code {
            KeyCode::Char(_) => 1,
            _ => 0,
        }
    }

    /// Format a single key code together with this matcher's modifiers.
    fn format_code(&self, code: KeyCode) -> String {
        let mut prefixes = Vec::new();
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            prefixes.push("ctrl");
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            prefixes.push("alt");
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            prefixes.push("shift");
        }

        let key = format_key_code(code, self.modifiers.contains(KeyModifiers::SHIFT));
        if prefixes.is_empty() {
            key
        } else {
            format!("{}+{key}", prefixes.join("+"))
        }
    }

    #[inline]
    fn matches(self, ev: &KeyEvent) -> bool {
        if self.kind != ev.kind {
            return false;
        }

        if self.modifiers != ev.modifiers {
            return false;
        }

        self.codes
            .into_iter()
            .flatten()
            .any(|key_code| key_code == ev.code)
    }
}

/// Format a key code into the hotbar display representation.
fn format_key_code(code: KeyCode, shifted: bool) -> String {
    match code {
        KeyCode::Backspace => "backspace".to_owned(),
        KeyCode::Enter => "enter".to_owned(),
        KeyCode::Left => "←".to_owned(),
        KeyCode::Right => "→".to_owned(),
        KeyCode::Up => "↑".to_owned(),
        KeyCode::Down => "↓".to_owned(),
        KeyCode::Home => "home".to_owned(),
        KeyCode::End => "end".to_owned(),
        KeyCode::PageUp => "pageup".to_owned(),
        KeyCode::PageDown => "pagedown".to_owned(),
        KeyCode::Tab => "tab".to_owned(),
        KeyCode::BackTab => "backtab".to_owned(),
        KeyCode::Delete => "del".to_owned(),
        KeyCode::Insert => "ins".to_owned(),
        KeyCode::Esc => "esc".to_owned(),
        KeyCode::Char(ch) => normalize_char_for_display(ch, shifted).to_string(),
        KeyCode::Null => "null".to_owned(),
        KeyCode::CapsLock => "capslock".to_owned(),
        KeyCode::ScrollLock => "scrolllock".to_owned(),
        KeyCode::NumLock => "numlock".to_owned(),
        KeyCode::PrintScreen => "printscreen".to_owned(),
        KeyCode::Pause => "pause".to_owned(),
        KeyCode::Menu => "menu".to_owned(),
        KeyCode::KeypadBegin => "keypadbegin".to_owned(),
        KeyCode::Media(_) => "media".to_owned(),
        KeyCode::Modifier(_) => "modifier".to_owned(),
        KeyCode::F(number) => format!("f{number}"),
    }
}

/// Normalize a character for chord display rendering.
fn normalize_char_for_display(ch: char, shifted: bool) -> char {
    if shifted && ch.is_ascii_alphabetic() {
        ch.to_ascii_lowercase()
    } else {
        ch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chord_display_for_plain_and_modified_keys() {
        assert_eq!(press().code(KeyCode::Char('q')).chord_display(), "q");
        assert_eq!(press().code(KeyCode::Enter).chord_display(), "enter");
        assert_eq!(press().code(KeyCode::Esc).chord_display(), "esc");
        assert_eq!(
            press().control().code(KeyCode::Char('r')).chord_display(),
            "ctrl+r"
        );
        assert_eq!(
            press().control().code(KeyCode::Char('[')).chord_display(),
            "ctrl+["
        );
        assert_eq!(
            press().alt().code(KeyCode::Char('e')).chord_display(),
            "alt+e"
        );
        assert_eq!(
            press().shift().code(KeyCode::Char('J')).chord_display(),
            "shift+j"
        );
    }

    #[test]
    fn chord_display_for_alternate_codes() {
        assert_eq!(
            press()
                .code(KeyCode::Char('j'))
                .alt_code(KeyCode::Down)
                .chord_display(),
            "↓/j"
        );
        assert_eq!(
            press()
                .code(KeyCode::Char('k'))
                .alt_code(KeyCode::Up)
                .chord_display(),
            "↑/k"
        );
        assert_eq!(
            press()
                .code(KeyCode::Char('h'))
                .alt_code(KeyCode::Left)
                .chord_display(),
            "←/h"
        );
        assert_eq!(
            press()
                .code(KeyCode::Char('l'))
                .alt_code(KeyCode::Right)
                .chord_display(),
            "→/l"
        );
        assert_eq!(
            press()
                .code(KeyCode::Char('n'))
                .alt_code(KeyCode::Esc)
                .chord_display(),
            "esc/n"
        );
    }

    #[test]
    fn matcher_still_matches_primary_and_alternate_codes() {
        let matcher = press().code(KeyCode::Char('j')).alt_code(KeyCode::Down);

        assert!(matcher.matches(&KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)));
        assert!(matcher.matches(&KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)));
        assert!(!matcher.matches(&KeyEvent::new(KeyCode::Char('j'), KeyModifiers::SHIFT)));
    }
}
