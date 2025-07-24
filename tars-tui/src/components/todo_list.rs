use async_trait::async_trait;
use common::{
    TarsClient,
    types::{Task, TaskFetchOptions},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::Paragraph,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, app::Mode, config::Config};
use color_eyre::Result;

use super::{Component, frame_block};

#[derive(Debug)]
/// Component that shows all the tasks within the current scope, ordered by priority.
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
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> color_eyre::eyre::Result<()> {
        self.config = config;
        Ok(())
    }

    async fn update(&mut self, action: Action) -> color_eyre::eyre::Result<Option<Action>> {
        match action {
            Action::Tick => Ok(None),
            Action::Render => Ok(None),
            Action::SwitchTo(Mode::TodoList) => {
                self.active = true;

                // yeah gotta rework todo_list component
                // let action = self
                //     .tasks
                //     .get(self.selection as usize)
                //     .map(|t| Action::Select(Selection::Task(t.clone())));

                // Ok(action)
                Ok(None)
            }
            Action::SwitchTo(_) => {
                self.active = false;
                Ok(None)
            }
            Action::ScopeUpdate(scope) => {
                if let Some(g) = scope {
                    self.tasks = Task::fetch(
                        &self.client,
                        TaskFetchOptions::ByGroup {
                            group_id: g.id,
                            recursive: true,
                        },
                    )
                    .await?;
                } else {
                    self.tasks = Task::fetch(&self.client, TaskFetchOptions::All).await?;
                }

                Ok(None)

                // TODO: run priority sorting algorithmn
            }

            _ => Ok(None),
        }
    }
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if !self.active {
            return Ok(None);
        }

        // vim bindings
        // j would move selection down
        // k would move selection up
        // l would move into a new scope
        // h would move into the outer scope
        match key.code {
            KeyCode::Char('j') => {
                // would increment by one
                if let Some(next) = self.tasks.get(self.selection as usize + 1) {
                    // self.selection += 1;
                    // return Ok(Some(Action::Select(Selection::Task(next.clone()))));
                }

                Ok(None)
            }
            KeyCode::Char('k') => {
                if let Some(prev) = self.tasks.get({
                    if let Some(i) = (self.selection as usize).checked_sub(1) {
                        i
                    } else {
                        return Ok(None);
                    }
                }) {
                    // self.selection -= 1;
                    // return Ok(Some(Action::Select(Selection::Task(prev.clone()))));
                }

                Ok(None)
            }
            KeyCode::Char('l') => Ok(None),
            KeyCode::Char('h') => Ok(None),
            _ => Ok(None),
        }
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

        let task_group_layouts = Layout::new(Direction::Vertical, constraints).split(area);

        let split = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(60), Constraint::Percentage(40)],
        );

        let splits: Vec<(Rect, Rect)> = task_group_layouts
            .iter()
            .map(|rect| {
                let virt_split = split.clone();

                let x = virt_split.split(*rect);

                (x[0], x[1])
            })
            .collect();

        for (i, (task, (task_area, group_area))) in self.tasks.iter().zip(splits.iter()).enumerate()
        {
            let text_style = Style::new().fg((&task.group.color).into()).bg({
                if i as u16 == self.selection {
                    if self.active {
                        Color::Rgb(70, 70, 70)
                    } else {
                        Color::Rgb(35, 35, 35)
                    }
                } else {
                    Color::Reset
                }
            });

            frame.render_widget(
                Paragraph::new((*task.name).to_string()).style(text_style),
                *task_area,
            );
            frame.render_widget(
                Paragraph::new((*task.group.name).to_string()).style(text_style),
                *group_area,
            );
        }
        Ok(())
    }
}
