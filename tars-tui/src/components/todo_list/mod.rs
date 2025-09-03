use async_trait::async_trait;
use common::TarsClient;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Size};
use state::State;
use tokio::sync::mpsc::UnboundedSender;
use tui_scrollview::{ScrollView, ScrollbarVisibility};

use crate::{
    action::{Action, Signal},
    app::Mode,
    config::Config,
    tree::TarsTreeHandle,
};
use color_eyre::Result;

use super::{Component, frame_block};

mod state;

#[derive(Debug)]
/// Component that shows all the tasks within the current scope, ordered by priority.
pub struct TodoList<'a> {
    signal_tx: Option<UnboundedSender<Signal>>,
    config: Config,
    state: State<'a>,
    _tree_handle: TarsTreeHandle,
}

impl<'a> TodoList<'a> {
    pub async fn new(client: &TarsClient, tree_handle: TarsTreeHandle) -> Result<Self> {
        let tree = tree_handle.read().await;
        let pot = tree.traverse_root();
        let (selection, _) = pot.get(if pot.len() >= 2 { 1 } else { 0 }).unwrap().clone();
        let selection = selection.clone();
        let scope = tree.root_node_id().unwrap().clone();

        let state =
            State::new(false, scope, selection, tree_handle.clone(), client.clone()).await?;

        Ok(Self {
            signal_tx: Default::default(),
            config: Default::default(),
            _tree_handle: tree_handle.clone(),
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
    fn register_signal_handler(
        &mut self,
        tx: UnboundedSender<Signal>,
    ) -> color_eyre::eyre::Result<()> {
        self.signal_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> color_eyre::eyre::Result<()> {
        self.config = config;
        Ok(())
    }

    async fn update(&mut self, action: Signal) -> color_eyre::eyre::Result<Option<Signal>> {
        match action.clone() {
            Signal::Tick => Ok(None),
            Signal::Render => Ok(None),
            Signal::Action(Action::SwitchTo(Mode::TodoList)) => {
                self.state.active = true;

                Ok(Some(Signal::Select(self.state.get_selected_id().clone())))
            }
            Signal::Action(Action::SwitchTo(_)) => {
                self.state.active = false;
                self.state.calculate_draw_info().await?;
                Ok(None)
            }
            Signal::ScopeUpdate(scope) => {
                self.state.set_scope(scope).await?;
                Ok(None)
            }

            Signal::Refresh => {
                self.state.calculate_draw_info().await?;
                Ok(None)
            }

            Signal::Select(id) => {
                if self.state.active {
                    self.state.set_selection(id).await?;
                }
                Ok(None)
            }

            Signal::Action(action) => {
                if !self.state.active {
                    return Ok(None);
                }

                let tasks = self.state.get_tasks();
                let sel_idx = *self.state.get_selected_idx();
                match action {
                    Action::MoveDown => {
                        if let Some((next_id, _, _)) = tasks.get(sel_idx + 1) {
                            let next_id = next_id.clone();
                            let offset = self.state.scroll_state.offset().y as usize;

                            if sel_idx - offset + self.config.config.scroll_offset as usize
                                >= self.state.frame_height as usize
                            {
                                self.state.scroll_state.scroll_down();
                            }

                            return Ok(Some(Signal::Select(next_id)));
                        }

                        Ok(None)
                    }

                    Action::MoveUp => {
                        if let Some((prev_id, _, _)) = tasks.get(sel_idx.saturating_sub(1)) {
                            let prev_id = prev_id.clone();
                            let offset = self.state.scroll_state.offset().y as usize;
                            if sel_idx - offset < self.config.config.scroll_offset as usize {
                                self.state.scroll_state.scroll_up();
                            }

                            return Ok(Some(Signal::Select(prev_id)));
                        }

                        if sel_idx + 10 >= self.state.frame_height as usize {
                            self.state.scroll_state.scroll_down();
                        }

                        Ok(None)
                    }
                    _ => Ok(None),
                }
            }

            _ => Ok(None),
        }
    }
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Signal>> {
        if !self.state.active {
            return Ok(None);
        }

        match key.code {
            KeyCode::Enter => {
                self.signal_tx
                    .as_ref()
                    .unwrap()
                    .send(Signal::Action(Action::SwitchTo(Mode::Inspector)))?;
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

        self.state.frame_height = area.height;

        let mut scroll_view =
            ScrollView::new(Size::new(area.width, self.state.get_tasks().len() as u16))
                .horizontal_scrollbar_visibility(ScrollbarVisibility::Never)
                .vertical_scrollbar_visibility(ScrollbarVisibility::Automatic);

        let scroll_area = scroll_view.area();

        let draw_info = self.state.get_draw_info();

        let rects = draw_info.line_layout.split(scroll_area);

        for (line, rect) in draw_info.lines.iter().zip(rects.iter()) {
            let parts = line.layout.split(*rect);

            let task_rect = parts[0];
            let group_rect = parts[1];
            let prio_date_rect = parts[2];

            scroll_view.render_widget(&line.task, task_rect);
            scroll_view.render_widget(&line.group, group_rect);
            scroll_view.render_widget(&line.prio_date, prio_date_rect);
        }

        frame.render_stateful_widget(scroll_view, area, &mut self.state.scroll_state);

        Ok(())
    }
}
