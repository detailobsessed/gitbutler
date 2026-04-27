use std::{borrow::Cow, cell::Cell};

use bstr::ByteSlice;
use crossterm::event::Event;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use gix::refs::FullName;
use nonempty::NonEmpty;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, List, ListItem, Padding},
};
use ratatui_textarea::TextArea;
use unicode_width::UnicodeWidthStr;

use crate::{
    command::legacy::status::tui::Message,
    theme::{PatchStyle as _, Theme},
    utils::DebugAsType,
};

#[derive(Debug)]
pub(super) struct BranchPicker {
    items: NonEmpty<Item>,
    branches_to_show: Vec<BranchToShow>,
    textarea: TextArea<'static>,
    cursor: usize,
    scroll_top: Cell<usize>,
    matcher: DebugAsType<SkimMatcherV2>,
    on_branch_selected: DebugAsType<Box<dyn FnOnce(Item, &mut Vec<Message>) -> anyhow::Result<()>>>,
    theme: &'static Theme,
}

#[derive(Debug, Clone)]
pub(super) enum Item {
    Branch(FullName),
    Unassigned,
}

impl Item {
    fn name(&self) -> Cow<'_, str> {
        match self {
            Item::Branch(full_name) => full_name.shorten().to_str_lossy(),
            Item::Unassigned => Cow::Borrowed("unassigned changes"),
        }
    }

    fn style(&self, theme: &'static Theme) -> Style {
        match self {
            Item::Branch(..) => theme.local_branch,
            Item::Unassigned => theme.info,
        }
    }
}

#[derive(Debug)]
enum BranchToShow {
    Plain {
        branch_names_idx: usize,
    },
    FuzzyMatch {
        branch_names_idx: usize,
        char_indices: Vec<usize>,
    },
}

impl BranchPicker {
    pub(super) fn new<F>(
        branch_names: NonEmpty<FullName>,
        theme: &'static Theme,
        on_branch_selected: F,
    ) -> Self
    where
        F: FnOnce(Item, &mut Vec<Message>) -> anyhow::Result<()> + 'static,
    {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(theme.default);

        let mut items = NonEmpty::new(Item::Unassigned);
        items.extend(branch_names.map(Item::Branch));

        let mut this = Self {
            items,
            branches_to_show: Default::default(),
            textarea,
            cursor: 0,
            scroll_top: Cell::new(0),
            on_branch_selected: DebugAsType(Box::new(on_branch_selected)),
            matcher: DebugAsType(SkimMatcherV2::default()),
            theme,
        };

        this.filter_branches();

        this
    }

    pub(super) fn render(&self, area: Rect, frame: &mut Frame) {
        let padding = Padding {
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
        };
        let horizontal_padding = padding.left + padding.right;

        let space_taken_up_by_border: u16 = 2;
        let input_height: u16 = 1;

        let longest_branch_name_width = self
            .items
            .iter()
            .map(|item| item.name().width())
            .max()
            .unwrap();

        let horizontal_layout = Layout::horizontal([Constraint::Length(std::cmp::max(
            (longest_branch_name_width as u16) + space_taken_up_by_border + horizontal_padding,
            65,
        ))])
        .flex(Flex::Center)
        .split(area);

        let popup_height = 15.min(area.height);

        let centered_layout = Layout::vertical([Constraint::Length(popup_height)])
            .flex(Flex::Center)
            .split(horizontal_layout[0]);

        frame.render_widget(Clear, centered_layout[0]);

        let outer_block = Block::bordered()
            .padding(padding)
            .border_type(BorderType::Rounded)
            .border_style(self.theme.border);
        let inner_area = outer_block.inner(centered_layout[0]);
        frame.render_widget(outer_block, centered_layout[0]);

        let content_layout =
            Layout::vertical([Constraint::Length(input_height), Constraint::Min(1)])
                .split(inner_area);

        {
            let input_layout = Layout::horizontal([Constraint::Length(2), Constraint::Min(1)])
                .split(content_layout[0]);
            frame.render_widget("> ", input_layout[0]);
            frame.render_widget(&self.textarea, input_layout[1]);
        }

        let visible_rows = content_layout[1].height as usize;
        let mut scroll_top = self.scroll_top.get();

        if visible_rows == 0 {
            scroll_top = 0;
        } else {
            if self.cursor < scroll_top {
                scroll_top = self.cursor;
            } else if self.cursor >= scroll_top + visible_rows {
                scroll_top = self.cursor + 1 - visible_rows;
            }

            let max_scroll = self.branches_to_show.len().saturating_sub(visible_rows);
            scroll_top = scroll_top.min(max_scroll);
        }

        self.scroll_top.set(scroll_top);

        let items = self
            .branches_to_show
            .iter()
            .enumerate()
            .skip(scroll_top)
            .take(visible_rows)
            .map(|(idx, branches_to_show_idx)| {
                let item = match branches_to_show_idx {
                    BranchToShow::Plain { branch_names_idx } => {
                        let item = &self.items[*branch_names_idx];
                        ListItem::new(item.name()).style(item.style(self.theme))
                    }
                    BranchToShow::FuzzyMatch {
                        branch_names_idx,
                        char_indices,
                    } => {
                        let item = &self.items[*branch_names_idx];
                        let branch_name = item.name();
                        let spans = branch_name.chars().enumerate().map(|(idx, c)| {
                            let span = Span::raw(c.to_string());
                            if char_indices.contains(&idx) {
                                span.underlined()
                            } else {
                                span
                            }
                        });
                        ListItem::new(Line::from_iter(spans)).style(item.style(self.theme))
                    }
                };
                if idx == self.cursor {
                    item.patch_style(self.theme.selection_highlight)
                } else {
                    item
                }
            });

        frame.render_widget(List::new(items), content_layout[1]);
    }

