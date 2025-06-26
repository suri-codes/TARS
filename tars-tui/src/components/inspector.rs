use async_trait::async_trait;
use color_eyre::Result;
use common::{
    TarsClient,
    types::{Priority, Task, TaskFetchOptions},
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::{Action, Selection},
    app::Mode,
    config::Config,
};

use super::{Component, frame_block};
#[derive(Debug)]
pub struct Inspector {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    task: Option<Task>,
    client: TarsClient,
    active: bool,
}

impl Inspector {
    pub async fn new(client: &TarsClient) -> Result<Self> {
        let task = Task::fetch(client, TaskFetchOptions::All)
            .await?
            .first()
            .cloned();

        Ok(Self {
            command_tx: Default::default(),
            config: Default::default(),
            task,
            client: client.clone(),
            active: false,
        })
    }

    fn mode(&self) -> Mode {
        Mode::Inspector
    }
}

#[async_trait]
impl Component for Inspector {
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
            Action::SwitchTo(Mode::Inspector) => self.active = true,
            Action::SwitchTo(_) => self.active = false,
            Action::Select(Selection::Task(t)) => self.task = Some(t),
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
                Constraint::Percentage(15), // name
                Constraint::Percentage(15), // group | Priority
                Constraint::Percentage(50), // Description
                Constraint::Percentage(15), // completion | Due
            ],
        )
        .horizontal_margin(3)
        .vertical_margin(2)
        .split(area);

        // Task name:

        frame.render_widget(
            Paragraph::new(task.name.as_str()).block(
                Block::new()
                    .title_top("Name")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded),
            ),
            task_layout[0],
        );

        // group | priority
        let group_priority = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .split(task_layout[1]);

        // Group name:
        frame.render_widget(
            Paragraph::new(task.group.name.as_str()).block(
                Block::new()
                    .title_top("Group")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded),
            ),
            group_priority[0],
        );

        // Priority
        frame.render_widget(
            Paragraph::new(Into::<String>::into(task.priority)).block(
                Block::new()
                    .title_top("Priority")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded)
                    .style({
                        match task.priority {
                            Priority::Far => Style::new().fg(ratatui::style::Color::LightBlue),
                            Priority::Low => Style::new().fg(ratatui::style::Color::Blue),
                            Priority::Medium => Style::new().fg(ratatui::style::Color::Yellow),
                            Priority::High => Style::new().fg(ratatui::style::Color::LightRed),
                            Priority::Asap => Style::new().fg(ratatui::style::Color::Red),
                        }
                    }),
            ),
            group_priority[1],
        );

        // Description
        frame.render_widget(
            Paragraph::new(task.description.clone()).block(
                Block::new()
                    .title_top("Description")
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

        let completion_symbol = if task.completed {
            " ✅ Awesome"
        } else {
            " ❌ Get to work cornball"
        };
        // Completion status
        frame.render_widget(
            Paragraph::new(completion_symbol).block({
                let block = Block::new()
                    .title_top("Completed")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded);

                let style = if task.completed {
                    Style::new().fg(Color::Green)
                } else {
                    Style::new().fg(Color::Red)
                };

                block.style(style)
            }),
            completion_due[0],
        );

        // Due Date
        frame.render_widget(
            Paragraph::new(format!("{:?}", task.due)).block(
                Block::new()
                    .title_top("Due")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded),
            ),
            completion_due[1],
        );

        Ok(())
    }
}
