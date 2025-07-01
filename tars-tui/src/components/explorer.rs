use std::{collections::HashMap, fmt::Debug};

use async_trait::async_trait;
use color_eyre::Result;
use common::{
    TarsClient,
    types::{Color, Group, Id, Task, TaskFetchOptions},
};
use crossterm::event::{KeyCode, KeyEvent};
use id_tree::{InsertBehavior, Node, NodeId, Tree, TreeBuilder};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color as RatColor, Style, Stylize},
    text::Text,
    widgets::Paragraph,
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{info, warn};

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
    scope: NodeId,
    selection: NodeId,
    tree: Tree<TarsNode>,
    rel_depth: u16,
    // pot: Vec<&Node TarsNode>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum TarsKind {
    Root,
    Task(Task),
    Group(Group),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TarsNode {
    kind: TarsKind,
    // might need it when creating a new task but even then i think that
    // should just cause a node refresh so this should ultimately be removed
    parent: Option<NodeId>,
    depth: u16,
}

impl Explorer {
    pub async fn new(client: &TarsClient) -> Result<Self> {
        let tree = Self::generate_tree(client).await?;
        let root = tree.root_node_id().unwrap();

        let pot: Vec<NodeId> = tree.traverse_pre_order_ids(root).unwrap().collect();

        let explorer = Self {
            command_tx: Default::default(),
            config: Default::default(),
            client: client.clone(),
            active: false,
            scope: tree.root_node_id().unwrap().clone(),

            // if selection == 0 then its cookked
            selection: pot.get(if pot.len() >= 2 { 1 } else { 0 }).unwrap().clone(),
            tree,
            rel_depth: 0,
            // pot,
        };

        Ok(explorer)
    }

    fn mode(&self) -> Mode {
        Mode::Explorer
    }