    fn filter_branches(&mut self) {
        let query = self
            .textarea
            .lines()
            .first()
            .map(|q| &**q)
            .unwrap_or_default();

        self.branches_to_show.clear();
        self.cursor = 0;
        self.scroll_top.set(0);

        if query.is_empty() {
            self.branches_to_show.extend(
                self.items
                    .iter()
                    .enumerate()
                    .map(|(branch_names_idx, _)| BranchToShow::Plain { branch_names_idx }),
            );
        } else {
            let mut fuzzy_matches = self
                .items
                .iter()
                .enumerate()
                .filter_map(|(branch_names_idx, item)| {
                    let branch_name = item.name();
                    let (score, indices) = self.matcher.fuzzy_indices(&branch_name, query)?;
                    Some((branch_names_idx, branch_name, score, indices))
                })
                .collect::<Vec<_>>();
            fuzzy_matches.sort_unstable_by(|(_, _, score_a, _), (_, _, score_b, _)| {
                score_a.cmp(score_b).reverse()
            });
            self.branches_to_show.extend(fuzzy_matches.into_iter().map(
                |(branch_names_idx, _, _, indices)| BranchToShow::FuzzyMatch {
                    branch_names_idx,
                    char_indices: indices,
                },
            ));
        }
    }

    pub(super) fn handle_message(
        mut self,
        msg: BranchPickerMessage,
        messages: &mut Vec<Message>,
    ) -> anyhow::Result<Option<Self>> {
        match msg {
            BranchPickerMessage::Close => Ok(None),
            BranchPickerMessage::MoveCursorDown => {
                let cursor = if self.branches_to_show.is_empty() {
                    0
                } else {
                    std::cmp::min(
                        self.cursor.saturating_add(1),
                        self.branches_to_show.len() - 1,
                    )
                };
                Ok(Some(Self { cursor, ..self }))
            }
            BranchPickerMessage::MoveCursorUp => Ok(Some(Self {
                cursor: self.cursor.saturating_sub(1),
                ..self
            })),
            BranchPickerMessage::Confirm => {
                let Some(branch_name) = self
                    .branches_to_show
                    .get(self.cursor)
                    .map(|idx| match idx {
                        BranchToShow::Plain { branch_names_idx }
                        | BranchToShow::FuzzyMatch {
                            branch_names_idx, ..
                        } => *branch_names_idx,
                    })
                    .map(|branch_names_idx| &self.items[branch_names_idx])
                    .cloned()
                else {
                    return Ok(Some(self));
                };
                (self.on_branch_selected.0)(branch_name, messages)?;
                Ok(None)
            }
            BranchPickerMessage::Input(event) => {
                self.textarea.input(event);
                self.filter_branches();
                Ok(Some(self))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum BranchPickerMessage {
    MoveCursorDown,
    MoveCursorUp,
    Input(Event),
    Confirm,
    Close,
}
