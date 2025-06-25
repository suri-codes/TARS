use async_trait::async_trait;
use common::{
    TarsClient,
    types::{Task, TaskFetchOptions},
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::Paragraph,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, app::Mode, config::Config};
use color_eyre::Result;

use super::{Component, frame_block};

#[derive(Debug)]
pub struct TodoList {
    command_tx: Option<UnboundedSender<Action>>,
    client: TarsClient,
    config: Config,
    active: bool,
    tasks: Vec<Task>,
    selection: u16,
}

impl TodoList {
    pub async fn new(client: &TarsClient) -> Result<Self> {
        // new todo list will start at root scope
        let tasks = Task::fetch(client, TaskFetchOptions::All).await?;

        Ok(Self {
            command_tx: Default::default(),
            config: Default::default(),
            client: client.clone(),
            active: false,
            tasks,
            selection: 0,
        })
    }

    fn mode(&self) -> Mode {
        Mode::TodoList
    }
}

#[async_trait]
impl Component for TodoList {
    fn init(
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
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> color_eyre::eyre::Result<()> {
        self.config = config;
        Ok(())
    }

    async fn update(&mut self, action: Action) -> color_eyre::eyre::Result<Option<Action>> {
        match action {
            Action::Tick => {}
            Action::Render => {}
            Action::SwitchTo(Mode::TodoList) => self.active = true,
            Action::SwitchTo(_) => self.active = false,
            Action::ScopeUpdate(scope) => {
                if let Some(g) = scope {
                    self.tasks =
                        Task::fetch(&self.client, TaskFetchOptions::ByGroup { group: g }).await?;
                } else {
                    self.tasks = Task::fetch(&self.client, TaskFetchOptions::All).await?;
                }

                // TODO: run priority sorting algorithmn
            }

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

        let area = Layout::new(Direction::Vertical, [Constraint::Percentage(100)])
            .horizontal_margin(2)
            .vertical_margin(1)
            .split(area)[0];

        let constraints: Vec<Constraint> = self.tasks.iter().map(|_| Constraint::Max(1)).collect();

        let task_layouts = Layout::new(Direction::Vertical, constraints).split(area);
        for (i, (task, area)) in self.tasks.iter().zip(task_layouts.into_iter()).enumerate() {
            frame.render_widget(
                Paragraph::new((*task.name).to_string()).style({
                    if i as u16 == self.selection {
                        Style::new().bg(Color::Blue)
                    } else {
                        Style::new().bg(Color::Reset)
                    }
                }),
                *area,
            );
        }
        Ok(())
    }
}
