use std::{
    io::{Stdin, stdin, stdout},
    process::{Command, Stdio},
    thread::sleep,
    time::Duration,
};

use async_trait::async_trait;
use color_eyre::Result;
use common::{
    ParseError, TarsClient,
    types::{Priority, Task, parse_date_time},
};
use crossterm::{
    ExecutableCommand, cursor,
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;
use tui_textarea::{Input, Key, TextArea};

use crate::{action::Action, components::Component};

#[derive(Debug)]
pub struct TaskComponent<'a> {
    task: Task,
    name: TarsText<'a>,
    description: String,
    due: TarsText<'a>,
    priority: TarsText<'a>,
    edit_mode: EditMode,
    client: TarsClient,
    // action_tx:
    command_tx: Option<UnboundedSender<Action>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum EditMode {
    #[default]
    Inactive,
    Name,
    Priority,
    Due,
}

#[derive(Debug)]
struct TarsText<'a> {
    textarea: TextArea<'a>,
    is_valid: bool,
}

// for the description we are going to use helix as the editor

impl<'a> TarsText<'a> {
    pub fn new(string: &str, block: Block<'a>) -> Self {
        let mut text_area = TextArea::default();
        text_area.set_placeholder_text(string);
        text_area.set_placeholder_style(Style::default());
        text_area.set_block(block);

        let mut text = Self {
            textarea: text_area,
            is_valid: true,
        };

        text.deactivate();
        text
    }

    pub fn deactivate(&mut self) {
        self.textarea.set_cursor_line_style(Style::default());
        self.textarea.set_cursor_style(Style::default());
    }

    pub fn activate(&mut self) {
        self.textarea
            .set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
        self.textarea
            .set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    }
}

impl<'a> TaskComponent<'a> {
    pub fn new(task: &Task, client: TarsClient) -> Self {
        Self {
            name: TarsText::new(
                &task.name,
                Block::new()
                    .title_top("[N]ame")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded),
            ),
            priority: TarsText::new(
                Into::<String>::into(task.priority).as_str(),
                task.priority.into(),
            ),
            edit_mode: EditMode::Inactive,
            client,
            description: task.description.to_owned(),
            due: TarsText::new(
                Into::<String>::into(
                    task.due
                        .map(|d| d.format("%m/%d/%Y %I:%M:%S %p").to_string())
                        .unwrap_or_else(|| "None".to_string()),
                )
                .as_str(),
                Block::new()
                    .title_top("D[u]e")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded),
            ),
            task: task.clone(),
            command_tx: None,
        }
    }

    pub async fn sync(&mut self) -> Result<()> {
        let new_name = self.name.textarea.lines()[0].clone();

        if !new_name.is_empty() {
            self.task.name = new_name.into();
        };

        self.task.sync(&self.client).await?;

        Ok(())
    }
}

