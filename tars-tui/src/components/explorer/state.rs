use common::types::Color;
use id_tree::NodeId;

use ratatui::{layout::Layout, text::Text, widgets::Paragraph};
use ratatui::{
    layout::{Constraint, Direction},
    style::{Color as RatColor, Style, Stylize},
};
use tracing::info;

use crate::tree::TarsKind;
use crate::tree::TarsTreeHandle;

#[derive(Debug, Clone)]
pub struct State<'a> {
    active: bool,
    scope: NodeId,
    selection: Selection,
    pub tree_handle: TarsTreeHandle,
    rel_depth: u16,
    draw_info: Option<DrawInfo<'a>>,
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
    idx: u32,
}

impl<'a> State<'a> {
    pub async fn new(
        active: bool,
        scope: NodeId,
        selection: NodeId,

        tree_handle: TarsTreeHandle,
        rel_depth: u16,
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
            rel_depth,
            draw_info: None,
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
            let root_node_id = tree.root_node_id().expect("root node id should exist");

            let traverse = tree.traverse(&self.scope);

            let pot: Vec<_> = traverse
                .iter()
                .enumerate()
                // if the scope is the root scope AND the element is the first one, we drop it cuz we dont want to render the root
                .filter(|(i, _)| !(self.scope == *root_node_id && *i == 0))
                .collect();

            // now we validate selection
            if tree.get(self.get_selection()).is_err() {
                let (_, (id, _)) = pot
                    .get(self.selection.idx.saturating_sub(1) as usize)
                    .unwrap_or(pot.last().expect("why is there nothing to render"));

                self.selection.id = id.clone();
            };

            pot.iter()
                .map(|(i, (entry_id, entry))| {
                    let (style, postfix) = if self.selection.id == entry_id.clone() {
                        self.selection.idx = *i as u32;
                        (Style::new().bold().italic(), "*")
                    } else {
                        (Style::new(), "")
                    };

                    let widget = match entry.data().kind {
                        TarsKind::Root(_) => Paragraph::new("SHOULDNTBEPOSSIBLE"),
                        TarsKind::Task(ref t) => {
                            Paragraph::new(format!("{}    {postfix}", *t.name))
                                .style(style.fg(t.group.color.as_ref().into()))
                        }

                        TarsKind::Group(ref g) => {
                            Paragraph::new(format!("{}    {postfix}", *g.name))
                                .style(style.fg(RatColor::Black).bg(g.color.as_ref().into()))
                        }
                    };

                    let layout = Layout::new(
                        Direction::Horizontal,
                        [
                            Constraint::Min(entry.data().depth.saturating_sub(self.rel_depth)),
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

        info!("updated draw info! {:#?}", self.draw_info);
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_is_active(&mut self, is_active: bool) {
        self.active = is_active;
    }

    pub fn get_scope(&self) -> &NodeId {
        &self.scope
    }

    pub async fn set_scope(&mut self, scope: NodeId) {
        self.scope = scope;
        self.calculate_draw_info().await;
    }

    pub fn get_selection(&self) -> &NodeId {
        &self.selection.id
    }
    pub async fn set_selection(&mut self, selection: NodeId) {
        self.selection.id = selection;

        self.calculate_draw_info().await;
    }

    pub async fn set_rel_depth(&mut self, new_rel_depth: u16) {
        self.rel_depth = new_rel_depth;
        self.calculate_draw_info().await;
    }

    pub fn get_draw_info(&self) -> &DrawInfo<'a> {
        self.draw_info.as_ref().unwrap()
    }
}
