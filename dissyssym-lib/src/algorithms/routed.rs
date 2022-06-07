use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use async_trait::async_trait;
use sha2::{Digest, Sha512};
use tokio::sync::RwLock;

use super::Algorithm;
use crate::{node::Node, topology::FlowGraph, Message, Topology};

pub struct RoutedAlgorithm {
    received: HashSet<String>,
    routes: HashMap<usize, HashSet<usize>>,
}

#[async_trait]
impl Algorithm for RoutedAlgorithm {
    fn new(n: usize, topology: Arc<Topology>) -> Self {
        let mut routes = HashMap::new();
        let nodes = FlowGraph::new(&topology.get_edges()).get_nodes();
        let f = topology.get_faulty().len();

        for (s, _) in &nodes {
            routes.insert(*s, Self::build_routes(&nodes, f, *s, n));
        }

        Self {
            routes,
            received: HashSet::new(),
        }
    }

    async fn on_message<T>(
        &mut self,
        current: Arc<RwLock<Node<T>>>,
        _sender: Arc<RwLock<Node<T>>>,
        message: Message,
    ) where
        T: Algorithm + Send + Sync + 'static,
    {
        let id = message.get_id();
        if self.received.contains(&id) {
            return;
        }

        self.received.insert(id);
        current.write().await.deliver(message.clone());
        let current = current.read().await;
        let edges = current.get_edges();

        for edge in edges {
            let mut edge = edge.write().await;
            let label = edge.to_label().await;

            if !self
                .routes
                .get(&message.get_sender())
                .unwrap()
                .contains(&label)
            {
                continue;
            }

            edge.send(message.clone()).await;
        }
    }

    async fn send_broadcast<T: 'static>(&mut self, current: Arc<RwLock<Node<T>>>, message: Message)
    where
        T: Algorithm + Send + Sync + 'static,
    {
        self.received.insert(message.get_id());
        current.write().await.deliver(message.clone());
        let node = current.read().await;

        for edge in node.get_edges() {
            let mut edge = (*edge).write().await;
            edge.send(message.clone()).await;
        }
    }
}

impl RoutedAlgorithm {
    fn build_routes(
        nodes: &HashMap<usize, HashSet<usize>>,
        f: usize,
        s: usize,
        n: usize,
    ) -> HashSet<usize> {
        let mut q = VecDeque::new();
        let mut node_paths: HashMap<usize, Vec<Vec<usize>>> = HashMap::new();

        for (n, _) in nodes {
            node_paths.insert(*n, Vec::new());
        }

        node_paths.insert(s, vec![vec![s]]);
        q.push_back(vec![s]);

        while !q.is_empty() {
            let mut possible_paths: HashMap<usize, Vec<Vec<usize>>> = HashMap::new();
            let changed = q.clone();
            q.clear();

            for path in changed {
                let neigh = nodes.get(path.last().unwrap()).unwrap();

                for other in neigh {
                    let mut valid = true;
                    let other_paths = node_paths.get(other).unwrap();

                    for op in other_paths {
                        for path_item in &path {
                            if op.contains(&path_item) && *path_item != s {
                                valid = false;
                                break;
                            }
                        }

                        if !valid {
                            break;
                        }
                    }

                    if valid {
                        let mut new_path = path.clone();
                        new_path.push(*other);

                        if let Some(pl) = possible_paths.get_mut(other) {
                            pl.push(new_path);
                        } else {
                            possible_paths.insert(*other, vec![new_path]);
                        }
                    }
                }
            }

            for (node, paths) in possible_paths {
                let mut ordered_hashed: Vec<(String, Vec<usize>)> = Vec::new();

                for path in paths {
                    let mut hasher = Sha512::new();

                    for entry in &path {
                        hasher.update(entry.to_string());
                    }

                    let hash = format!("{:X}", hasher.finalize());
                    ordered_hashed.push((hash, path.clone()));
                }

                ordered_hashed.sort_by(|(a, _), (b, _)| a.cmp(b));
                let existing = node_paths.get_mut(&node).unwrap();

                for _ in 0..(f + 1 - existing.len()) {
                    if let Some((_, p)) = ordered_hashed.pop() {
                        existing.push(p.to_vec());
                        q.push_back(p.to_vec());
                    } else {
                        break;
                    }
                }
            }
        }

        let mut routes = HashSet::new();
        for neigh in nodes.get(&n).unwrap() {
            for path in node_paths.get(neigh).unwrap() {
                for i in 0..(path.len() - 1) {
                    if path[i] != n {
                        continue;
                    }

                    if path[i + 1] == s {
                        continue;
                    }

                    routes.insert(path[i + 1]);
                }
            }
        }

        routes
    }
}
