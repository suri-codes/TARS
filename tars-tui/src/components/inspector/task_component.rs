use async_trait::async_trait;
use chrono::Local;
use color_eyre::Result;
use common::{
    ParseError, TarsClient,
    types::{Priority, Task, parse_date_time},
};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;
use tui_textarea::{Input, Key};

use crate::{
    action::{Action, Signal},
    components::Component,
    tree::{TarsKind, TarsTreeHandle},
};

use super::TarsText;

#[derive(Debug)]
pub struct TaskComponent<'a> {
    pub task: Task,
    description: String,
    edit_mode: EditMode,
    client: TarsClient,
    signal_tx: Option<UnboundedSender<Signal>>,
    tree_handle: TarsTreeHandle,
    static_draw_info: StaticDrawInfo<'a>,
    reactive_widgets: ReactiveWidgets<'a>,
    on_update: OnUpdate,
    pub active: bool,
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

#[derive(Debug)]
struct ReactiveWidgets<'a> {
    name: TarsText<'a>,
    due: TarsText<'a>,
    priority: TarsText<'a>,
}

impl From<&Task> for ReactiveWidgets<'_> {
    fn from(value: &Task) -> Self {
        let name = TarsText::new(
            &value.name,
            Block::new()
                .title_top("[N]ame")
                .borders(Borders::all())
                .border_type(BorderType::Rounded),
        );
        let priority = TarsText::new(&value.priority.to_string(), value.priority.into());

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

        ReactiveWidgets {
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
                Constraint::Percentage(20), // group | Priority
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
            let completion_symbol = if let Some(time) = value.finished_at {
                format!("{} ", time.format("%m/%d/%Y %I:%M:%S %p"))
            } else {
                " ❌ Get to work cornball".to_owned()
            };

            Paragraph::new(completion_symbol).block({
                let block = Block::new()
                    .title_top("[F]inished")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded);

                let style = if value.finished_at.is_some() {
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
        //DEBUG
        info!("new task component! :{task:#?}");
        let reactive_draw_info = ReactiveWidgets::from(task);
        let static_draw_info = StaticDrawInfo::from(task);
        Ok(Self {
            reactive_widgets: reactive_draw_info,
            client,
            description: task.description.clone(),
            task: task.clone(),
            edit_mode: EditMode::Inactive,
            signal_tx: None,
            static_draw_info,
            on_update: OnUpdate::NoOp,
            tree_handle,
            active: false,
        })
    }

    async fn sync(&mut self) -> Result<()> {
        let new_name = self.reactive_widgets.name.textarea.lines()[0].clone();

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

    async fn update(&mut self, signal: Signal) -> Result<Option<Signal>> {
        match signal {
            Signal::Select(id) => {
                let tree = self.tree_handle.read().await;
                let node = tree.get(&id)?;

                if let TarsKind::Task(ref task) = node.data().kind {
                    self.task = task.clone();
                    self.description = task.description.clone();
                    self.static_draw_info = StaticDrawInfo::from(task);

                    let reactive_draw_info = ReactiveWidgets::from(task);
                    self.reactive_widgets = reactive_draw_info;
                }
                Ok(None)
            }

            Signal::Update => match self.on_update {
                OnUpdate::ReRender => {
                    let tree = self.tree_handle.read().await;
                    let node = tree.get_by_tars_id(&self.task.id).expect("should exist");

                    if let TarsKind::Task(ref task) = node.data().kind {
                        self.task = task.clone();
                        self.description = task.description.clone();
                        self.static_draw_info = StaticDrawInfo::from(task);
                        self.reactive_widgets = ReactiveWidgets::from(task);
                    }

                    self.on_update = OnUpdate::NoOp;
                    Ok(None)
                }
                OnUpdate::NoOp => Ok(None),
            },

            Signal::Action(action) => {
                if !self.active {
                    return Ok(None);
                }
                info!("Processing {}", action);

                match action {
                    Action::EditName => {
                        self.reactive_widgets.name.activate();
                        self.edit_mode = EditMode::Name;
                        Ok(Some(Signal::RawText))
                    }
                    Action::EditPriority => {
                        self.reactive_widgets.priority.activate();
                        self.edit_mode = EditMode::Priority;
                        Ok(Some(Signal::RawText))
                    }
                    Action::EditDescription => {
                        self.on_update = OnUpdate::ReRender;
                        Ok(Some(Signal::EditDescriptionForTask(self.task.clone())))
                    }

                    Action::ToggleFinishTask => {
                        // if self.task.finished_at.is_some() {
                        //     self.task.finished_at = None;
                        // } else {
                        //     let current_time = {
                        //         let now = Local::now();
                        //         now.naive_local()
                        //     };

                        //     self.task.finished_at = Some(current_time);
                        // }
                        //
                        self.task.toggle_finish(&self.client).await?;

                        self.on_update = OnUpdate::ReRender;
                        Ok(None)
                    }
                    Action::EditDue => {
                        self.reactive_widgets.due.activate();
                        self.edit_mode = EditMode::Due;
                        Ok(Some(Signal::RawText))
                    }
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }

    fn register_signal_handler(&mut self, tx: UnboundedSender<Signal>) -> Result<()> {
        self.signal_tx = Some(tx.clone());
        info!("received action handler");
        Ok(())
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Signal>> {
        match self.edit_mode {
            EditMode::Inactive => {}
            EditMode::Name => {
                match key.into() {
                    Input { key: Key::Esc, .. }
                    | Input {
                        key: Key::Enter, ..
                    } => {
                        self.reactive_widgets.name.deactivate();
                        self.sync().await?;
                        self.edit_mode = EditMode::Inactive;
                        return Ok(Some(Signal::Refresh));
                        // can validate here
                    }
                    input => {
                        self.reactive_widgets.name.textarea.input(input);
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
                        self.reactive_widgets.priority.deactivate();
                        if self.reactive_widgets.priority.is_valid {
                            self.sync().await?;
                        }
                        self.reactive_widgets
                            .priority
                            .textarea
                            .set_placeholder_text(self.task.priority);
                        self.edit_mode = EditMode::Inactive;
                        return Ok(Some(Signal::Refresh));
                    }
                    input => {
                        if self.reactive_widgets.priority.textarea.input(input) {
                            let p: Result<Priority, ParseError> =
                                self.reactive_widgets.priority.textarea.lines()[0]
                                    .as_str()
                                    .try_into();
                            let Some(block) =
                                self.reactive_widgets.priority.textarea.block().cloned()
                            else {
                                return Ok(None);
                            };

                            let block = match p {
                                Ok(p) => {
                                    self.task.priority = p;
                                    self.reactive_widgets.priority.is_valid = true;
                                    self.task.priority.into()
                                }
                                Err(_) => {
                                    self.reactive_widgets.priority.is_valid = false;
                                    block.border_style(Style::new().fg(Color::Red))
                                }
                            };

                            self.reactive_widgets.priority.textarea.set_block(block);
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
                        self.reactive_widgets.due.deactivate();
                        if self.reactive_widgets.due.is_valid {
                            self.sync().await?;
                        }
                        self.edit_mode = EditMode::Inactive;
                        return Ok(Some(Signal::Refresh));
                    }
                    input => {
                        if self.reactive_widgets.due.textarea.input(input) {
                            // now we want to parse if the due date is valid

                            let entered_date_str =
                                self.reactive_widgets.due.textarea.lines()[0].as_str();

                            let Some(block) = self.reactive_widgets.due.textarea.block().cloned()
                            else {
                                return Ok(None);
                            };

                            let block = block.border_style(Style::new().fg(Color::Red));

                            let block = match parse_date_time(entered_date_str) {
                                Ok(date) => {
                                    self.task.due = Some(date);
                                    self.reactive_widgets.due.is_valid = true;
                                    block.border_style(Style::new().fg(Color::Green))
                                }

                                Err(_) => {
                                    if entered_date_str.is_empty() {
                                        self.task.due = None;
                                        self.reactive_widgets.due.is_valid = true;
                                        self.reactive_widgets
                                            .due
                                            .textarea
                                            .set_placeholder_text("None");

                                        block.border_style(Style::new().fg(Color::Green))
                                    } else {
                                        self.reactive_widgets.due.is_valid = false;
                                        block.border_style(Style::new().fg(Color::Red))
                                    }
                                }
                            };

                            self.reactive_widgets.due.textarea.set_block(block);
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
        frame.render_widget(&self.reactive_widgets.name.textarea, task_rects[0]);

        // group | priority
        let group_priority = draw_info.group_priority_layout.split(task_rects[1]);

        // Group name:
        frame.render_widget(&draw_info.group, group_priority[0]);

        // Priority
        frame.render_widget(&self.reactive_widgets.priority.textarea, group_priority[1]);

        // Description
        frame.render_widget(&draw_info.description, task_rects[2]);

        let completion_due = draw_info.completion_due_layout.split(task_rects[3]);
        // Completion status
        frame.render_widget(&draw_info.completion, completion_due[0]);

        // Due Date
        frame.render_widget(&self.reactive_widgets.due.textarea, completion_due[1]);

        Ok(())
    }
}
