use async_trait::async_trait;
use color_eyre::Result;
use common::{
    TarsClient,
    types::{Task, TaskFetchOptions},
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, app::Mode, config::Config};

use super::{Component, frame_block};

// #[derive(Default)]
pub struct TaskView {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    task: Option<Task>,
    _client: TarsClient,
    active: bool,
}

impl TaskView {
    pub async fn new(client: TarsClient) -> Result<Self> {
        let task = Task::fetch(&client, TaskFetchOptions::All)
            .await?
            .first()
            .cloned();

        Ok(Self {
            command_tx: Default::default(),
            config: Default::default(),
            task,
            _client: client,
            active: false,
        })
    }

    fn mode(&self) -> Mode {
        Mode::TaskView
    }
}

#[async_trait]
impl Component for TaskView {
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
            Action::SwitchTo(Mode::TaskView) => self.active = true,
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
        // draw frame
        frame.render_widget(frame_block(self.active, self.mode()), area);
        if self.task.is_none() {
            // todo
            frame.render_widget(Paragraph::new("Please select a Task!"), area);
            return Ok(());
        }

        let task = self.task.as_ref().unwrap();

        let task_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Percentage(12), // name
                Constraint::Percentage(15), // group
                Constraint::Percentage(15), // priority
                Constraint::Percentage(38), // Description
                Constraint::Percentage(15), // completion | Due
            ],
        )
        .horizontal_margin(3)
        .vertical_margin(2)
        .split(area);

        // Task name:

        frame.render_widget(
            Paragraph::new(task.name.as_str())
                .block(Block::new().title_top("Name").borders(Borders::all())),
            task_layout[0],
        );

        // Group name:
        frame.render_widget(
            Paragraph::new(task.group.name.as_str())
                .block(Block::new().title_top("Group").borders(Borders::all())),
            task_layout[1],
        );

        // Priority
        frame.render_widget(
            Paragraph::new(Into::<String>::into(task.priority))
                .block(Block::new().title_top("Priority").borders(Borders::all())),
            task_layout[2],
        );

        // Description
        frame.render_widget(
            Paragraph::new(task.description.clone()).block(
                Block::new()
                    .title_top("Description")
                    .borders(Borders::all()),
            ),
            task_layout[3],
        );

        let completion_due = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .split(task_layout[4]);

        let completion_symbol = if task.completed {
            "☑ Awesome"
        } else {
            "☐ Get to work cornball"
        };
        // Completion status
        frame.render_widget(
            Paragraph::new(completion_symbol)
                .block(Block::new().title_top("Completed").borders(Borders::all())),
            completion_due[0],
        );

        // Due Date
        frame.render_widget(
            Paragraph::new(format!("{:?}", task.due))
                .block(Block::new().title_top("Due").borders(Borders::all())),
            completion_due[1],
        );

        Ok(())
    }
}
