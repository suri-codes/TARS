use async_trait::async_trait;
use color_eyre::eyre::{OptionExt, Result};
use common::{
    ParseError, TarsClient,
    types::{Color as MyColor, Group, Id, Priority, Task},
};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
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
pub struct GroupComponent<'a> {
    group: Group,
    edit_mode: EditMode,
    client: TarsClient,
    signal_tx: Option<UnboundedSender<Signal>>,
    tree_handle: TarsTreeHandle,
    reactive_widgets: ReactiveWidgets<'a>,
    on_update: OnUpdate,
    pub active: bool,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum EditMode {
    #[default]
    Inactive,
    Name,
    Color,
    Priority,
}

#[derive(Debug)]
enum OnUpdate {
    NoOp,
    ReRender,
    NewTask(Id),
}

#[derive(Debug)]
struct ReactiveWidgets<'a> {
    name: TarsText<'a>,
    color: TarsText<'a>,
    priority: TarsText<'a>,
}

impl From<&Group> for ReactiveWidgets<'_> {
    fn from(value: &Group) -> Self {
        let name = TarsText::new(
            &value.name,
            Block::new()
                .title_top("[N]ame")
                .borders(Borders::all())
                .border_type(BorderType::Rounded),
        );

        let color = TarsText::new(
            value.color.as_str(),
            Block::new()
                .title_top("[C]olor")
                .borders(Borders::all())
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(value.color.clone().into())),
        );

        let priority = TarsText::new(&value.priority.to_string(), value.priority.into());

        ReactiveWidgets {
            name,
            color,
            priority,
        }
    }
}
impl<'a> GroupComponent<'a> {
    pub fn new(group: &Group, client: TarsClient, tree_handle: TarsTreeHandle) -> Result<Self> {
        let reactive_widgets = ReactiveWidgets::from(group);
        let comp = Self {
            reactive_widgets,
            group: group.clone(),
            edit_mode: EditMode::Inactive,
            client,
            signal_tx: None,
            tree_handle,
            on_update: OnUpdate::NoOp,
            active: false,
        };
        Ok(comp)
    }

    pub async fn sync(&mut self) -> Result<()> {
        let new_name = self.reactive_widgets.name.textarea.lines()[0].clone();

        if !new_name.is_empty() {
            self.group.name = new_name.into();
        };

        let new_color = self.reactive_widgets.color.textarea.lines()[0].clone();

        if !new_color.is_empty() {
            self.group.color =
                MyColor::parse_str(self.reactive_widgets.color.textarea.lines()[0].as_str())?;
        }

        let new_prio = self.reactive_widgets.priority.textarea.lines()[0].as_str();

        if !new_prio.is_empty() {
            self.group.priority = new_prio.try_into()?;
        }

        self.group.sync(&self.client).await?;
        self.on_update = OnUpdate::ReRender;

        Ok(())
    }
}

#[async_trait]
impl Component for GroupComponent<'_> {
    async fn init(
        &mut self,

        _area: ratatui::prelude::Size,
        _default_mode: crate::app::Mode,
    ) -> color_eyre::eyre::Result<()> {
        Ok(())
    }

    async fn update(&mut self, action: Signal) -> Result<Option<Signal>> {
        match action {
            Signal::Select(id) => {
                let tree = self.tree_handle.read().await;
                let node = tree.get(&id)?;

                if let TarsKind::Group(ref group) = node.data().kind {
                    self.group = group.clone();

                    let reactive_draw_info = ReactiveWidgets::from(group);
                    self.reactive_widgets.color = reactive_draw_info.color;
                    self.reactive_widgets.name = reactive_draw_info.name;
                }
                Ok(None)
            }

            Signal::Update => match self.on_update {
                OnUpdate::ReRender => {
                    let tree = self.tree_handle.read().await;
                    let node = tree.get_by_tars_id(&self.group.id).expect("should exist");

                    if let TarsKind::Group(ref group) = node.data().kind {
                        self.group = group.clone();
                        self.reactive_widgets = ReactiveWidgets::from(group);
                    }

                    self.on_update = OnUpdate::NoOp;
                    Ok(None)
                }
                OnUpdate::NewTask(ref id) => {
                    let tree = self.tree_handle.read().await;

                    let node_id = tree
                        .translate_id_to_node_id(id)
                        .ok_or_eyre("node should exist in here")?;

                    self.signal_tx
                        .as_mut()
                        .ok_or_eyre("command tx should exist")?
                        .send(Signal::Select(node_id))
                        .unwrap();

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
                    Action::EditColor => {
                        self.reactive_widgets.color.activate();
                        self.edit_mode = EditMode::Color;
                        Ok(Some(Signal::RawText))
                    }

                    Action::EditPriority => {
                        self.reactive_widgets.priority.activate();
                        self.edit_mode = EditMode::Priority;
                        Ok(Some(Signal::RawText))
                    }

                    Action::NewTask => {
                        let id = Task::new(
                            &self.client,
                            &self.group,
                            "new task",
                            common::types::Priority::Medium,
                            "",
                            None,
                        )
                        .await?
                        .id;

                        self.on_update = OnUpdate::NewTask(id);
                        Ok(None)
                    }

                    Action::RandomColor => {
                        let new_color = MyColor::random();

                        self.group.color = new_color;

                        self.sync().await?;
                        Ok(None)
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
            EditMode::Name => match key.into() {
                Input { key: Key::Esc, .. }
                | Input {
                    key: Key::Enter, ..
                } => {
                    self.reactive_widgets.name.deactivate();
                    self.sync().await?;
                    self.edit_mode = EditMode::Inactive;
                    return Ok(Some(Signal::Refresh));
                }
                input => {
                    self.reactive_widgets.name.textarea.input(input);
                }
            },

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
                            .set_placeholder_text(self.group.priority);
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
                                    self.group.priority = p;
                                    self.reactive_widgets.priority.is_valid = true;
                                    self.group.priority.into()
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

            EditMode::Color => match key.into() {
                Input { key: Key::Esc, .. }
                | Input {
                    key: Key::Enter, ..
                } => {
                    self.reactive_widgets.color.deactivate();

                    if self.reactive_widgets.color.is_valid {
                        self.sync().await?;
                    }
                    self.edit_mode = EditMode::Inactive;
                    return Ok(Some(Signal::Refresh));
                }

                input => {
                    if self.reactive_widgets.color.textarea.input(input) {
                        let entered_color =
                            self.reactive_widgets.color.textarea.lines()[0].as_str();
                        let Some(block) = self.reactive_widgets.color.textarea.block().cloned()
                        else {
                            return Ok(None);
                        };

                        let block = block.border_style(Style::new().fg(Color::Red));

                        let block = if let Ok(col) = MyColor::parse_str(entered_color) {
                            self.reactive_widgets.color.is_valid = true;
                            block.border_style(Style::new().fg(col.into()))
                        } else {
                            self.reactive_widgets.color.is_valid = false;
                            block
                        };
                        self.reactive_widgets.color.textarea.set_block(block);
                    }
                }
            },
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) -> Result<()> {
        let group_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Percentage(15), // name
                Constraint::Percentage(15), // color
                Constraint::Percentage(15), // priority
            ],
        )
        .split(area);

        // Group name:
        frame.render_widget(&self.reactive_widgets.name.textarea, group_layout[0]);

        // Group color:
        frame.render_widget(&self.reactive_widgets.color.textarea, group_layout[1]);

        // Group priority:
        frame.render_widget(&self.reactive_widgets.priority.textarea, group_layout[2]);

        Ok(())
    }
}