    async fn generate_tree(client: &TarsClient) -> Result<Tree<TarsNode>> {
        let g_to_g = {
            let mut map: HashMap<Id, Vec<Group>> = HashMap::new();

            for group in Group::fetch_all(client).await? {
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

            for task in Task::fetch(client, TaskFetchOptions::All).await? {
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

        let mut tree: Tree<TarsNode> = TreeBuilder::new().build();

        let root_id: NodeId = tree.insert(
            Node::new(TarsNode {
                kind: TarsKind::Root,
                parent: None,
                depth: 0,
            }),
            InsertBehavior::AsRoot,
        )?;

        let all_groups = Group::fetch_all(client).await?;
        let root_groups: Vec<&Group> = all_groups
            .iter()
            .filter(|e| e.parent_id.is_none())
            .collect();

        for group in root_groups {
            let mut depth = 0;

            Explorer::tree_children_of_group(
                &mut tree,
                group.clone(),
                &g_to_g,
                &g_to_t,
                &mut depth,
                root_id.clone(),
            )?;
        }

        info!("{tree:#?}");

        Ok(tree)
    }

    fn tree_children_of_group(
        tree: &mut Tree<TarsNode>,
        group: Group,
        g_to_g: &HashMap<Id, Vec<Group>>,
        g_to_t: &HashMap<Id, Vec<Task>>,
        depth: &mut u16,
        parent_id: NodeId,
    ) -> Result<()> {
        // insert group into the parent group
        let group_id = tree.insert(
            Node::new(TarsNode {
                kind: TarsKind::Group(group.clone()),
                parent: Some(parent_id.clone()),
                depth: *depth,
            }),
            InsertBehavior::UnderNode(&parent_id),
        )?;
        *depth += 1;

        // now we want to add all tasks to it?
        if let Some(tasks) = g_to_t.get(&group.id) {
            for task in tasks {
                let _ = tree.insert(
                    Node::new(TarsNode {
                        kind: TarsKind::Task(task.clone()),
                        parent: Some(group_id.clone()),
                        depth: *depth,
                    }),
                    InsertBehavior::UnderNode(&group_id),
                );
            }
        }

        if let Some(child_groups) = g_to_g.get(&group.id) {
            *depth += 1;
            for child_group in child_groups {
                Explorer::tree_children_of_group(
                    tree,
                    child_group.clone(),
                    g_to_g,
                    g_to_t,
                    depth,
                    group_id.clone(),
                )?;
            }
        }
        Ok(())
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
            Action::Refresh => {
                let updated_tree = Self::generate_tree(&self.client).await?;

                let scope = self.tree.get(&self.scope)?.data();

                if let Some((_, new_scope_id)) = updated_tree
                    .traverse_pre_order(updated_tree.root_node_id().unwrap())?
                    .zip(
                        updated_tree
                            .traverse_pre_order_ids(updated_tree.root_node_id().unwrap())?,
                    )
                    .find(|(e, _)| {
                        if let TarsKind::Group(ref g1) = e.data().kind
                            && let TarsKind::Group(ref g2) = scope.kind
                        {
                            return g1.id == g2.id;
                        }

                        false
                    })
                {
                    self.scope = new_scope_id;
                } else {
                    warn!("Scope not found on tree refresh, setting to root node.");
                    self.scope = updated_tree.root_node_id().unwrap().clone();
                }

                let selection = self.tree.get(&self.selection)?.data().clone();

                if let Some((_, new_selection_id)) = updated_tree
                    .traverse_pre_order(updated_tree.root_node_id().unwrap())?
                    .zip(
                        updated_tree
                            .traverse_pre_order_ids(updated_tree.root_node_id().unwrap())?,
                    )
                    .find(|(e, _)| match (&e.data().kind, &selection.kind) {
                        (TarsKind::Task(t1), TarsKind::Task(t2)) => t1.id == t2.id,
                        (TarsKind::Group(g1), TarsKind::Group(g2)) => g1.id == g2.id,
                        _ => false,
                    })
                {
                    self.selection = new_selection_id;
                } else {
                    warn!("Selection not found on tree refresh, setting to root node.");
                    self.selection = updated_tree.root_node_id().unwrap().clone();
                }

                self.tree = updated_tree;
            }
            _ => {}
        }
        Ok(None)
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        // vim bindings
        // j would move selection down
        // k would move selection up
        // l would move into a new scope
        // h would move into the outer scope
        if !self.active {
            return Ok(None);
        }

        let pot: Vec<(NodeId, &Node<TarsNode>)> = self
            .tree
            .traverse_pre_order_ids(&self.scope)
            .unwrap()
            .zip(self.tree.traverse_pre_order(&self.scope).unwrap())
            .collect();

        let Some((curr_idx, (_, node))) = pot
            .iter()
            .enumerate()
            .find(|(_, (id, _))| self.selection == *id)
        else {
            return Ok(None);
        };

        info!("key handler curr_node: {node:#?}");

        match key.code {
            KeyCode::Char('j') => {
                info!("J pressed");
                if let Some((next_id, next_node)) = pot.get(curr_idx + 1) {
                    self.selection = next_id.clone();

                    match &next_node.data().kind {
                        TarsKind::Root => {}
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
                if let Some((prev_id, prev_node)) = pot.get(curr_idx - 1) {
                    if self.tree.root_node_id().unwrap() == prev_id {
                        return Ok(None);
                    }

                    self.selection = prev_id.clone();

                    match &prev_node.data().kind {
                        TarsKind::Root => {}
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
                    self.rel_depth = curr_node.data().depth;
                    self.scope = curr_id.clone();
                    self.selection = curr_id.clone();
                    return Ok(Some(Action::ScopeUpdate(Some(g.clone()))));
                };

                Ok(None)
            }
            KeyCode::Char('h') => {
                // now we need the ancestors of this guy
                let ancestors: Vec<&NodeId> = self.tree.ancestor_ids(&self.scope)?.collect();
                if let Some(parent) = ancestors.first() {
                    self.scope = (*parent).clone();
                    let parent_node = self.tree.get(parent)?;

                    match parent_node.data().kind {
                        TarsKind::Root => {
                            self.rel_depth = parent_node.data().depth;
                            return Ok(Some(Action::ScopeUpdate(None)));
                        }
                        TarsKind::Group(ref g) => {
                            self.rel_depth = parent_node.data().depth;
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
        frame.render_widget(frame_block(self.active, self.mode()), area);

        let areas = Layout::new(
            Direction::Vertical,
            [Constraint::Percentage(100), Constraint::Min(1)],
        )
        .horizontal_margin(2)
        .vertical_margin(1)
        .split(area);

        let area = areas[0];

        let breadcrumbs = areas[1];

        //TODO: actually have to split the breadcrumbs area into the shi.
        let mut ancestors: Vec<(Text, Constraint)> = self
            .tree
            .ancestors(&self.scope)
            .expect("ancestors should be valid")
            .map(|ancestor| {
                let (name, color) = {
                    match ancestor.data().kind {
                        TarsKind::Root => (
                            " Home ".into(),
                            TryInto::<Color>::try_into("red".to_owned()).unwrap(),
                        ),

                        TarsKind::Group(ref g) => (format!(" {} ", *g.name), g.color.clone()),

                        _ => {
                            panic!()
                        }
                    }
                };

                (
                    Text::styled(
                        name.clone(),
                        Style::new().bg(color.into()).fg(RatColor::Black),
                    ),
                    Constraint::Length(name.len() as u16),
                )
            })
            .collect();
        ancestors.reverse();

        let constraints: Vec<Constraint> = ancestors.iter().map(|(_, c)| *c).collect();

        let crumb_layout = Layout::new(Direction::Horizontal, constraints).split(breadcrumbs);

        for ((ancestor, _), area) in ancestors.iter().zip(crumb_layout.iter()) {
            frame.render_widget(ancestor, *area);
        }

        let root_node_id = self.tree.root_node_id().expect("root node id should exist");

        let pot: Vec<(NodeId, TarsNode)> = self
            .tree
            .traverse_pre_order_ids(&self.scope)?
            .zip(self.tree.traverse_pre_order(&self.scope)?)
            .enumerate()
            // if the scope is the root scope AND the element is the first one, we drop it cuz we dont want to render the root
            .filter(|(i, _)| !(self.scope == *root_node_id && *i == 0))
            .map(|(_, (node_id, node))| (node_id, node.data().clone()))
            .collect();

        let constraints: Vec<Constraint> = pot.iter().map(|_| Constraint::Max(1)).collect();

        let task_layouts = Layout::new(Direction::Vertical, constraints).split(area);
        // how am i supposed to render this shit dawg

        // need to divide up the area. algorithmically.

        // ideally top 4 tasks per group + a line that says more coming after

        // groups organized by parents
        for ((entry_id, entry), area) in pot.iter().zip(task_layouts.iter()) {
            let (style, postfix) = if self.selection == *entry_id {
                (Style::new().bold().italic(), "*")
            } else {
                (Style::new(), "")
            };
            let widget = match entry.kind {
                TarsKind::Root => Paragraph::new("SHOULDNTBEPOSSIBLE"),
                TarsKind::Task(ref t) => Paragraph::new(format!("{}    {postfix}", *t.name))
                    .style(style.fg(t.group.color.as_ref().into())),

                TarsKind::Group(ref g) => Paragraph::new(format!("{}    {postfix}", *g.name))
                    .style(style.fg(RatColor::Black).bg(g.color.as_ref().into())),
            };

            // pad with the depth we want
            let area = Layout::new(
                Direction::Horizontal,
                [
                    Constraint::Min(entry.depth - self.rel_depth),
                    Constraint::Percentage(100),
                ],
            )
            .split(*area)[1];

            frame.render_widget(widget, area);
        }

        Ok(())
    }
}
