use async_trait::async_trait;
use color_eyre::Result;
use common::{
    TarsClient,
    types::{Color, Group, Task},
};
use crossterm::event::{KeyCode, KeyEvent};
use id_tree::NodeId;
use ratatui::layout::{Constraint, Direction, Layout};
use state::State;
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

use crate::{
    action::{Action, Selection},
    app::Mode,
    config::Config,
    tree::{TarsKind, TarsTreeHandle},
};

use super::{Component, frame_block};

mod state;

#[derive(Debug)]
/// Explorer component that allows you to navigate between different groups (scopes).
pub struct Explorer<'a> {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    client: TarsClient,
    state: State<'a>,
    tree_handle: TarsTreeHandle,
}

impl<'a> Explorer<'a> {
    pub async fn new(client: &TarsClient, tree_handle: TarsTreeHandle) -> Result<Self> {
        let tree = tree_handle.read().await;
        let pot = tree.traverse_root();
        let (selection, _) = pot.get(if pot.len() >= 2 { 1 } else { 0 }).unwrap().clone();
        let selection = selection.clone();
        let scope = tree.root_node_id().unwrap().clone();

        let state = State::new(false, scope, selection, tree_handle.clone(), 0).await;

        let explorer = Self {
            command_tx: Default::default(),
            config: Default::default(),
            client: client.clone(),
            state,
            tree_handle: tree_handle.clone(),
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
            self.state.set_is_active(true)
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
            Action::SwitchTo(Mode::Explorer) => {
                self.state.set_is_active(true);
                match self
                    .state
                    .tree_handle
                    .read()
                    .await
                    .get(self.state.get_selection())?
                    .data()
                    .kind
                {
                    TarsKind::Root(_) => Ok(None),
                    TarsKind::Group(ref g) => Ok(Some(Action::Select(Selection::Group(g.clone())))),
                    TarsKind::Task(ref t) => Ok(Some(Action::Select(Selection::Task(t.clone())))),
                }
            }
            Action::SwitchTo(_) => {
                self.state.set_is_active(false);
                Ok(None)
            }
            Action::Refresh => {
                // haha not this simple haha!
                self.state.calculate_draw_info().await;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        // vim bindings
        // j would move selection down
        // k would move selection up
        // l would move into a new scope
        // h would move into the outer scope
        if !self.state.is_active() {
            return Ok(None);
        }

        let tree = self.tree_handle.read().await;

        let pot = tree.traverse(self.state.get_scope());

        // let pot = tree.traverse_root;
        let Some((curr_idx, (_, node))) = pot
            .iter()
            .enumerate()
            .find(|(_, (id, _))| *self.state.get_selection() == *id)
        else {
            return Ok(None);
        };

        match key.code {
            KeyCode::Enter => {
                self.command_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::SwitchTo(Mode::Inspector))?;
                Ok(None)
            }

            KeyCode::Char('x') => {
                let selected = tree.get(self.state.get_selection())?.data();

                match selected.kind {
                    TarsKind::Task(ref t) => {
                        t.delete(&self.client).await?;
                    }
                    TarsKind::Group(ref g) => {
                        g.delete(&self.client).await?;
                    }
                    TarsKind::Root(_) => return Ok(None),
                };

                return Ok(Some(Action::Refresh));
            }

            KeyCode::Char('t') => {
                let parent = match tree.get(self.state.get_selection())?.data().kind {
                    TarsKind::Task(ref t) => &t.group,
                    TarsKind::Group(ref g) => g,
                    TarsKind::Root(_) => return Ok(None),
                };

                let _ = Task::new(
                    &self.client,
                    parent,
                    "new task",
                    common::types::Priority::Far,
                    "",
                    None,
                )
                .await?;

                Ok(Some(Action::Refresh))
            }

            // this will make a root group
            KeyCode::Char('G') => {
                let parent_group = match tree.get(self.state.get_scope())?.data().kind {
                    TarsKind::Root(_) => None,
                    TarsKind::Group(ref g) => Some(g.id.clone()),
                    TarsKind::Task(_) => return Ok(None),
                };

                let _ = Group::new(
                    &self.client,
                    "new_group",
                    parent_group,
                    Color::parse_str("white")?,
                )
                .await?;

                Ok(Some(Action::Refresh))
            }

            // this will make a child of the currently selected group
            KeyCode::Char('g') => {
                let curr_node_id = match tree.get(self.state.get_selection())?.data().kind {
                    TarsKind::Task(ref t) => Some(t.group.id.clone()),
                    TarsKind::Group(ref g) => Some(g.id.clone()),
                    TarsKind::Root(_) => None,
                };

                let _ = Group::new(
                    &self.client,
                    "new_group",
                    curr_node_id,
                    Color::parse_str("white")?,
                )
                .await?;

                Ok(Some(Action::Refresh))
                // parent of the current selection
            }

            KeyCode::Char('j') => {
                info!("J pressed");

                if let Some((next_id, next_node)) = pot.get(curr_idx + 1) {
                    self.state.set_selection(next_id.clone()).await;

                    match &next_node.data().kind {
                        TarsKind::Root(_) => {}
                        TarsKind::Task(t) => {
                            info!("selected: {t:#?}!");
                            return Ok(Some(Action::Select(Selection::Task(t.clone()))));
                        }
                        TarsKind::Group(g) => {
                            info!("selected: {g:#?}!");
                            return Ok(Some(Action::Select(Selection::Group(g.clone()))));
                        }
                    };
                }

                info!("nothing!");

                Ok(None)
            }

            KeyCode::Char('k') => {
                if let Some((prev_id, prev_node)) = pot.get({
                    let Some(i) = curr_idx.checked_sub(1) else {
                        return Ok(None);
                    };
                    i
                }) {
                    if tree.root_node_id().unwrap() == prev_id {
                        return Ok(None);
                    }

                    self.state.set_selection(prev_id.clone()).await;

                    match &prev_node.data().kind {
                        TarsKind::Root(_) => {}
                        TarsKind::Task(t) => {
                            return Ok(Some(Action::Select(Selection::Task(t.clone()))));
                        }
                        TarsKind::Group(g) => {
                            return Ok(Some(Action::Select(Selection::Group(g.clone()))));
                        }
                    };
                }

                Ok(None)
            }

            KeyCode::Char('l') => {
                // all we do here is change the scope to be this new one
                let (curr_id, curr_node) = pot.get(curr_idx).unwrap();

                if let TarsKind::Group(ref g) = curr_node.data().kind {
                    self.state.set_rel_depth(curr_node.data().depth).await;
                    self.state.set_scope(curr_id.clone()).await;
                    self.state.set_selection(curr_id.clone()).await;
                    return Ok(Some(Action::ScopeUpdate(Some(g.clone()))));
                };

                Ok(None)
            }
            KeyCode::Char('h') => {
                // now we need the ancestors of this guy
                let ancestors: Vec<&NodeId> = tree.ancestor_ids(self.state.get_scope())?.collect();
                if let Some(parent) = ancestors.first() {
                    self.state.set_scope((*parent).clone()).await;
                    let parent_node = tree.get(parent)?;

                    match parent_node.data().kind {
                        TarsKind::Root(_) => {
                            self.state.set_rel_depth(parent_node.data().depth).await;
                            return Ok(Some(Action::ScopeUpdate(None)));
                        }
                        TarsKind::Group(ref g) => {
                            self.state.set_rel_depth(parent_node.data().depth).await;
                            return Ok(Some(Action::ScopeUpdate(Some(g.clone()))));
                        }
                        _ => {
                            return Ok(None);
                        }
                    }
                };
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
        frame.render_widget(frame_block(self.state.is_active(), self.mode()), area);

        let areas = Layout::new(
            Direction::Vertical,
            [Constraint::Percentage(100), Constraint::Min(1)],
        )
        .horizontal_margin(2)
        .vertical_margin(1)
        .split(area);

        let entries_area = areas[0];

        let breadcrumbs_area = areas[1];

        let draw_info = self.state.get_draw_info();

        let crumb_rects = draw_info.breadcrumb_layout.split(breadcrumbs_area);

        for (crumb, crumb_rect) in draw_info.breadcrumbs.iter().zip(crumb_rects.iter()) {
            frame.render_widget(crumb, *crumb_rect);
        }

        let entries_rects = draw_info.entries_layout.split(entries_area);

        for ((entry, depth_offset_layout), entry_rect) in
            draw_info.entries.iter().zip(entries_rects.iter())
        {
            frame.render_widget(entry, depth_offset_layout.split(*entry_rect)[1]);
        }

        Ok(())
    }
}
