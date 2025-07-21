use async_trait::async_trait;
use color_eyre::eyre::Result;
use common::{
    TarsClient,
    types::{Color as MyColor, Group},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;
use tui_textarea::{Input, Key};

use crate::{action::Action, components::Component};

use super::TarsText;

#[derive(Debug)]
pub struct GroupComponent<'a> {
    group: Group,
    name: TarsText<'a>,
    color: TarsText<'a>,
    edit_mode: EditMode,
    client: TarsClient,
    command_tx: Option<UnboundedSender<Action>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum EditMode {
    #[default]
    Inactive,
    Name,
    Color,
}

impl<'a> GroupComponent<'a> {
    pub fn new(group: &Group, client: TarsClient) -> Result<Self> {
        let comp = Self {
            name: TarsText::new(
                &group.name,
                Block::new()
                    .title_top("[N]ame")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded),
            ),
            color: TarsText::new(
                group.color.as_str(),
                Block::new()
                    .title_top("[C]olor")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(group.color.clone().into())),
            ),
            group: group.clone(),
            edit_mode: EditMode::Inactive,
            client,
            command_tx: None,
        };
        Ok(comp)
    }

    pub async fn sync(&mut self) -> Result<()> {
        let new_name = self.name.textarea.lines()[0].clone();

        if !new_name.is_empty() {
            self.group.name = new_name.into();
        };

        let new_color = self.color.textarea.lines()[0].clone();

        if !new_color.is_empty() {
            self.group.color = MyColor::parse_str(self.color.textarea.lines()[0].as_str())?;
        }

        self.group.sync(&self.client).await?;

        Ok(())
    }
}

#[async_trait]
impl Component for GroupComponent<'_> {
    fn init(
        &mut self,

        _area: ratatui::prelude::Size,
        _default_mode: crate::app::Mode,
    ) -> color_eyre::eyre::Result<()> {
        Ok(())
    }

    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::SwitchTo(_) = action {
            self.name.deactivate()
        }
        Ok(None)
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx.clone());
        info!("received action handler");
        Ok(())
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match self.edit_mode {
            EditMode::Inactive => {
                if let KeyCode::Char('n') | KeyCode::Char('N') = key.code {
                    self.name.activate();
                    self.edit_mode = EditMode::Name;
                    return Ok(Some(Action::RawText));
                }
                if let KeyCode::Char('c') | KeyCode::Char('C') = key.code {
                    self.color.activate();
                    self.edit_mode = EditMode::Color;
                    return Ok(Some(Action::RawText));
                }
            }
            EditMode::Name => {
                match key.into() {
                    Input { key: Key::Esc, .. }
                    | Input {
                        key: Key::Enter, ..
                    } => {
                        self.name.deactivate();
                        self.sync().await?;
                        self.edit_mode = EditMode::Inactive;
                        return Ok(Some(Action::Refresh));
                    }
                    input => {
                        self.name.textarea.input(input);
                        // TextArea::input returns if the input modified its text
                        // if textarea.input(input) {
                        //     is_valid = validate(&mut textarea);
                        // }
                    }
                }
            }

            EditMode::Color => match key.into() {
                Input { key: Key::Esc, .. }
                | Input {
                    key: Key::Enter, ..
                } => {
                    self.color.deactivate();

                    if self.color.is_valid {
                        self.sync().await?;
                    }
                    self.edit_mode = EditMode::Inactive;
                    return Ok(Some(Action::Refresh));
                }

                input => {
                    if self.color.textarea.input(input) {
                        let entered_color = self.color.textarea.lines()[0].as_str();
                        let Some(block) = self.color.textarea.block().cloned() else {
                            return Ok(None);
                        };

                        let block = block.border_style(Style::new().fg(Color::Red));

                        let block = if let Ok(col) = MyColor::parse_str(entered_color) {
                            self.color.is_valid = true;
                            block.border_style(Style::new().fg(col.into()))
                        } else {
                            self.color.is_valid = false;
                            block
                        };
                        self.color.textarea.set_block(block);
                    }
                }
            },
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) -> Result<()> {
        frame.render_widget(Paragraph::new("lol"), area);

        let group_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Percentage(15), // name
                Constraint::Percentage(15), // color
                Constraint::Percentage(15), // parent
            ],
        )
        .split(area);

        // Group name:
        frame.render_widget(&self.name.textarea, group_layout[0]);

        // Group color:
        //
        frame.render_widget(&self.color.textarea, group_layout[1]);
        // frame.render_widget(
        //     Paragraph::new(self.group.color.as_str()).block(
        //         Block::new()
        //             .title_top("Color")
        //             .borders(Borders::all())
        //             .border_type(BorderType::Rounded)
        //             .style(Style::new().fg(self.group.color.clone().into())),
        //     ),
        //     group_layout[1],
        // );
        Ok(())
    }
}
