use async_trait::async_trait;
use color_eyre::Result;
use common::TarsClient;
use crossterm::event::KeyEvent;
use group_component::GroupComponent;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Paragraph},
};
use task_component::TaskComponent;
use tokio::sync::mpsc::UnboundedSender;
use tracing::debug;
use tui_textarea::TextArea;

use crate::{
    action::{Action, Signal},
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
    signal_tx: Option<UnboundedSender<Signal>>,
    config: Config,
    // selection: Option<Selection>,
    client: TarsClient,
    active: bool,
    tree_handle: TarsTreeHandle,
    rendered_component: RenderedComponent<'a>,
}

#[derive(Debug)]
struct RenderedComponent<'a> {
    active_component: RenderedComponentKind,
    task_component: Option<Box<TaskComponent<'a>>>,
    group_component: Option<Box<GroupComponent<'a>>>,
}

#[derive(Debug)]
enum RenderedComponentKind {
    Task,
    Group,
    Blank,
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

impl<'a> Inspector<'a> {
    pub async fn new(client: &TarsClient, tree_handle: TarsTreeHandle) -> Result<Self> {
        Ok(Self {
            signal_tx: Default::default(),
            config: Default::default(),
            // selection: None,
            client: client.clone(),
            active: false,
            rendered_component: RenderedComponent {
                active_component: RenderedComponentKind::Blank,
                task_component: None,
                group_component: None,
            },

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
    fn register_signal_handler(
        &mut self,
        tx: UnboundedSender<Signal>,
    ) -> color_eyre::eyre::Result<()> {
        self.signal_tx = Some(tx.clone());
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> color_eyre::eyre::Result<()> {
        self.config = config;
        Ok(())
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Signal>> {
        if !self.active {
            return Ok(None);
        }

        let rendered_component = &mut self.rendered_component;

        match rendered_component.active_component {
            RenderedComponentKind::Task => {
                rendered_component
                    .task_component
                    .as_mut()
                    .unwrap()
                    .handle_key_event(key)
                    .await
            }
            RenderedComponentKind::Group => {
                rendered_component
                    .group_component
                    .as_mut()
                    .unwrap()
                    .handle_key_event(key)
                    .await
            }
            _ => Ok(None),
        }
    }

    async fn update(&mut self, action: Signal) -> color_eyre::eyre::Result<Option<Signal>> {
        match action {
            Signal::Tick => {}
            Signal::Render => {}
            Signal::Action(Action::SwitchTo(Mode::Inspector)) => {
                self.active = true;
                match self.rendered_component.active_component {
                    RenderedComponentKind::Task => {
                        self.rendered_component
                            .task_component
                            .as_mut()
                            .unwrap()
                            .active = true;
                    }
                    RenderedComponentKind::Group => {
                        self.rendered_component
                            .group_component
                            .as_mut()
                            .unwrap()
                            .active = true;
                    }
                    RenderedComponentKind::Blank => {}
                }
            }

            Signal::Action(Action::SwitchTo(_)) => {
                self.active = false;
                if let Some(group_component) = &mut self.rendered_component.group_component {
                    group_component.active = false;
                }
                if let Some(task_component) = &mut self.rendered_component.task_component {
                    task_component.active = false;
                }
            }
            Signal::Select(ref id) => {
                // on first select
                // we make sure that we carry the task and group components
                debug!("Processing select for {id:#?}");

                // we can use this id to determine what we should be using
                let tree = self.tree_handle.read().await;
                let node = tree.get(id)?;
                match node.data().kind {
                    TarsKind::Task(ref t) => {
                        if self.rendered_component.task_component.is_none() {
                            let mut task_component = TaskComponent::new(
                                t,
                                self.client.clone(),
                                self.tree_handle.clone(),
                            )?;
                            task_component
                                .register_signal_handler(self.signal_tx.clone().unwrap())?;
                            task_component.register_config_handler(self.config.clone())?;

                            self.rendered_component.task_component = Some(Box::new(task_component));
                        }

                        if let Some(group_component) = &mut self.rendered_component.group_component
                        {
                            group_component.active = false;
                        }

                        if self.active {
                            self.rendered_component
                                .task_component
                                .as_mut()
                                .unwrap()
                                .active = true;
                        }

                        self.rendered_component.active_component = RenderedComponentKind::Task;
                    }

                    TarsKind::Group(ref g) => {
                        if self.rendered_component.group_component.is_none() {
                            let mut group_component = GroupComponent::new(
                                g,
                                self.client.clone(),
                                self.tree_handle.clone(),
                            )?;

                            group_component
                                .register_signal_handler(self.signal_tx.clone().unwrap())?;
                            group_component.register_config_handler(self.config.clone())?;

                            self.rendered_component.group_component =
                                Some(Box::new(group_component));
                        }

                        if let Some(task_component) = &mut self.rendered_component.task_component {
                            task_component.active = false;
                        }

                        if self.active {
                            self.rendered_component
                                .group_component
                                .as_mut()
                                .unwrap()
                                .active = true;
                        }

                        self.rendered_component.active_component = RenderedComponentKind::Group;
                    }

                    _ => {}
                }
            }
            _ => {}
        }

        return match self.rendered_component.active_component {
            RenderedComponentKind::Task => {
                self.rendered_component
                    .task_component
                    .as_mut()
                    .unwrap()
                    .update(action)
                    .await
            }
            RenderedComponentKind::Group => {
                self.rendered_component
                    .group_component
                    .as_mut()
                    .unwrap()
                    .update(action)
                    .await
            }
            _ => Ok(None),
        };
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

        let rendered_component = &mut self.rendered_component;

        match rendered_component.active_component {
            RenderedComponentKind::Task => rendered_component
                .task_component
                .as_mut()
                .unwrap()
                .draw(frame, area)?,
            RenderedComponentKind::Group => rendered_component
                .group_component
                .as_mut()
                .unwrap()
                .draw(frame, area)?,
            RenderedComponentKind::Blank => {
                frame.render_widget(Paragraph::new("Please perform a Selection!"), area);
            }
        }

        Ok(())
    }
}
