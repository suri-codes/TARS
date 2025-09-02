use async_trait::async_trait;
use color_eyre::{Result, eyre::OptionExt};
use common::{
    TarsClient,
    types::{Color, Group, Id, Task},
};
use crossterm::event::{KeyCode, KeyEvent};
use id_tree::NodeId;
use ratatui::layout::{Constraint, Direction, Layout, Size};
use state::State;
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;
use tui_scrollview::{ScrollView, ScrollbarVisibility};

use crate::{
    action::{Action, Signal},
    app::Mode,
    config::Config,
    tree::{TarsKind, TarsTreeHandle},
};

use super::{Component, frame_block};

mod state;

#[derive(Debug)]
/// Explorer component that allows you to navigate between different groups (scopes).
pub struct Explorer<'a> {
    signal_tx: Option<UnboundedSender<Signal>>,
    config: Config,
    client: TarsClient,
    state: State<'a>,
    tree_handle: TarsTreeHandle,
    on_update: OnUpdate,
}

#[derive(Debug)]
enum OnUpdate {
    None,
    Select(Id),
}

impl<'a> Explorer<'a> {
    pub async fn new(client: &TarsClient, tree_handle: TarsTreeHandle) -> Result<Self> {
        let tree = tree_handle.read().await;
        let pot = tree.traverse_root();
        let (selection, _) = pot.get(if pot.len() >= 2 { 1 } else { 0 }).unwrap().clone();
        let selection = selection.clone();
        let scope = tree.root_node_id().unwrap().clone();

        let state = State::new(false, scope, selection, tree_handle.clone()).await;

        let explorer = Self {
            signal_tx: Default::default(),
            config: Default::default(),
            client: client.clone(),
            state,
            tree_handle: tree_handle.clone(),
            on_update: OnUpdate::None,
        };

        Ok(explorer)
    }

    fn mode(&self) -> Mode {
        Mode::Explorer
    }
}

#[async_trait]
impl<'a> Component for Explorer<'a> {
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
            Signal::Select(id) => {
                if self.state.active {
                    self.state.set_selection(id).await;
                }

                Ok(None)
            }

            Signal::Update => match self.on_update {
                OnUpdate::Select(ref id) => {
                    let tree = self.tree_handle.read().await;

                    let sel_idx = *self.state.get_selected_idx();
                    let offset = self.state.scroll_state.offset().y as usize;
                    if sel_idx - offset + self.config.config.scroll_offset as usize
                        >= self.state.frame_height as usize
                    {
                        self.state.scroll_state.scroll_down();
                    }

                    let node_id = tree
                        .translate_id_to_node_id(id)
                        .ok_or_eyre("missing node id")?;

                    let command_tx = self.signal_tx.as_ref().expect("should exist");

                    command_tx.send(Signal::Select(node_id.clone()))?;
                    self.on_update = OnUpdate::None;

                    Ok(Some(Signal::Action(Action::SwitchTo(Mode::Inspector))))
                }
                OnUpdate::None => Ok(None),
            },

            Signal::Action(Action::SwitchTo(Mode::Explorer)) => {
                self.state.active = true;

                Ok(Some(Signal::Select(self.state.get_selected_id().clone())))
            }
            Signal::Action(Action::SwitchTo(_)) => {
                self.state.active = false;
                Ok(None)
            }
            Signal::Refresh => {
                self.state.calculate_draw_info().await;
                Ok(None)
            }

