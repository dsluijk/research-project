use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::Algorithm;
use crate::{node::Node, topology::FlowGraph, Message, RouteCache, Topology};

pub struct RoutedAlgorithm {
    received: HashSet<String>,
    routes: HashMap<usize, HashSet<usize>>,
}

#[async_trait]
impl Algorithm for RoutedAlgorithm {
    fn new(n: usize, topology: Arc<Topology>, route_cache: Arc<Mutex<RouteCache>>) -> Self {
        let mut routes = HashMap::new();
        let nodes = FlowGraph::new(&topology.get_edges()).get_nodes();
        let f = topology.get_faulty().len();

        for (s, _) in &nodes {
            routes.insert(
                *s,
                Self::build_routes(route_cache.clone(), &nodes, f, *s, n),
            );
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
        cache: Arc<Mutex<RouteCache>>,
        nodes: &HashMap<usize, HashSet<usize>>,
        f: usize,
        s: usize,
        n: usize,
    ) -> HashSet<usize> {
        let mut lock = cache.lock().unwrap();
        match lock.gen_routes(nodes, f, s).get(&n) {
            Some(routes) => routes.clone(),
            None => panic!("Failed to generate routes."),
        }
    }
}
