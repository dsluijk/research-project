use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use tokio::sync::RwLock;

use crate::{algorithms::Algorithm, edge::Edge, message::Message, RouteCache, Topology};

#[derive(Debug)]
pub struct Node<T>
where
    T: Algorithm + Send + Sync + 'static,
{
    label: usize,
    faulty: bool,
    edges: Vec<Arc<RwLock<Edge<T>>>>,
    algo: Arc<RwLock<T>>,
    delivered: Vec<Message>,
}

impl<T: Algorithm + Send + Sync + 'static> Node<T> {
    pub fn new(
        label: usize,
        topology: Arc<Topology>,
        route_cache: Arc<Mutex<RouteCache>>,
    ) -> Option<Arc<RwLock<Self>>> {
        let algo = match T::new(label, topology.clone(), route_cache) {
            Some(a) => a,
            None => return None,
        };

        Some(Arc::new(RwLock::new(Node {
            label,
            faulty: false,
            algo: Arc::new(RwLock::new(algo)),
            edges: Vec::new(),
            delivered: Vec::new(),
        })))
    }

    pub async fn broadcast(node: Arc<RwLock<Node<T>>>, msg: Message) {
        let locked = node.read().await;
        let algo = locked.algo.clone();
        drop(locked);

        let mut algo = algo.write().await;

        algo.send_broadcast(node, msg).await
    }

    pub async fn recv(node: Arc<RwLock<Self>>, sender: Arc<RwLock<Node<T>>>, msg: Message) {
        let node_lock = node.read().await;
        let algo = node_lock.algo.clone();

        if node_lock.faulty {
            return;
        }

        drop(node_lock);

        let mut algo = algo.write().await;
        algo.on_message(node, sender, msg).await;
    }

    pub fn add_edge(&mut self, edge: Edge<T>) {
        self.edges.push(Arc::new(RwLock::new(edge)));
    }

    pub fn get_edges(&self) -> &Vec<Arc<RwLock<Edge<T>>>> {
        &self.edges
    }

    pub async fn get_messages(&self) -> u64 {
        let mut total = 0;

        for edge in &self.edges {
            total += edge.read().await.get_messages();
        }

        total
    }

    pub fn deliver(&mut self, msg: Message) {
        self.delivered.push(msg);
    }

    pub fn get_delivered(&self) -> Vec<Message> {
        self.delivered.clone()
    }

    pub fn get_faulty(&self) -> bool {
        self.faulty
    }

    pub fn set_faulty(&mut self) {
        self.faulty = true;
    }

    pub fn get_label(&self) -> usize {
        self.label
    }
}

impl<T> PartialEq for Node<T>
where
    T: Algorithm + Send + Sync + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        if self.label != other.label {
            return false;
        }

        if self.edges.len() != other.edges.len() {
            return false;
        }

        return true;
    }
}

impl<T> Display for Node<T>
where
    T: Algorithm + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Node `{}` ({} edges)", self.label, self.edges.len())
    }
}
