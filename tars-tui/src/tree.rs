use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use color_eyre::Result;
use common::{
    TarsClient,
    types::{Group, Id, Task, TaskFetchOptions},
};
use derive_deref::Deref;
use id_tree::{InsertBehavior, Node, NodeId, Tree, TreeBuilder};
use tokio::sync::RwLock;
use tracing::info;

pub type TarsTreeHandle<'a> = Arc<RwLock<TarsTree<'a>>>;

#[derive(Debug)]
pub struct TarsTree<'a> {
    inner_tree: Tree<TarsNode>,
    pot: Option<Vec<(NodeId, &'a Node<TarsNode>)>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TarsKind {
    Root,
    Task(Task),
    Group(Group),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TarsNode {
    pub kind: TarsKind,
    // might need it when creating a new task but even then i think that
    // should just cause a node refresh so this should ultimately be removed
    pub parent: Option<NodeId>,

    pub depth: u16,
}

// impl Deref for TarsTree<'_> {
//     type Target = Tree<TarsNode>;

//     fn deref(&self) -> &Self::Target {
//         &self.inner_tree
//     }
// }
// impl DerefMut for TarsTree<'_> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner_tree
//     }
// }

impl<'a> TarsTree<'a> {
    pub fn tree(&self) -> &Tree<TarsNode> {
        &self.inner_tree
    }

    pub fn tree_mut(&mut self) -> &mut Tree<TarsNode> {
        &mut self.inner_tree
    }

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

        let mut tree = TarsTree {
            inner_tree: TreeBuilder::new().build(),
            pot: None,
        };

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

            tree.attach_children_of_group(
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

    fn attach_children_of_group(
        &mut self,
        group: Group,
        g_to_g: &HashMap<Id, Vec<Group>>,
        g_to_t: &HashMap<Id, Vec<Task>>,
        depth: &mut u16,
        parent_id: NodeId,
    ) -> Result<()> {
        // insert group into the parent group
        let group_id = self.insert(
            Node::new(TarsNode {
                kind: TarsKind::Group(group.clone()),
                parent: Some(parent_id.clone()),
                depth: *depth,
            }),
            InsertBehavior::UnderNode(&parent_id),
        )?;

        // now we want to add all tasks to it?
        if let Some(tasks) = g_to_t.get(&group.id) {
            *depth += 1;
            for task in tasks {
                let _ = self.insert(
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
            // *depth += 1;
            let mut depth = *depth + 1;
            for child_group in child_groups {
                self.attach_children_of_group(
                    child_group.clone(),
                    g_to_g,
                    g_to_t,
                    &mut depth,
                    group_id.clone(),
                )?;
            }
        }
        Ok(())
    }

    /// a post order traversal of the TarsTree
    pub fn traverse(&'a mut self) -> Vec<(NodeId, &'a Node<TarsNode>)> {
        if self.pot.is_some() {
            return self.pot.clone().unwrap();
        }

        let root = self.tree().root_node_id().unwrap();

        let pot: Vec<(NodeId, &'a Node<TarsNode>)> = self
            .tree()
            .traverse_pre_order_ids(root)
            .unwrap()
            .into_iter()
            .zip(self.tree().traverse_pre_order(root).unwrap())
            .collect();

        self.pot = Some(pot.clone());
        pot
    }
}
