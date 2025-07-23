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
use futures::StreamExt as _;
use id_tree::{InsertBehavior, Node, NodeId, Tree, TreeBuilder};
use reqwest_eventsource::{Event, EventSource};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TarsNode {
    pub kind: TarsKind,
    // might need it when creating a new task but even then i think that
    // should just cause a node refresh so this should ultimately be removed
    pub parent: Option<NodeId>,

    pub depth: u16,
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
            Node::new(TarsNode {
                kind: TarsKind::Group(group.clone()),
                parent: Some(parent_id.clone()),
                depth: *depth,
            }),
            InsertBehavior::UnderNode(&parent_id),
        )?;

        inverted_map.insert(group.id.clone(), group_id.clone());

        // now we want to add all tasks to it?
        if let Some(tasks) = g_to_t.get(&group.id) {
            *depth += 1;
            for task in tasks {
                let node_id = self.insert(
                    Node::new(TarsNode {
                        kind: TarsKind::Task(task.clone()),
                        parent: Some(group_id.clone()),
                        depth: *depth,
                    }),
                    InsertBehavior::UnderNode(&group_id),
                )?;
                inverted_map.insert(task.id.clone(), node_id.clone());
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

    // syncs the tree to the daemon
    pub async fn sync(&mut self, client: &TarsClient) -> Result<()> {
        // let mut stream = client
        //     .conn
        //     .get(client.base_path.join("/subscribe")?)
        //     .send()
        //     .await?
        //     .byte_stream;

        // a recursive sync that takes a group, fetches the tasks of that remotely, and then verifies that way
        //

        // let groups = Group::fetch_all(client).await?;

        // for (node_id, node) in self.traverse_root() {
        //     // now what
        //     //
        //     if let TarsKind::Group(ref g) = node.data().kind {
        //         // tasks for this group
        //         let group_tasks = Task::fetch(
        //             client,
        //             TaskFetchOptions::ByGroup {
        //                 group_id: g.id.clone(),
        //                 recursive: false,
        //             },
        //         )
        //         .await?;

        //         // now we want to ensure that the node in our tree has all these tasks

        //         // self.get()
        //     }
        // }

        // let groups = Group::fetch_all(client).await?;

        // // how do i sync the tree with the thing
        // let x = self.traverse_root();

        Ok(())
    }
}
