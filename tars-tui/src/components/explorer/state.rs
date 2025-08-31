use async_recursion::async_recursion;
use common::types::Color;
use id_tree::{Node, NodeId};
use std::collections::HashMap;
use tui_scrollview::ScrollViewState;

use ratatui::style::Modifier;
use ratatui::{layout::Layout, text::Text, widgets::Paragraph};
use ratatui::{
    layout::{Constraint, Direction},
    style::{Color as RatColor, Style, Stylize},
};
use tracing::debug;

use crate::tree::TarsTreeHandle;
use crate::tree::{TarsKind, TarsNode};

#[derive(Debug, Clone)]
pub struct State<'a> {
    pub active: bool,
    show_completed: bool,
    scope: NodeId,
    selection: Selection,
    pub tree_handle: TarsTreeHandle,
    draw_info: Option<DrawInfo<'a>>,
    // pot: Vec<(NodeId, &'a Node<TarsNode>)>,
    pub scroll_state: ScrollViewState,

    //TODO: scroll_state cache
    pub frame_height: u16,
}

#[derive(Debug, Clone)]
pub struct DrawInfo<'a> {
    pub entries: Vec<(Paragraph<'a>, Layout)>,
    pub entries_layout: Layout,

    pub breadcrumbs: Vec<Text<'a>>,
    pub breadcrumb_layout: Layout,
}

#[derive(Debug, Clone)]
struct Selection {
    id: NodeId,
    idx: usize,
}

impl<'a> State<'a> {
    pub async fn new(
        active: bool,
        scope: NodeId,
        selection: NodeId,

        tree_handle: TarsTreeHandle,
    ) -> Self {
        let selection = Selection {
            id: selection,
            idx: 0,
        };

        let mut state = Self {
            active,
            scope,
            selection,
            tree_handle,
            draw_info: None,
            show_completed: false,
            frame_height: 80,
            scroll_state: Default::default(), // pot,
        };

        state.calculate_draw_info().await;

        state
    }

