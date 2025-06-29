use std::{collections::HashMap, fmt::Debug};

use async_trait::async_trait;
use color_eyre::Result;
use common::{
    TarsClient,
    types::{Group, Id, Task, TaskFetchOptions},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, BorderType, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

use crate::{
    action::{Action, Selection},
    app::Mode,
    config::Config,
};

use super::{Component, frame_block};

#[derive(Debug)]
/// Explorer component that allows you to navigate between different groups (scopes).
pub struct Explorer {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    client: TarsClient,
    active: bool,
    groups: Vec<Group>,
    tasks: Vec<Task>,
    entries: Vec<TodoWidget>,
    root: Option<Id>,
    selection: Vec<u16>,
}

#[derive(Debug, Clone)]
struct TodoWidget {
    kind: TodoWidgetKind,
    depth: u16,
}

#[expect(dead_code)]
#[derive(Debug, Clone)]
enum TodoWidgetKind {
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
            client: client.clone(),
            active: false,
            groups,
            tasks,
            entries: Vec::new(),
            root: None,
            selection: vec![0],
        })
    }

    fn mode(&self) -> Mode {
        Mode::Explorer
    }

    async fn process(&mut self) -> Result<()> {
        let root_groups: Vec<&Group> = self
            .groups
            .iter()
            .filter(|e| e.parent_id == self.root)
            .collect();

        let g_to_g = {
            let mut map: HashMap<Id, Vec<Group>> = HashMap::new();

            for group in Group::fetch_all(&self.client).await? {
                let Some(ref parent_id) = group.parent_id else {
                    continue;
                };

                let children = match map.get_mut(parent_id) {
                    Some(e) => e,
                    None => {
                        map.insert(parent_id.clone(), Vec::new());
                        map.get_mut(parent_id).unwrap()
                    }
                };

                children.push(group)
            }

            map
        };

        let g_to_t = {
            let mut map: HashMap<Id, Vec<Task>> = HashMap::new();

            for task in Task::fetch(&self.client, TaskFetchOptions::All).await? {
                let children = match map.get_mut(&task.group.id) {
                    Some(e) => e,
                    None => {
                        map.insert(task.group.id.clone(), Vec::new());
                        map.get_mut(&task.group.id).unwrap()
                    }
                };

                children.push(task)
            }

            map
        };

        let mut new_widgets: Vec<TodoWidget> = vec![];

        for group in root_groups {
            let mut depth = 0;

            Explorer::add_children_of_group(&mut new_widgets, group, &g_to_g, &g_to_t, &mut depth);
        }

        self.entries = new_widgets;

        Ok(())
    }

    fn add_children_of_group(
        widgets: &mut Vec<TodoWidget>,
        group: &Group,
        g_to_g: &HashMap<Id, Vec<Group>>,
        g_to_t: &HashMap<Id, Vec<Task>>,
        depth: &mut u16,
    ) {
        // add the group first
        widgets.push(TodoWidget {
            kind: TodoWidgetKind::Group(group.clone()),
            depth: *depth,
        });

        *depth += 1;

        if let Some(tasks) = g_to_t.get(&group.id) {
            for task in tasks {
                widgets.push(TodoWidget {
                    kind: TodoWidgetKind::Task(task.clone()),
                    depth: *depth,
                });
            }
        }

        if let Some(groups) = g_to_g.get(&group.id) {
            *depth += 1;
            for group in groups {
                Explorer::add_children_of_group(widgets, group, g_to_g, g_to_t, depth);
            }
        }
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
            Action::Tick => {
                self.process().await?;
            }
            Action::Render => {}
            Action::SwitchTo(Mode::Explorer) => self.active = true,
            Action::SwitchTo(_) => self.active = false,
            _ => {}
        }
        Ok(None)
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        info!("received key event: {key:#?}");
        // vim bindings
        // j would move selection down
        // k would move selection up
        // l would move into a new scope
        // h would move into the outer scope
        if !self.active {
            return Ok(None);
        }

        match key.code {
            KeyCode::Char('j') => {
                info!("J pressed");
                if let Some(next) = self
                    .entries
                    .get(*self.selection.last().unwrap() as usize + 1)
                {
                    *self.selection.last_mut().unwrap() += 1;

                    match &next.kind {
                        TodoWidgetKind::Task(t) => {
                            info!("selected: {t:#?}!");
                            return Ok(Some(Action::Select(Selection::Task(t.clone()))));
                        }
                        TodoWidgetKind::Group(g) => {
                            info!("selected: {g:#?}!");
                            return Ok(Some(Action::Select(Selection::Group(g.clone()))));
                        }
                    };
                }

                info!("nothing!");

                Ok(None)
            }

            KeyCode::Char('k') => {
                if let Some(prev) = self.entries.get({
                    if let Some(i) = (*self.selection.last().unwrap_or(&0) as usize).checked_sub(1)
                    {
                        i
                    } else {
                        return Ok(None);
                    }
                }) {
                    *self.selection.last_mut().unwrap() -= 1;

                    match &prev.kind {
                        TodoWidgetKind::Task(t) => {
                            return Ok(Some(Action::Select(Selection::Task(t.clone()))));
                        }
                        TodoWidgetKind::Group(g) => {
                            return Ok(Some(Action::Select(Selection::Group(g.clone()))));
                        }
                    };
                }

                Ok(None)
            }

            KeyCode::Char('l') => {
                if let TodoWidgetKind::Group(ref g) = self
                    .entries
                    .get(*self.selection.last().unwrap() as usize)
                    .unwrap()
                    .kind
                {
                    self.root = Some(g.id.clone());
                    self.selection.push(0);

                    return Ok(Some(Action::ScopeUpdate(Some(g.clone()))));
                };

                Ok(None)
            }
            KeyCode::Char('h') => {
                if let Some(ref root) = self.root {
                    let all_groups = Group::fetch_all(&self.client).await?;
                    let root = all_groups.iter().find(|g| g.id == *root).unwrap();

                    let Some(ref parent_id) = root.parent_id else {
                        self.root = None;
                        let _ = self.selection.pop();
                        return Ok(Some(Action::ScopeUpdate(None)));
                    };

                    let parent = all_groups
                        .iter()
                        .find(|g| g.id == *parent_id)
                        .expect("this group should exist");

                    self.root = Some(parent.id.clone());
                    self.selection.push(0);
                    return Ok(Some(Action::ScopeUpdate(Some(parent.clone()))));
                } else {
                    return Ok(None);
                }
            }

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

        let constraints: Vec<Constraint> =
            self.entries.iter().map(|_| Constraint::Max(1)).collect();

        let task_layouts = Layout::new(Direction::Vertical, constraints).split(area);
        // how am i supposed to render this shit dawg

        // need to divide up the area. algorithmically.

        // ideally top 4 tasks per group + a line that says more coming after

        // groups organized by parents

        for (i, (entry, area)) in self.entries.iter().zip(task_layouts.iter()).enumerate() {
            let (style, postfix) = if *self.selection.last().unwrap() == i as u16 {
                (Style::new().bold().italic(), "*")
                // .underlined()
                // .underline_color(Color::Black)
                // .slow_blink()
                // .rapid_blink()
            } else {
                (Style::new(), "")
            };
            let widget = match entry.kind {
                TodoWidgetKind::Task(ref t) => Paragraph::new(format!("{}    {postfix}", *t.name))
                    .style(style.fg(t.group.color.as_ref().into())),

                TodoWidgetKind::Group(ref g) => Paragraph::new(format!("{}    {postfix}", *g.name))
                    .style(style.fg(Color::Black).bg(g.color.as_ref().into())),
            };

            // pad with the depth we want
            let area = Layout::new(
                Direction::Horizontal,
                [Constraint::Min(entry.depth), Constraint::Percentage(100)],
            )
            .split(*area)[1];

            frame.render_widget(widget, area);
        }

        Ok(())
    }
}
