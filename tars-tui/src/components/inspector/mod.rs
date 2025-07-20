use async_trait::async_trait;
use color_eyre::Result;
use common::{
    TarsClient,
    types::{Task, TaskFetchOptions},
};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, BorderType, Borders, Paragraph},
};
use task_component::TaskComponent;
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

use crate::{
    action::{Action, Selection},
    app::Mode,
    config::Config,
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
    selection: Option<Selection>,

    client: TarsClient,
    active: bool,

    task_component: Option<TaskComponent<'a>>,
}

// let mut textarea = TextArea::default();
// textarea.set_cursor_line_style(Style::default());
// textarea.set_placeholder_text("Enter a valid float (e.g. 1.56)");

impl<'a> Inspector<'a> {
    pub async fn new(client: &TarsClient) -> Result<Self> {
        Ok(Self {
            command_tx: Default::default(),
            config: Default::default(),
            selection: None,
            client: client.clone(),
            active: false,
            task_component: None,
        })
    }

    fn mode(&self) -> Mode {
        Mode::Inspector
    }
}

#[async_trait]
impl<'a> Component for Inspector<'a> {
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

        if let Some(task_component) = self.task_component.as_mut() {
            return task_component.handle_key_event(key).await;
        }

        Ok(None)
    }

    async fn update(&mut self, action: Action) -> color_eyre::eyre::Result<Option<Action>> {
        match action {
            Action::Tick => {}
            Action::Render => {}
            Action::SwitchTo(Mode::Inspector) => self.active = true,
            Action::SwitchTo(_) => self.active = false,
            Action::Select(s) => {
                if let Selection::Task(ref t) = s {
                    let mut new_task_component = TaskComponent::new(t, self.client.clone());
                    new_task_component.register_action_handler(
                        self.command_tx.as_ref().expect("should exist").clone(),
                    )?;
                    self.task_component = Some(new_task_component);
                }
                self.selection = Some(s);
            }

            Action::Refresh => match self.selection {
                None => {}
                Some(Selection::Task(ref task)) => {
                    //TODO: make task fetch by id an actual call
                    let all_tasks = Task::fetch(&self.client, TaskFetchOptions::All).await?;
                    let Some(task) = all_tasks.iter().find(|t| t.id == task.id) else {
                        self.selection = None;
                        return Ok(None);
                    };

                    let mut selected_task = TaskComponent::new(task, self.client.clone());

                    selected_task.register_action_handler(
                        self.command_tx.as_ref().expect("should exist").clone(),
                    )?;
                    self.task_component = Some(selected_task);
                }
                Some(Selection::Group(ref _g)) => {
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
        // draw frame
        frame.render_widget(frame_block(self.active, self.mode()), area);

        let area = Layout::new(Direction::Horizontal, [Constraint::Percentage(100)])
            .horizontal_margin(3)
            .vertical_margin(2)
            .split(area)[0];

        match self.selection {
            Some(Selection::Task(ref _task)) => {
                self.task_component.as_mut().unwrap().draw(frame, area)?;

                // let task_layout = Layout::new(
                //     Direction::Vertical,
                //     [
                //         Constraint::Percentage(15), // name
                //         Constraint::Percentage(15), // group | Priority
                //         Constraint::Percentage(50), // Description
                //         Constraint::Percentage(15), // completion | Due
                //     ],
                // )
                // .split(area);

                // // Task name:
                // frame.render_widget(
                //     Paragraph::new(task.name.as_str()).block(
                //         Block::new()
                //             .title_top("Name")
                //             .borders(Borders::all())
                //             .border_type(BorderType::Rounded),
                //     ),
                //     task_layout[0],
                // );

                // // group | priority
                // let group_priority = Layout::new(
                //     Direction::Horizontal,
                //     [Constraint::Percentage(50), Constraint::Percentage(50)],
                // )
                // .split(task_layout[1]);

                // // Group name:
                // frame.render_widget(
                //     Paragraph::new(task.group.name.as_str()).block(
                //         Block::new()
                //             .title_top("Group")
                //             .borders(Borders::all())
                //             .border_type(BorderType::Rounded)
                //             .style(Style::new().fg((&task.group.color).into())),
                //     ),
                //     group_priority[0],
                // );

                // // Priority
                // frame.render_widget(
                //     Paragraph::new(Into::<String>::into(task.priority)).block(
                //         Block::new()
                //             .title_top("Priority")
                //             .borders(Borders::all())
                //             .border_type(BorderType::Rounded)
                //             .style({
                //                 match task.priority {
                //                     Priority::Far => {
                //                         Style::new().fg(ratatui::style::Color::LightBlue)
                //                     }
                //                     Priority::Low => Style::new().fg(ratatui::style::Color::Blue),
                //                     Priority::Medium => {
                //                         Style::new().fg(ratatui::style::Color::Yellow)
                //                     }
                //                     Priority::High => {
                //                         Style::new().fg(ratatui::style::Color::LightRed)
                //                     }
                //                     Priority::Asap => Style::new().fg(ratatui::style::Color::Red),
                //                 }
                //             }),
                //     ),
                //     group_priority[1],
                // );

                // // Description
                // frame.render_widget(
                //     Paragraph::new(task.description.clone()).block(
                //         Block::new()
                //             .title_top("Description")
                //             .borders(Borders::all())
                //             .border_type(BorderType::Rounded),
                //     ),
                //     task_layout[2],
                // );

                // let completion_due = Layout::new(
                //     Direction::Horizontal,
                //     [Constraint::Percentage(50), Constraint::Percentage(50)],
                // )
                // .split(task_layout[3]);

                // let completion_symbol = if task.completed {
                //     " ✅ Awesome"
                // } else {
                //     " ❌ Get to work cornball"
                // };
                // // Completion status
                // frame.render_widget(
                //     Paragraph::new(completion_symbol).block({
                //         let block = Block::new()
                //             .title_top("Completed")
                //             .borders(Borders::all())
                //             .border_type(BorderType::Rounded);

                //         let style = if task.completed {
                //             Style::new().fg(Color::Green)
                //         } else {
                //             Style::new().fg(Color::Red)
                //         };

                //         block.style(style)
                //     }),
                //     completion_due[0],
                // );

                // // Due Date
                // frame.render_widget(
                //     Paragraph::new(format!("{:?}", task.due)).block(
                //         Block::new()
                //             .title_top("Due")
                //             .borders(Borders::all())
                //             .border_type(BorderType::Rounded),
                //     ),
                //     completion_due[1],
                // );
            }
            Some(Selection::Group(ref group)) => {
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
                frame.render_widget(
                    Paragraph::new(group.name.as_str()).block(
                        Block::new()
                            .title_top("Name")
                            .borders(Borders::all())
                            .border_type(BorderType::Rounded),
                    ),
                    group_layout[0],
                );
                // Group color:
                frame.render_widget(
                    Paragraph::new(group.color.as_str()).block(
                        Block::new()
                            .title_top("Color")
                            .borders(Borders::all())
                            .border_type(BorderType::Rounded)
                            .style(Style::new().fg(group.color.clone().into())),
                    ),
                    group_layout[1],
                );
            }
            None => {
                frame.render_widget(Paragraph::new("Please perform a Selection!"), area);

                // TODO: xd
            }
        }

        Ok(())
    }
}
