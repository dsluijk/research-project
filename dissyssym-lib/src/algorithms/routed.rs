use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::Algorithm;
use crate::{node::Node, Message, Topology};

pub struct RoutedAlgorithm {
    received: HashSet<String>,
    topology: Arc<Topology>,
}

#[async_trait]
impl Algorithm for RoutedAlgorithm {
    fn new(_: String, topology: Arc<Topology>) -> Self {
        Self {
            topology,
            received: HashSet::new(),
        }
    }

    async fn on_message<T: 'static>(
        &mut self,
        current: Arc<RwLock<Node<T>>>,
        sender: Arc<RwLock<Node<T>>>,
        message: Message,
    ) where
        T: Algorithm + Send + Sync + 'static,
    {
        let id = message.get_id();
        if self.received.contains(&id) {
            return;
        }

        self.received.insert(id);
        let current = current.read().await;
        let edges = current.get_edges();

        let sender_locked = sender.read().await;
        let sender_label = sender_locked.get_label();
        drop(sender_locked);

        for edge in edges {
            let mut edge = (*edge).write().await;
            if sender_label == edge.to_label().await {
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
        let node = current.read().await;

        for edge in node.get_edges() {
            let mut edge = (*edge).write().await;
            edge.send(message.clone()).await;
        }
    }
}
