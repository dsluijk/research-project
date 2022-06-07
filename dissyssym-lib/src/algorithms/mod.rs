mod flooding;
mod routed;

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

pub use flooding::FloodingAlgorithm;
pub use routed::RoutedAlgorithm;

use crate::{node::Node, Message, Topology};

#[async_trait]
pub trait Algorithm {
    fn new(node_id: usize, topology: Arc<Topology>) -> Self;

    async fn on_message<T>(
        &mut self,
        current: Arc<RwLock<Node<T>>>,
        sender: Arc<RwLock<Node<T>>>,
        message: Message,
    ) where
        T: Algorithm + 'static + Send + Sync;

    async fn send_broadcast<T>(&mut self, current: Arc<RwLock<Node<T>>>, message: Message)
    where
        T: Algorithm + 'static + Send + Sync;
}
