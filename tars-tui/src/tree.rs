use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use color_eyre::Result;
use common::{
    Diff, DiffInner, TarsClient,
    types::{Group, Id, Task, TaskFetchOptions},
};
use id_tree::{InsertBehavior, MoveBehavior, Node, NodeId, RemoveBehavior, Tree, TreeBuilder};
use tokio::sync::RwLock;
use tracing::{error, info};

#[derive(Debug)]
pub struct TarsTree(Tree<TarsNode>);

pub type TarsTreeHandle = Arc<RwLock<TarsTree>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TarsKind {
    Root(HashMap<Id, NodeId>),
    Task(Task),
    Group(Group),
}

impl TarsKind {
    pub fn id(&self) -> Option<Id> {
        match self {
            TarsKind::Root(_) => None,
            TarsKind::Task(t) => Some(t.id.clone()),
            TarsKind::Group(g) => Some(g.id.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TarsNode {
    pub kind: TarsKind,
    pub parent: Option<NodeId>,

    pub depth: u16,
}

impl TarsNode {
    fn new(kind: TarsKind, parent: Option<NodeId>, depth: u16) -> Self {
        Self {
            kind,
            parent,
            depth,
        }
    }
}

impl Deref for TarsTree {
    type Target = Tree<TarsNode>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for TarsTree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TarsTree {
    pub async fn generate(client: &TarsClient) -> Result<Self> {
        // create a hashmap from a group id to its children groups
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

        // a hashmap between group id's and their children tasks
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

        let mut tree = TarsTree(TreeBuilder::new().build());

        let root_id: NodeId = tree.insert(
            Node::new(TarsNode {
                kind: TarsKind::Root(HashMap::new()),
                parent: None,
                depth: 0,
            }),
            InsertBehavior::AsRoot,
        )?;

        let mut inverted_map = HashMap::new();

        let all_groups = Group::fetch_all(client).await?;
        let root_groups: Vec<&Group> = all_groups
            .iter()
            .filter(|e| e.parent_id.is_none())
            .collect();

        for group in root_groups {
            let mut depth = 0;

            tree.attach_children_of_group(
                group.clone(),
                &g_to_g,
                &g_to_t,
                &mut depth,
                root_id.clone(),
                &mut inverted_map,
            )?;
        }

        let root = tree.get_mut(&root_id)?;

        // replace the map with the one we created
        if let TarsKind::Root(map) = &mut root.data_mut().kind {
            *map = inverted_map
        }

        info!("{tree:#?}");
        Ok(tree)
    }

    fn attach_children_of_group(
        &mut self,
        group: Group,
        g_to_g: &HashMap<Id, Vec<Group>>,
        g_to_t: &HashMap<Id, Vec<Task>>,
        depth: &mut u16,
        parent_id: NodeId,
        inverted_map: &mut HashMap<Id, NodeId>,
    ) -> Result<()> {
        // insert group into the parent group
        let group_id = self.insert(
            Node::new(TarsNode::new(
                TarsKind::Group(group.clone()),
                Some(parent_id.clone()),
                *depth,
            )),
            InsertBehavior::UnderNode(&parent_id),
        )?;

        inverted_map.insert(group.id.clone(), group_id.clone());

        *depth += 1;
        // now we want to add all tasks to it
        if let Some(tasks) = g_to_t.get(&group.id) {
            for task in tasks {
                let node_id = self.insert(
                    Node::new(TarsNode::new(
                        TarsKind::Task(task.clone()),
                        Some(group_id.clone()),
                        *depth,
                    )),
                    InsertBehavior::UnderNode(&group_id),
                )?;
                inverted_map.insert(task.id.clone(), node_id.clone());
            }
        }

        if let Some(child_groups) = g_to_g.get(&group.id) {
            for child_group in child_groups {
                let mut depth = *depth;
                self.attach_children_of_group(
                    child_group.clone(),
                    g_to_g,
                    g_to_t,
                    &mut depth,
                    group_id.clone(),
                    inverted_map,
                )?;
            }
        }
        Ok(())
    }

    /// a post order traversal of the TarsTree, starting at the root
    pub fn traverse_root(&self) -> Vec<(NodeId, &Node<TarsNode>)> {
        let node = self.root_node_id().unwrap();
        self.traverse(node)
    }
    /// a post order traversal of the TarsTree, starting at the specified node
    pub fn traverse(&self, node: &NodeId) -> Vec<(NodeId, &Node<TarsNode>)> {
        let pot: Vec<(NodeId, &Node<TarsNode>)> = self
            .traverse_pre_order_ids(node)
            .unwrap()
            .zip(self.traverse_pre_order(node).unwrap())
            .collect();

        pot
    }

    pub fn apply_diff(&mut self, diff: Diff) -> Result<()> {
        match diff {
            Diff::Added(DiffInner::Task(t)) => {
                let group_id = &t.group.id;
                let group_node_id = self
                    .inverted_map_mut()
                    .get(group_id)
                    .expect("group_id should exist")
                    .clone();

                let group_depth = self
                    .get(&group_node_id)
                    .expect("node should exist")
                    .data()
                    .depth;

                let inserted = self.insert(
                    Node::new(TarsNode::new(
                        TarsKind::Task(t.clone()),
                        Some(group_node_id.clone()),
                        group_depth + 1,
                    )),
                    InsertBehavior::UnderNode(&group_node_id),
                )?;

                self.inverted_map_mut().insert(t.id, inserted);
            }

            Diff::Added(DiffInner::Group(g)) => {
                let parent_group_id = g.parent_id.clone();

                let inserted = if parent_group_id.is_none() {
                    // its a new root group
                    let root = self.root_node_id().cloned().unwrap();
                    self.insert(
                        Node::new(TarsNode::new(TarsKind::Group(g.clone()), None, 0)),
                        InsertBehavior::UnderNode(&root),
                    )?
                } else {
                    let parent_node_id = self
                        .inverted_map_mut()
                        .get(&parent_group_id.unwrap())
                        .expect("group should exist")
                        .clone();

                    let parent_depth = self.get(&parent_node_id)?.data().depth;

                    self.insert(
                        Node::new(TarsNode::new(
                            TarsKind::Group(g.clone()),
                            Some(parent_node_id.clone()),
                            parent_depth + 1,
                        )),
                        InsertBehavior::UnderNode(&parent_node_id),
                    )?
                };
                self.inverted_map_mut().insert(g.id, inserted);
            }
            Diff::Updated(DiffInner::Task(t)) => {
                let node_id = self
                    .inverted_map_mut()
                    .get(&t.id)
                    .expect("node should exist")
                    .clone();

                // just gonna drop the old node
                self.remove_node(node_id, RemoveBehavior::DropChildren)?;

                let parent_node_id = self
                    .inverted_map_mut()
                    .get(&t.group.id)
                    .expect("parent group should exist")
                    .clone();

                let parent_depth = self.get(&parent_node_id)?.data().depth;

                self.insert(
                    Node::new(TarsNode::new(
                        TarsKind::Task(t),
                        Some(parent_node_id.clone()),
                        parent_depth + 1,
                    )),
                    InsertBehavior::UnderNode(&parent_node_id),
                )?;
            }
            Diff::Updated(DiffInner::Group(g)) => {
                let curr_node_id = self
                    .inverted_map_mut()
                    .get(&g.id)
                    .expect("node should exist")
                    .clone();
                let curr_node = self.get(&curr_node_id)?;

                // we want to make sure that they are still parented to the same node
                let curr_parent_id = curr_node
                    .parent()
                    .and_then(|e| self.get(e).expect("should be valid").data().kind.id());

                // the node has been moved
                if g.parent_id != curr_parent_id {
                    let new_parent_node_id = match g.parent_id {
                        Some(ref id) => self
                            .inverted_map_mut()
                            .get(id)
                            .expect("should exist")
                            .clone(),
                        None => self
                            .root_node_id()
                            .expect("root node id should exist")
                            .clone(),
                    };

                    self.move_node(&curr_node_id, MoveBehavior::ToParent(&new_parent_node_id))?
                } else {
                    // only the node data has changed
                    let curr_node = self.get_mut(&curr_node_id)?;
                    let depth = curr_node.data().depth;
                    let parent_id = curr_node.parent().expect("Parent should exist");

                    curr_node.replace_data(TarsNode::new(
                        TarsKind::Group(g.clone()),
                        Some(parent_id.clone()),
                        depth,
                    ));
                }

                // now we have to update the direct tasks of this child
                let curr_node = self.get(&curr_node_id)?;
                let children = curr_node.children().clone();
                for child in children {
                    if let TarsKind::Task(t) = &mut self.get_mut(&child)?.data_mut().kind {
                        t.group = g.clone()
                    }
                }
            }

            Diff::Deleted(id) => {
                let node_id = self
                    .inverted_map_mut()
                    .get(&id)
                    .expect("should exist")
                    .clone();
                self.recur_delete(id)?;
                let _ = self.remove_node(node_id, RemoveBehavior::DropChildren)?;
            }
        };
        Ok(())
    }

    fn recur_delete(&mut self, id: Id) -> Result<()> {
        let node_id = self
            .inverted_map_mut()
            .get(&id)
            .expect("should exist")
            .clone();
        let children = self.get(&node_id)?.children().clone();

        // first remove all the children from the map
        for child in children {
            let node = self
                .get(&child)
                .expect("should exist")
                .data()
                .kind
                .id()
                .expect("node should eist");

            let _ = self.inverted_map_mut().remove(&node);

            self.recur_delete(node)?;
        }

        let _ = self.inverted_map_mut().remove(&id);

        Ok(())
    }

    fn inverted_map_mut(&mut self) -> &mut HashMap<Id, NodeId> {
        let root = self.root_node_id().unwrap().clone();
        let node = self.get_mut(&root).unwrap().data_mut();

        if let TarsKind::Root(map) = &mut node.kind {
            map
        } else {
            error!("Tree in impossible state");
            panic!()
        }
    }

    fn inverted_map(&self) -> &HashMap<Id, NodeId> {
        let root = self.root_node_id().unwrap().clone();
        let node = self.get(&root).unwrap().data();

        if let TarsKind::Root(map) = &node.kind {
            map
        } else {
            error!("Tree in impossible state");
            panic!()
        }
    }

    pub fn get_by_tars_id(&self, id: Id) -> Option<&Node<TarsNode>> {
        let node_id = self.inverted_map().get(&id)?;

        let node = self.get(node_id).ok()?;
        Some(node)
    }

    pub fn translate_id_to_node_id(&self, id: &Id) -> Option<NodeId> {
        self.inverted_map().get(id).cloned()
    }
    pub fn translate_node_id_to_id(&self, node_id: &NodeId) -> Option<Id> {
        let x = self.get(node_id).ok()?;

        match x.data().kind {
            TarsKind::Root(_) => None,
            TarsKind::Group(ref g) => Some(g.id.clone()),
            TarsKind::Task(ref t) => Some(t.id.clone()),
        }
    }

    // syncs the tree to the daemon
}
