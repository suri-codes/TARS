use async_trait::async_trait;
use color_eyre::Result;
use common::{
    TarsClient,
    types::{Task, TaskFetchOptions},
};
use crossterm::event::KeyEvent;
use group_component::GroupComponent;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Paragraph},
};
use task_component::TaskComponent;
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::TextArea;

use crate::{
    action::Action,
    app::Mode,
    config::Config,
    tree::{TarsKind, TarsTreeHandle},
};

mod group_component;
mod task_component;

use super::{Component, frame_block};
#[derive(Debug)]
/// Inspector component that shows detailed information about groups and tasks,
/// and allows them to be modified.
pub struct Inspector<'a> {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    // selection: Option<Selection>,
    client: TarsClient,
    active: bool,
    tree_handle: TarsTreeHandle,

    // task_component: Option<TaskComponent<'a>>,
    // group_component: Option<GroupComponent<'a>>,
    active_component: Option<ActiveComponent<'a>>,
}

#[derive(Debug)]
enum ActiveComponent<'a> {
    TaskComponent(Box<TaskComponent<'a>>),
    GroupComponent(Box<GroupComponent<'a>>),
}

#[derive(Debug)]
struct TarsText<'a> {
    textarea: TextArea<'a>,
    is_valid: bool,
}
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
// let mut textarea = TextArea::default();
// textarea.set_cursor_line_style(Style::default());
// textarea.set_placeholder_text("Enter a valid float (e.g. 1.56)");

impl<'a> Inspector<'a> {
    pub async fn new(client: &TarsClient, tree_handle: TarsTreeHandle) -> Result<Self> {
        Ok(Self {
            command_tx: Default::default(),
            config: Default::default(),
            // selection: None,
            client: client.clone(),
            active: false,
            // task_component: None,
            // group_component: None,
            active_component: None,
            tree_handle,
        })
    }

    fn mode(&self) -> Mode {
        Mode::Inspector
    }
}

#[async_trait]
impl<'a> Component for Inspector<'a> {
    async fn init(
        &mut self,
        _area: ratatui::prelude::Size,
        default_mode: Mode,
    ) -> color_eyre::eyre::Result<()> {
        if default_mode == self.mode() {
            self.active = true
        }

        Ok(())
    }
    fn register_action_handler(
        &mut self,
        tx: UnboundedSender<Action>,
    ) -> color_eyre::eyre::Result<()> {
        self.command_tx = Some(tx.clone());
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> color_eyre::eyre::Result<()> {
        self.config = config;
        Ok(())
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if !self.active {
            return Ok(None);
        }

        match self.active_component.as_mut() {
            Some(ActiveComponent::TaskComponent(c)) => c.handle_key_event(key).await,
            Some(ActiveComponent::GroupComponent(c)) => c.handle_key_event(key).await,
            None => Ok(None),
        }
        // if let Some(task_component) = self.task_component.as_mut() {
        //     return task_component.handle_key_event(key).await;
        // }

        // Ok(None)
    }

    async fn update(&mut self, action: Action) -> color_eyre::eyre::Result<Option<Action>> {
        match action {
            Action::Tick => {}
            Action::Render => {}
            Action::SwitchTo(Mode::Inspector) => self.active = true,
            Action::SwitchTo(_) => self.active = false,
            // Action::Select(s) => match s {
            //     Selection::Task(ref t) => {
            //         let mut new_task_component = TaskComponent::new(t, self.client.clone())?;
            //         new_task_component.register_action_handler(
            //             self.command_tx.as_ref().expect("should exist").clone(),
            //         )?;
            //         self.active_component =
            //             Some(ActiveComponent::TaskComponent(Box::new(new_task_component)));
            //     }
            //     Selection::Group(ref g) => {
            //         let mut new_group_component = GroupComponent::new(g, self.client.clone())?;
            //         new_group_component.register_action_handler(
            //             self.command_tx.as_ref().expect("should exit").clone(),
            //         )?;

            //         self.active_component = Some(ActiveComponent::GroupComponent(Box::new(
            //             new_group_component,
            //         )));
            //     }
            // },
            //
            Action::Select(id) => {
                let tree = self.tree_handle.read().await;

                let node = tree.get(&id)?;

                match node.data().kind {
                    TarsKind::Task(ref t) => {
                        let mut new_task_component = TaskComponent::new(t, self.client.clone())?;
                        new_task_component.register_action_handler(
                            self.command_tx.as_ref().expect("should exist").clone(),
                        )?;
                        self.active_component =
                            Some(ActiveComponent::TaskComponent(Box::new(new_task_component)));
                    }

                    TarsKind::Group(ref g) => {
                        let mut new_group_component = GroupComponent::new(g, self.client.clone())?;
                        new_group_component.register_action_handler(
                            self.command_tx.as_ref().expect("should exit").clone(),
                        )?;

                        self.active_component = Some(ActiveComponent::GroupComponent(Box::new(
                            new_group_component,
                        )));
                    }

                    _ => {}
                }
            }

            Action::Refresh => match self.active_component {
                None => {}
                Some(ActiveComponent::TaskComponent(ref t)) => {
                    let task = t.task.clone();
                    //TODO: make task fetch by id an actual call
                    let all_tasks = Task::fetch(&self.client, TaskFetchOptions::All).await?;
                    let Some(task) = all_tasks.iter().find(|t| t.id == task.id) else {
                        self.active_component = None;
                        return Ok(None);
                    };

                    let mut selected_task = TaskComponent::new(task, self.client.clone())?;

                    selected_task.register_action_handler(
                        self.command_tx.as_ref().expect("should exist").clone(),
                    )?;

                    self.active_component =
                        Some(ActiveComponent::TaskComponent(Box::new(selected_task)));
                }
                Some(ActiveComponent::GroupComponent(ref _g)) => {
                    //TODO: write refresh code once we have a group_component too.
                    return Ok(None);
                }
            },
            _ => {}
        }
        Ok(None)
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::eyre::Result<()> {
        frame.render_widget(frame_block(self.active, self.mode()), area);

        let area = Layout::new(Direction::Horizontal, [Constraint::Percentage(100)])
            .horizontal_margin(3)
            .vertical_margin(2)
            .split(area)[0];

        match self.active_component.as_mut() {
            Some(ActiveComponent::TaskComponent(t)) => {
                t.draw(frame, area)?;
            }
            Some(ActiveComponent::GroupComponent(g)) => {
                g.draw(frame, area)?;
            }
            None => {
                frame.render_widget(Paragraph::new("Please perform a Selection!"), area);
            }
        }

        Ok(())
    }
}
