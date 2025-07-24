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
use state::State;
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, app::Mode, config::Config, tree::TarsTreeHandle};
use color_eyre::Result;

use super::{Component, frame_block};

mod state;

#[derive(Debug)]
/// Component that shows all the tasks within the current scope, ordered by priority.
pub struct TodoList<'a> {
    command_tx: Option<UnboundedSender<Action>>,
    client: TarsClient,
    config: Config,
    state: State<'a>,
    tree_handle: TarsTreeHandle,
}

impl<'a> TodoList<'a> {
    pub async fn new(client: &TarsClient, tree_handle: TarsTreeHandle) -> Result<Self> {
        let tree = tree_handle.read().await;
        let pot = tree.traverse_root();
        let (selection, _) = pot.get(if pot.len() >= 2 { 1 } else { 0 }).unwrap().clone();
        let selection = selection.clone();
        let scope = tree.root_node_id().unwrap().clone();

        let state = State::new(false, scope, selection, tree_handle.clone()).await;

        Ok(Self {
            command_tx: Default::default(),
            config: Default::default(),
            client: client.clone(),
            tree_handle: tree_handle.clone(),
            state,
        })
    }

    fn mode(&self) -> Mode {
        Mode::TodoList
    }
}

#[async_trait]
impl Component for TodoList<'_> {
    async fn init(
        &mut self,
        _area: ratatui::prelude::Size,
        default_mode: Mode,
    ) -> color_eyre::eyre::Result<()> {
        if default_mode == self.mode() {
            self.state.active = true
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
                self.state.active = true;

                Ok(Some(Action::Select(self.state.get_selected_id().clone())))
            }
            Action::SwitchTo(_) => {
                self.state.active = false;
                Ok(None)
            }
            Action::ScopeUpdate(scope) => {
                self.state.set_scope(scope).await;
                Ok(None)
            }

            Action::Refresh => {
                self.state.calculate_draw_info().await;
                Ok(None)
            }

            Action::Select(id) => {
                self.state.set_selection(id).await;
                Ok(None)
            }

            _ => Ok(None),
        }
    }
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if !self.state.active {
            return Ok(None);
        }

        match key.code {
            KeyCode::Char('j') => {
                if let Some((next_id, _)) = self
                    .state
                    .get_tasks()
                    .get(*self.state.get_selected_idx() as usize + 1)
                {
                    return Ok(Some(Action::Select(next_id.clone())));
                }

                Ok(None)
            }
            KeyCode::Char('k') => {
                if let Some((prev_id, _)) = self
                    .state
                    .get_tasks()
                    .get((*self.state.get_selected_idx() as usize).saturating_sub(1))
                {
                    return Ok(Some(Action::Select(prev_id.clone())));
                }

                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn draw(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) -> Result<()> {
        frame.render_widget(frame_block(self.state.active, self.mode()), area);

        let area = Layout::new(Direction::Vertical, [Constraint::Percentage(100)])
            .horizontal_margin(2)
            .vertical_margin(1)
            .split(area)[0];
        let draw_info = self.state.get_draw_info();

        let rects = draw_info.line_layout.split(area);

        for (line, rect) in draw_info.lines.iter().zip(rects.iter()) {
            let parts = line.layout.split(*rect);

            let task_rect = parts[0];
            let group_rect = parts[1];

            frame.render_widget(&line.task, task_rect);
            frame.render_widget(&line.group, group_rect);
        }

        Ok(())
    }
}
