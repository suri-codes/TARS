use async_trait::async_trait;
use color_eyre::Result;
use common::{
    TarsClient,
    types::{Group, Task, TaskFetchOptions},
};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::Paragraph,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, app::Mode, config::Config};

use super::{Component, frame_block};

#[derive(Default)]
/// Explorer component that allows you to navigate between different groups (scopes).
pub struct Explorer {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    active: bool,
    groups: Vec<Group>,
    tasks: Vec<Task>,
    #[expect(dead_code)]
    widgets: Vec<TodoWidget>,
}

#[expect(dead_code)]
struct TodoWidget {
    w_type: TodoWidgetType,
}

#[expect(dead_code)]
enum TodoWidgetType {
    Task(Task),
    Group(Group),
}

impl Explorer {
    pub async fn new(client: &TarsClient) -> Result<Self> {
        // need some sort of datastructure i assume?
        let groups = Group::fetch_all(client).await?;
        let tasks = Task::fetch(client, TaskFetchOptions::All).await?;
        Ok(Self {
            command_tx: Default::default(),
            config: Default::default(),
            active: false,
            groups,
            tasks,
            widgets: Vec::new(),
        })
    }

    fn mode(&self) -> Mode {
        Mode::Explorer
    }

    #[expect(dead_code)]
    fn process(&mut self) {
        let _root_groups: Vec<&Group> = self
            .groups
            .iter()
            .filter(|e| e.parent_id.is_none())
            .collect();

        todo!(
            "need to figure out how to process the current 'scope' into a renderable datastructure"
        )
    }
}

#[async_trait]
impl Component for Explorer {
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
            Action::SwitchTo(Mode::Explorer) => self.active = true,
            Action::SwitchTo(_) => self.active = false,
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
        // how am i supposed to render this shit dawg

        // need to divide up the area. algorithmically.

        // ideally top 4 tasks per group + a line that says more coming after

        // groups organized by parents

        for (task, area) in self.tasks.iter().zip(task_layouts.iter()) {
            frame.render_widget(
                Paragraph::new((*task.name).to_string()).style(Style::new().bg(Color::Blue)),
                *area,
            );
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        // vim bindings
        // j would move selection down
        // k would move selection up
        // l would move into a new scope
        // h would move into the outer scope

        let _ = key; // to appease clippy
        Ok(None)
    }
}