#[async_trait]
impl Component for TaskComponent<'_> {
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
                if let KeyCode::Char('p') | KeyCode::Char('P') = key.code {
                    self.priority.activate();
                    self.edit_mode = EditMode::Priority;
                    return Ok(Some(Action::RawText));
                }
                if let KeyCode::Char('d') | KeyCode::Char('D') = key.code {
                    return Ok(Some(Action::LaunchHelix("lol".to_owned())));
                    // Ideally we would like to swap into helix to edit the description file and then pop back out once we exit helix, that way shi stays clean
                }
                if let KeyCode::Char('c') | KeyCode::Char('C') = key.code {
                    self.task.completed = !self.task.completed;
                    return self.sync().await.map(|_| None);
                }
                if let KeyCode::Char('u') | KeyCode::Char('U') = key.code {
                    self.due.activate();
                    self.edit_mode = EditMode::Due;
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
                        // can validate here
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
            EditMode::Priority => {
                match key.into() {
                    Input { key: Key::Esc, .. }
                    | Input {
                        key: Key::Enter, ..
                    } => {
                        self.priority.deactivate();
                        if self.priority.is_valid {
                            self.sync().await?;
                        }
                        self.priority
                            .textarea
                            .set_placeholder_text(self.task.priority);
                        self.edit_mode = EditMode::Inactive;
                        //NOTE maybe not the best thing to do but its the easiest way to reset all the placeholder text
                        return Ok(Some(Action::Refresh));
                    }
                    input => {
                        if self.priority.textarea.input(input) {
                            let p: Result<Priority, ParseError> =
                                self.priority.textarea.lines()[0].as_str().try_into();
                            let Some(block) = self.priority.textarea.block().cloned() else {
                                return Ok(None);
                            };

                            let block = match p {
                                Ok(p) => {
                                    self.task.priority = p;
                                    self.priority.is_valid = true;
                                    self.task.priority.into()
                                }
                                Err(_) => {
                                    self.priority.is_valid = false;
                                    block.border_style(Style::new().fg(Color::Red))
                                }
                            };

                            self.priority.textarea.set_block(block);
                        };
                    }
                };
            }
            EditMode::Due => {
                match key.into() {
                    Input { key: Key::Esc, .. }
                    | Input {
                        key: Key::Enter, ..
                    } => {
                        self.due.deactivate();
                        if self.due.is_valid {
                            self.sync().await?;
                        }
                        // self.due.textarea.set_placeholder_text(self.task.priority);
                        self.edit_mode = EditMode::Inactive;
                        //NOTE maybe not the best thing to do but its the easiest way to reset all the placeholder text
                        return Ok(Some(Action::Refresh));
                    }
                    input => {
                        if self.due.textarea.input(input) {
                            // now we want to parse if the due date is valid

                            let entered_date_str = self.due.textarea.lines()[0].as_str();
                            let Some(block) = self.due.textarea.block().cloned() else {
                                return Ok(None);
                            };

                            let block = block.border_style(Style::new().fg(Color::Red));

                            let block = match parse_date_time(entered_date_str) {
                                Ok(date) => {
                                    self.task.due = Some(date);
                                    self.due.is_valid = true;
                                    block.border_style(Style::new().fg(Color::Green))
                                }

                                Err(_) => {
                                    if entered_date_str.is_empty() {
                                        self.task.due = None;
                                        self.due.is_valid = true;
                                        self.due.textarea.set_placeholder_text("None");

                                        block.border_style(Style::new().fg(Color::Green))
                                    } else {
                                        self.due.is_valid = false;
                                        block.border_style(Style::new().fg(Color::Red))
                                    }
                                }
                            };

                            self.due.textarea.set_block(block);
                        };
                    }
                }
            }
        }

        Ok(None)
    }
    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::eyre::Result<()> {
        let task_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Percentage(15), // name
                Constraint::Percentage(15), // group | Priority
                Constraint::Percentage(50), // Description
                Constraint::Percentage(15), // completion | Due
            ],
        )
        .split(area);

        // Task name:
        frame.render_widget(&self.name.textarea, task_layout[0]);

        // group | priority
        let group_priority = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .split(task_layout[1]);

        // Group name:
        frame.render_widget(
            Paragraph::new(self.task.group.name.as_str()).block(
                Block::new()
                    .title_top("Group")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded)
                    .style(Style::new().fg((&self.task.group.color).into())),
            ),
            group_priority[0],
        );

        // Priority
        frame.render_widget(&self.priority.textarea, group_priority[1]);

        // Description
        frame.render_widget(
            Paragraph::new(self.description.clone()).block(
                Block::new()
                    .title_top("[D]escription")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded),
            ),
            task_layout[2],
        );

        let completion_due = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .split(task_layout[3]);

        let completion_symbol = if self.task.completed {
            " ✅ Awesome"
        } else {
            " ❌ Get to work cornball"
        };

        // Completion status
        frame.render_widget(
            Paragraph::new(completion_symbol).block({
                let block = Block::new()
                    .title_top("[C]ompleted")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded);

                let style = if self.task.completed {
                    Style::new().fg(Color::Green)
                } else {
                    Style::new().fg(Color::Red)
                };

                block.style(style)
            }),
            completion_due[0],
        );

        // Due Date
        frame.render_widget(&self.due.textarea, completion_due[1]);

        Ok(())
    }
}
