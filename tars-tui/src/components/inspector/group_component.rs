use async_trait::async_trait;
use color_eyre::eyre::{OptionExt, Result};
use common::{
    TarsClient,
    types::{Color as MyColor, Group, Id, Task},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
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
pub struct GroupComponent<'a> {
    group: Group,
    name: TarsText<'a>,
    color: TarsText<'a>,
    edit_mode: EditMode,
    client: TarsClient,
    command_tx: Option<UnboundedSender<Action>>,
    tree_handle: TarsTreeHandle,
    on_update: OnUpdate,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum EditMode {
    #[default]
    Inactive,
    Name,
    Color,
}

#[derive(Debug)]
enum OnUpdate {
    NoOp,
    ReRender,
    NewTask(Id),
}

struct ReactiveDrawInfo<'a> {
    name: TarsText<'a>,
    color: TarsText<'a>,
}

impl From<&Group> for ReactiveDrawInfo<'_> {
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

        ReactiveDrawInfo { name, color }
    }
}
impl<'a> GroupComponent<'a> {
    pub fn new(group: &Group, client: TarsClient, tree_handle: TarsTreeHandle) -> Result<Self> {
        let reactive_draw_info = ReactiveDrawInfo::from(group);
        let comp = Self {
            name: reactive_draw_info.name,
            color: reactive_draw_info.color,
            group: group.clone(),
            edit_mode: EditMode::Inactive,
            client,
            command_tx: None,
            tree_handle,
            on_update: OnUpdate::NoOp,
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

    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Select(id) => {
                let tree = self.tree_handle.read().await;
                let node = tree.get(&id)?;

                if let TarsKind::Group(ref group) = node.data().kind {
                    self.group = group.clone();

                    let reactive_draw_info = ReactiveDrawInfo::from(group);
                    self.color = reactive_draw_info.color;
                    self.name = reactive_draw_info.name;
                }
            }

            Action::Update => match self.on_update {
                OnUpdate::ReRender => {
                    let tree = self.tree_handle.read().await;
                    let node = tree.get_by_tars_id(&self.group.id).expect("should exist");

                    if let TarsKind::Group(ref group) = node.data().kind {
                        self.group = group.clone();

                        let reactive_draw_info = ReactiveDrawInfo::from(group);
                        self.color = reactive_draw_info.color;
                        self.name = reactive_draw_info.name;
                    }

                    self.on_update = OnUpdate::NoOp;
                }
                OnUpdate::NewTask(ref id) => {
                    let tree = self.tree_handle.read().await;

                    let node_id = tree
                        .translate_id_to_node_id(id)
                        .ok_or_eyre("node should exist in here")?;

                    self.command_tx
                        .as_mut()
                        .ok_or_eyre("command tx should exist")?
                        .send(Action::Select(node_id))
                        .unwrap();

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
        //TODO: if someone presses t on a group, it just creates a task and they can start editing it
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

                if let KeyCode::Char('t') | KeyCode::Char('T') = key.code {
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
                }

                if let KeyCode::Char('r') | KeyCode::Char('R') = key.code {
                    let new_color = MyColor::random();

                    self.group.color = new_color;

                    self.sync().await?;
                }
            }
            EditMode::Name => match key.into() {
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
                }
            },

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
        frame.render_widget(&self.color.textarea, group_layout[1]);
        Ok(())
    }
}