    pub async fn calculate_draw_info(&mut self) {
        let tree = self.tree_handle.read().await;

        let breadcrumbs_and_constraints = {
            let mut ancestors: Vec<(Text, Constraint)> = tree
                .ancestors(&self.scope)
                .expect("ancestors should be valid")
                .map(|ancestor| {
                    let (name, color) = {
                        match ancestor.data().kind {
                            TarsKind::Root(_) => (
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
            ancestors
        };

        let breadcrumb_layout = {
            let constraints: Vec<Constraint> = breadcrumbs_and_constraints
                .iter()
                .map(|(_, c)| *c)
                .collect();

            Layout::new(Direction::Horizontal, constraints)
        };

        let breadcrumbs = breadcrumbs_and_constraints
            .into_iter()
            .map(|(b, _)| b)
            .collect();

        let entries: Vec<(Paragraph<'_>, Layout)> = {
            let raw_render_list = self.generate_render_list().await;

            let render_list: Vec<(usize, &(NodeId, Node<TarsNode>))> =
                raw_render_list.iter().enumerate().collect();

            // now we validate selection
            if tree.get(self.get_selected_id()).is_err() {
                let (_, (id, _)) = render_list
                    .get(self.selection.idx.saturating_sub(1))
                    .unwrap_or(render_list.last().expect("why is there nothing to render"));

                self.selection.id = id.clone();
            };

            render_list
                .iter()
                .map(|(i, (entry_id, entry))| {
                    let (mut style, postfix) = if self.selection.id == entry_id.clone() {
                        self.selection.idx = *i;
                        (Style::new().bold().italic(), "*")
                    } else {
                        (Style::new(), "")
                    };

                    let widget = match entry.data().kind {
                        TarsKind::Root(_) => Paragraph::new("SHOULDNTBEPOSSIBLE"),
                        TarsKind::Task(ref t) => {
                            if t.completed {
                                style = style.add_modifier(Modifier::CROSSED_OUT);
                            }

                            Paragraph::new(format!("{}    {postfix}", *t.name))
                                .style(style.fg(t.group.color.as_ref().into()))
                        }

                        TarsKind::Group(ref g) => {
                            Paragraph::new(format!("{}    {postfix}", *g.name))
                                .style(style.fg(RatColor::Black).bg(g.color.as_ref().into()))
                        }
                    };

                    let rel_depth = tree.get(&self.scope).unwrap().data().depth;

                    let layout = Layout::new(
                        Direction::Horizontal,
                        [
                            Constraint::Min((entry.data().depth * 2).saturating_sub(rel_depth)),
                            Constraint::Percentage(100),
                        ],
                    );

                    (widget, layout)
                })
                .collect()
        };

        let entries_layout = {
            let constraints: Vec<Constraint> = entries.iter().map(|_| Constraint::Max(1)).collect();
            Layout::new(Direction::Vertical, constraints)
        };

        self.draw_info = Some(DrawInfo {
            breadcrumb_layout,
            breadcrumbs,
            entries,
            entries_layout,
        });

        debug!("updated draw info! {:#?}", self.draw_info);
    }

    pub fn get_scope(&self) -> &NodeId {
        &self.scope
    }

    pub async fn toggle_show_completed(&mut self) {
        self.show_completed = !self.show_completed;
        //TODO: move selection to nearest uncompleted entry when coming out of show_completed
        // or actually just show groups whose tasks have all been completed
        self.calculate_draw_info().await;
    }

    pub async fn set_scope(&mut self, scope: NodeId) {
        self.scope = scope;
        self.calculate_draw_info().await;
    }

    pub fn get_selected_id(&self) -> &NodeId {
        &self.selection.id
    }

    #[allow(unused)]
    pub fn get_selected_idx(&self) -> &usize {
        &self.selection.idx
    }

    pub async fn set_selection(&mut self, selection: NodeId) {
        self.selection.id = selection;

        self.calculate_draw_info().await;
    }

    pub async fn generate_render_list(&self) -> Vec<(NodeId, Node<TarsNode>)> {
        let tree = self.tree_handle.read().await;

        let mut memo = HashMap::new();

        let mut pot = Vec::new();

        for (id, node) in tree.traverse(&self.scope) {
            match node.data().kind {
                // we dont want to render the node
                TarsKind::Root(_) => {}
                TarsKind::Task(ref t) => {
                    if self.show_completed || !t.completed {
                        pot.push((id, node.clone()));
                    }
                }
                TarsKind::Group(ref g) => {
                    if self.show_completed
                        || self
                            .render_group(&tree.translate_id_to_node_id(&g.id).unwrap(), &mut memo)
                            .await
                    {
                        pot.push((id, node.clone()));
                    };
                }
            }
        }

        pot
    }

    #[async_recursion]
    async fn render_group(&self, group_id: &NodeId, memo: &mut HashMap<NodeId, bool>) -> bool {
        if let Some(result) = memo.get(group_id) {
            return *result;
        };
        let tree = self.tree_handle.read().await;

        let group = tree.get(group_id).unwrap();

        let mut exists_uncompleted_task = Some(false);

        let children = group.children();

        if children.is_empty() {
            exists_uncompleted_task = None;
        }

        for child_id in group.children() {
            let child = tree.get(child_id).unwrap();

            match child.data().kind {
                TarsKind::Task(ref t) => {
                    if !t.completed {
                        exists_uncompleted_task = Some(true);
                    }
                }

                TarsKind::Group(ref g) => {
                    if !exists_uncompleted_task.unwrap() {
                        exists_uncompleted_task = Some(
                            self.render_group(&tree.translate_id_to_node_id(&g.id).unwrap(), memo)
                                .await,
                        )
                    }
                }
                TarsKind::Root(_) => {
                    panic!("should be an impossible sptate")
                }
            }
        }

        let res = exists_uncompleted_task.unwrap_or(true);

        memo.insert(group_id.clone(), res);
        res
    }

    pub fn get_draw_info(&self) -> &DrawInfo<'a> {
        self.draw_info.as_ref().unwrap()
    }
}