            Signal::Action(action) => {
                if !self.state.active {
                    return Ok(None);
                }

                info!("Processing {}", action);

                match action {
                    Action::ToggleShowFinished => {
                        self.state.toggle_show_finished().await;
                        Ok(None)
                    }

                    Action::Delete => {
                        let tree = self.tree_handle.read().await;
                        let selected = tree.get(self.state.get_selected_id())?.data();
                        let render_list = self.state.generate_render_list().await;

                        match selected.kind {
                            TarsKind::Task(ref t) => {
                                t.delete(&self.client).await?;
                            }
                            TarsKind::Group(ref g) => {
                                g.delete(&self.client).await?;
                            }
                            TarsKind::Root(_) => return Ok(None),
                        };

                        let (next_node, _) = {
                            let next_node =
                                render_list.get(self.state.get_selected_idx().saturating_add(1));

                            match next_node {
                                Some(id) => id,
                                None => {
                                    match render_list
                                        .get(self.state.get_selected_idx().saturating_sub(1))
                                    {
                                        Some(id) => id,
                                        None => return Ok(None),
                                    }
                                }
                            }
                        };

                        self.state.set_selection(next_node.clone()).await;

                        return Ok(None);
                    }

                    Action::NewTask => {
                        let tree = self.tree_handle.read().await;
                        let parent = match tree.get(self.state.get_selected_id())?.data().kind {
                            TarsKind::Task(ref t) => &t.group,
                            TarsKind::Group(ref g) => g,
                            TarsKind::Root(_) => return Ok(None),
                        };

                        let t = Task::new(
                            &self.client,
                            parent,
                            "new task",
                            common::types::Priority::Medium,
                            "",
                            None,
                        )
                        .await?;

                        self.on_update = OnUpdate::Select(t.id.clone());

                        Ok(Some(Signal::Refresh))
                    }

                    Action::NewGroup => {
                        let tree = self.tree_handle.read().await;

                        let parent_group = match tree.get(self.state.get_scope())?.data().kind {
                            TarsKind::Root(_) => None,
                            TarsKind::Group(ref g) => Some(g.id.clone()),
                            TarsKind::Task(_) => return Ok(None),
                        };

                        let g = Group::new(
                            &self.client,
                            "new_group",
                            parent_group,
                            Default::default(),
                            Color::random(),
                        )
                        .await?;
                        self.on_update = OnUpdate::Select(g.id.clone());

                        Ok(Some(Signal::Refresh))
                    }

                    Action::NewSubGroup => {
                        let tree = self.tree_handle.read().await;
                        let curr_node_id = match tree.get(self.state.get_selected_id())?.data().kind
                        {
                            TarsKind::Task(ref t) => Some(t.group.id.clone()),
                            TarsKind::Group(ref g) => Some(g.id.clone()),
                            TarsKind::Root(_) => None,
                        };

                        let g = Group::new(
                            &self.client,
                            "new_group",
                            curr_node_id,
                            Default::default(),
                            Color::random(),
                        )
                        .await?;

                        self.on_update = OnUpdate::Select(g.id.clone());
                        Ok(Some(Signal::Refresh))
                    }

                    Action::MoveDown => {
                        let render_list = self.state.generate_render_list().await;
                        let sel_idx = *self.state.get_selected_idx();
                        if let Some((next_id, _)) = render_list.get(sel_idx + 1) {
                            let offset = self.state.scroll_state.offset().y as usize;

                            info!(
                                "configured scroll offset: {}",
                                self.config.config.scroll_offset
                            );

                            if sel_idx - offset + self.config.config.scroll_offset as usize
                                >= self.state.frame_height as usize
                            {
                                self.state.scroll_state.scroll_down();
                            }

                            self.state.set_selection(next_id.clone()).await;
                            return Ok(Some(Signal::Select(next_id.clone())));
                        }

                        Ok(None)
                    }
                    Action::MoveUp => {
                        let render_list = self.state.generate_render_list().await;
                        let tree = self.tree_handle.read().await;
                        let sel_idx = *self.state.get_selected_idx();
                        if let Some((prev_id, _)) = render_list.get({
                            let offset = self.state.scroll_state.offset().y as usize;
                            if sel_idx - offset < self.config.config.scroll_offset as usize {
                                self.state.scroll_state.scroll_up();
                            }

                            let Some(i) = sel_idx.checked_sub(1) else {
                                return Ok(None);
                            };
                            i
                        }) {
                            if tree.root_node_id().unwrap() == prev_id {
                                return Ok(None);
                            }

                            self.state.set_selection(prev_id.clone()).await;
                            return Ok(Some(Signal::Select(prev_id.clone())));
                        }

                        Ok(None)
                    }
                    Action::MoveInto => {
                        let render_list = self.state.generate_render_list().await;
                        let curr_idx = self.state.get_selected_idx();
                        // all we do here is change the scope to be this new one
                        let (curr_id, curr_node) = render_list.get(*curr_idx).unwrap();

                        if let TarsKind::Group(_) = curr_node.data().kind {
                            self.state.set_scope(curr_id.clone()).await;
                            self.state.set_selection(curr_id.clone()).await;
                            return Ok(Some(Signal::ScopeUpdate(curr_id.clone())));
                        };

                        Ok(None)
                    }
                    Action::MoveOutOf => {
                        let tree = self.tree_handle.read().await;
                        // now we need the ancestors of this guy
                        let ancestors: Vec<&NodeId> =
                            tree.ancestor_ids(self.state.get_scope())?.collect();
                        if let Some(parent) = ancestors.first() {
                            self.state.set_scope((*parent).clone()).await;

                            return Ok(Some(Signal::ScopeUpdate((*parent).clone())));
                        };
                        Ok(None)
                    }
                    _ => Ok(None),
                }
            }

            _ => Ok(None),
        }
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Signal>> {
        // vim bindings
        // j would move selection down
        // k would move selection up
        // l would move into a new scope
        // h would move into the outer scope
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

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::eyre::Result<()> {
        frame.render_widget(frame_block(self.state.active, self.mode()), area);

        let areas = Layout::new(
            Direction::Vertical,
            [Constraint::Percentage(100), Constraint::Min(1)],
        )
        .horizontal_margin(2)
        .vertical_margin(1)
        .split(area);

        let entries_area = areas[0];

        let breadcrumbs_area = areas[1];

        self.state.frame_height = entries_area.height;

        let draw_info = self.state.get_draw_info();

        let crumb_rects = draw_info.breadcrumb_layout.split(breadcrumbs_area);

        for (crumb, crumb_rect) in draw_info.breadcrumbs.iter().zip(crumb_rects.iter()) {
            frame.render_widget(crumb, *crumb_rect);
        }

        let mut scroll_view =
            ScrollView::new(Size::new(area.width, draw_info.entries.len() as u16))
                .horizontal_scrollbar_visibility(ScrollbarVisibility::Never)
                .vertical_scrollbar_visibility(ScrollbarVisibility::Automatic);

        let scroll_area = scroll_view.area();

        let entries_rects = draw_info.entries_layout.split(scroll_area);

        for ((entry, depth_offset_layout), entry_rect) in
            draw_info.entries.iter().zip(entries_rects.iter())
        {
            scroll_view.render_widget(entry, depth_offset_layout.split(*entry_rect)[1]);
        }

        frame.render_stateful_widget(scroll_view, entries_area, &mut self.state.scroll_state);

        Ok(())
    }
}
