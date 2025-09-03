use common::types::Task;
use futures::future::join_all;
use id_tree::NodeId;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::Paragraph,
};
use tui_scrollview::ScrollViewState;

use crate::tree::{TarsKind, TarsTreeHandle};

#[derive(Debug)]
pub struct State<'a> {
    pub active: bool,
    scope: NodeId,
    selection: Selection,
    pub tree_handle: TarsTreeHandle,
    draw_info: Option<DrawInfo<'a>>,
    tasks: Vec<(NodeId, Task)>,
    pub scroll_state: ScrollViewState,
    pub frame_height: u16,
}

#[derive(Debug, Clone)]
pub struct DrawInfo<'a> {
    pub lines: Vec<LineEntry<'a>>,
    pub line_layout: Layout,
}

#[derive(Debug, Clone)]
pub struct LineEntry<'a> {
    pub task: Paragraph<'a>,
    pub group: Paragraph<'a>,
    pub prio_date: Paragraph<'a>,
    pub layout: Layout,
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
            tasks: vec![],
            scroll_state: Default::default(),
            frame_height: 50,
        };

        state.calculate_draw_info().await;

        state
    }

    pub fn get_tasks(&self) -> &Vec<(NodeId, Task)> {
        &self.tasks
    }

    async fn refresh_tasks(&mut self) {
        let tree = self.tree_handle.read().await;

        let pot = tree.traverse(&self.scope);

        let mut tasks_in_scope: Vec<(NodeId, Task)> = pot
            .iter()
            .filter_map(|(id, node)| {
                if let TarsKind::Task(ref t) = node.data().kind
                    && t.finished_at.is_none()
                {
                    return Some((id.clone(), t.clone()));
                }

                None
            })
            .collect();

        // join_all(tasks_in_scope.iter().map(

        // ))

        // sort for least to most
        tasks_in_scope.sort_by(|(_, a), (_, b)| a.cmp(b));

        // reverse so we see highest first
        tasks_in_scope.reverse();

        self.tasks = tasks_in_scope;
    }

    pub async fn calculate_draw_info(&mut self) {
        self.refresh_tasks().await;

        let line_layout = {
            let constraints: Vec<Constraint> = self
                .get_tasks()
                .iter()
                .map(|_| Constraint::Max(1))
                .collect();

            Layout::new(Direction::Vertical, constraints)
        };

        let mut new_sel_idx = self.selection.idx;

        let lines: Vec<LineEntry> = self
            .get_tasks()
            .iter()
            .enumerate()
            .map(|(i, (id, task))| {
                let layout = Layout::new(
                    Direction::Horizontal,
                    [
                        Constraint::Percentage(50),
                        Constraint::Percentage(30),
                        Constraint::Percentage(20),
                    ],
                );

                let text_style = Style::new().fg((&task.group.color).into()).bg({
                    if *id == self.selection.id {
                        new_sel_idx = i;
                        if self.active {
                            Color::Rgb(70, 70, 70)
                        } else {
                            Color::Rgb(35, 35, 35)
                        }
                    } else {
                        Color::Reset
                    }
                });

                let task_line = Paragraph::new((*task.name).to_string()).style(text_style);

                let group_line = Paragraph::new((*task.group.name).to_string()).style(text_style);
                let prio_date = {
                    let text = match task.due {
                        Some(t) => {
                            let text = t.format("%I:%M:%S %p").to_string();

                            if text.as_str() == "11:59:59 PM" {
                                t.format("%m/%d").to_string()
                            } else {
                                t.format("%m/%d %I:%M %p").to_string()
                            }
                        }
                        None => task.priority.into(),
                    };
                    Paragraph::new(text.to_string()).style(text_style)
                };

                LineEntry {
                    layout,
                    task: task_line,
                    group: group_line,
                    prio_date,
                }
            })
            .collect();

        self.selection.idx = new_sel_idx;

        self.draw_info = Some(DrawInfo { lines, line_layout })
    }

    /// Returns a reference to the get scope of this [`State`].
    #[allow(unused)]
    pub fn get_scope(&self) -> &NodeId {
        &self.scope
    }

    pub async fn set_scope(&mut self, scope: NodeId) {
        self.scope = scope;
        self.calculate_draw_info().await;
    }

    pub fn get_selected_id(&self) -> &NodeId {
        &self.selection.id
    }

    pub fn get_selected_idx(&self) -> &usize {
        &self.selection.idx
    }

    pub async fn set_selection(&mut self, selection: NodeId) {
        self.selection.id = selection;
        self.calculate_draw_info().await;
    }
    pub fn get_draw_info(&self) -> &DrawInfo<'a> {
        self.draw_info.as_ref().unwrap()
    }
}
