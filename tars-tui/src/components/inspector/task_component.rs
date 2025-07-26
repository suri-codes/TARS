use async_trait::async_trait;
use color_eyre::Result;
use common::{
    ParseError, TarsClient,
    types::{Priority, Task, parse_date_time},
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

use crate::{
    action::Action,
    components::Component,
    tree::{TarsKind, TarsTreeHandle},
};

use super::TarsText;

#[derive(Debug)]
pub struct TaskComponent<'a> {
    pub task: Task,
    name: TarsText<'a>,
    description: String,
    due: TarsText<'a>,
    priority: TarsText<'a>,
    edit_mode: EditMode,
    client: TarsClient,
    command_tx: Option<UnboundedSender<Action>>,
    tree_handle: TarsTreeHandle,
    static_draw_info: StaticDrawInfo<'a>,
    on_update: OnUpdate,
}

#[derive(Debug)]
enum OnUpdate {
    NoOp,
    ReRender,
}
#[derive(Debug, Default, PartialEq, Eq)]
enum EditMode {
    #[default]
    Inactive,
    Name,
    Priority,
    Due,
}

struct ReactiveDrawInfo<'a> {
    name: TarsText<'a>,
    due: TarsText<'a>,
    priority: TarsText<'a>,
}

impl From<&Task> for ReactiveDrawInfo<'_> {
    fn from(value: &Task) -> Self {
        let name = TarsText::new(
            &value.name,
            Block::new()
                .title_top("[N]ame")
                .borders(Borders::all())
                .border_type(BorderType::Rounded),
        );
        let priority = TarsText::new(
            Into::<String>::into(value.priority).as_str(),
            value.priority.into(),
        );

        let due = TarsText::new(
            Into::<String>::into(
                value
                    .due
                    .map(|d| d.format("%m/%d/%Y %I:%M:%S %p").to_string())
                    .unwrap_or_else(|| "None".to_string()),
            )
            .as_str(),
            Block::new()
                .title_top("D[u]e")
                .borders(Borders::all())
                .border_type(BorderType::Rounded),
        );

        ReactiveDrawInfo {
            name,
            due,
            priority,
        }
    }
}

/// static draw info
#[derive(Debug)]
struct StaticDrawInfo<'a> {
    task_layout: Layout,
    group_priority_layout: Layout,
    group: Paragraph<'a>,
    description: Paragraph<'a>,
    completion_due_layout: Layout,
    completion: Paragraph<'a>,
}

impl From<&Task> for StaticDrawInfo<'_> {
    fn from(value: &Task) -> Self {
        let task_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Percentage(15), // name
                Constraint::Percentage(15), // group | Priority
                Constraint::Percentage(50), // Description
                Constraint::Percentage(15), // completion | Due
            ],
        );

        let group_priority_layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        );

        let group = Paragraph::new((*value.group.name).clone()).block(
            Block::new()
                .title_top("Group")
                .borders(Borders::all())
                .border_type(BorderType::Rounded)
                .style(Style::new().fg((&value.group.color).into())),
        );

        let description = Paragraph::new(value.description.clone()).block(
            Block::new()
                .title_top("[D]escription")
                .borders(Borders::all())
                .border_type(BorderType::Rounded),
        );

        let completion_due_layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        );

        let completion = {
            let completion_symbol = if value.completed {
                " ✅ Awesome"
            } else {
                " ❌ Get to work cornball"
            };

            Paragraph::new(completion_symbol).block({
                let block = Block::new()
                    .title_top("[C]ompleted")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded);

                let style = if value.completed {
                    Style::new().fg(Color::Green)
                } else {
                    Style::new().fg(Color::Red)
                };

                block.style(style)
            })
        };

        StaticDrawInfo {
            task_layout,
            group_priority_layout,
            group,
            description,
            completion_due_layout,
            completion,
        }
    }
}

impl<'a> TaskComponent<'a> {
    pub fn new(task: &Task, client: TarsClient, tree_handle: TarsTreeHandle) -> Result<Self> {
        let reactive_draw_info = ReactiveDrawInfo::from(task);
        let static_draw_info = StaticDrawInfo::from(task);
        Ok(Self {
            name: reactive_draw_info.name,
            priority: reactive_draw_info.priority,
            client,
            description: task.description.clone(),
            due: reactive_draw_info.due,
            task: task.clone(),
            edit_mode: EditMode::Inactive,
            command_tx: None,
            static_draw_info,

            on_update: OnUpdate::NoOp,
            tree_handle,
        })
    }

    async fn sync(&mut self) -> Result<()> {
        let new_name = self.name.textarea.lines()[0].clone();

        if !new_name.is_empty() {
            self.task.name = new_name.into();
        };

        self.task.sync(&self.client).await?;

        self.on_update = OnUpdate::ReRender;

        Ok(())
    }
}

#[async_trait]
impl Component for TaskComponent<'_> {
    async fn init(
        &mut self,

        _area: ratatui::prelude::Size,
        _default_mode: crate::app::Mode,
    ) -> color_eyre::eyre::Result<()> {
        Ok(())
    }

    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Select(id) => {
                let tree = self.tree_handle.read().await;
                let node = tree.get(&id)?;

                if let TarsKind::Task(ref task) = node.data().kind {
                    self.task = task.clone();
                    self.description = task.description.clone();
                    self.static_draw_info = StaticDrawInfo::from(task);

                    let reactive_draw_info = ReactiveDrawInfo::from(task);
                    self.priority = reactive_draw_info.priority;
                    self.due = reactive_draw_info.due;
                    self.name = reactive_draw_info.name;
                }
            }

            Action::Update => match self.on_update {
                OnUpdate::ReRender => {
                    let tree = self.tree_handle.read().await;
                    let node = tree
                        .get_by_tars_id(self.task.id.clone())
                        .expect("should exist");

                    if let TarsKind::Task(ref task) = node.data().kind {
                        self.task = task.clone();
                        self.description = task.description.clone();
                        self.static_draw_info = StaticDrawInfo::from(task);

                        let reactive_draw_info = ReactiveDrawInfo::from(task);
                        self.priority = reactive_draw_info.priority;
                        self.due = reactive_draw_info.due;
                        self.name = reactive_draw_info.name;
                    }

                    self.on_update = OnUpdate::NoOp;
                }
                OnUpdate::NoOp => {}
            },
            _ => {}
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
                    return Ok(Some(Action::EditDescription(self.task.clone())));
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
                        self.edit_mode = EditMode::Inactive;
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
        let draw_info = &self.static_draw_info;
        let task_rects = draw_info.task_layout.split(area);

        // Task name:
        frame.render_widget(&self.name.textarea, task_rects[0]);

        // group | priority
        let group_priority = draw_info.group_priority_layout.split(task_rects[1]);

        // Group name:
        frame.render_widget(&draw_info.group, group_priority[0]);

        // Priority
        frame.render_widget(&self.priority.textarea, group_priority[1]);

        // Description
        frame.render_widget(&draw_info.description, task_rects[2]);

        let completion_due = draw_info.completion_due_layout.split(task_rects[3]);
        // Completion status
        frame.render_widget(&draw_info.completion, completion_due[0]);

        // Due Date
        frame.render_widget(&self.due.textarea, completion_due[1]);

        Ok(())
    }
}
