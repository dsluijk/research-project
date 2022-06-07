use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use tokio::{sync::RwLock, time::sleep};

use crate::{algorithms::Algorithm, edge::Edge, message::Message, node::Node, Topology};

pub struct Graph<T>
where
    T: Algorithm + Send + Sync + 'static,
{
    nodes: Vec<Arc<RwLock<Node<T>>>>,
    unresolved: Arc<AtomicU64>,
}

impl<T> Graph<T>
where
    T: Algorithm + Send + Sync + 'static,
{
    pub async fn new(topology: Arc<Topology>) -> Self {
        let mut next_edge = 0;
        let mut nodes = Vec::with_capacity(topology.get_n());
        let unresolved = Arc::new(AtomicU64::new(0));

        for n in 0..topology.get_n() {
            nodes.push(Node::new(n.to_string(), topology.clone()));
        }

        for (a, b) in topology.get_edges() {
            next_edge = Self::connect_nodes(
                nodes[a].clone(),
                nodes[b].clone(),
                next_edge,
                unresolved.clone(),
            )
            .await;
        }

        for n in topology.get_faulty() {
            nodes[n].write().await.set_faulty();
        }

        Self { nodes, unresolved }
    }

    pub async fn broadcast(&self, node: Arc<RwLock<Node<T>>>, msg: Message) {
        Node::broadcast(node, msg).await
    }

    pub fn get_nodes(&self) -> Vec<Arc<RwLock<Node<T>>>> {
        self.nodes.clone()
    }

    pub async fn wait_settled(&self) {
        while self.unresolved.load(Ordering::Acquire) != 0 {
            sleep(Duration::from_millis(42)).await
        }
    }

    pub async fn print(&self) {
        for node in &self.nodes {
            let node = node.read().await;
            println!("Node #{}.", node.get_label());

            for edge in node.get_edges() {
                let edge = edge.read().await;
                println!("  Edge to {}.", edge.to_label().await);
            }
        }
    }

    pub async fn get_total_messages(&self) -> u64 {
        let mut total = 0;

        for node in &self.nodes {
            total += node.read().await.get_messages().await;
        }

        total
    }

    async fn connect_nodes(
        al: Arc<RwLock<Node<T>>>,
        bl: Arc<RwLock<Node<T>>>,
        next_edge: usize,
        unresolved: Arc<AtomicU64>,
    ) -> usize {
        let mut node = al.write().await;
        let mut new_node = bl.write().await;

        node.add_edge(Edge::new(
            next_edge,
            al.clone(),
            bl.clone(),
            unresolved.clone(),
        ));
        new_node.add_edge(Edge::new(
            next_edge + 1,
            bl.clone(),
            al.clone(),
            unresolved.clone(),
        ));

        next_edge + 2
    }
}
