use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use rand_distr::{Distribution, Normal};
use tokio::{
    sync::{mpsc, RwLock},
    time::sleep,
};

use crate::{algorithms::Algorithm, message::Message, node::Node};

#[derive(Debug)]
pub struct Edge<T>
where
    T: Algorithm + Send + Sync + 'static,
{
    id: usize,
    total_messages: u64,
    to: Arc<RwLock<Node<T>>>,
    unresolved: Arc<AtomicU64>,
    tx: mpsc::UnboundedSender<Message>,
}

impl<T> Edge<T>
where
    T: Algorithm + Send + Sync + 'static,
{
    pub fn new(
        id: usize,
        from: Arc<RwLock<Node<T>>>,
        to: Arc<RwLock<Node<T>>>,
        unresolved: Arc<AtomicU64>,
    ) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        let mov_to = to.clone();
        let mov_unresolved = unresolved.clone();

        tokio::spawn(async move {
            loop {
                let msg = match rx.recv().await {
                    Some(it) => it,
                    _ => continue,
                };

                let mov_to = mov_to.clone();
                let from = from.clone();
                let mov_unresolved = mov_unresolved.clone();

                tokio::spawn(async move {
                    let normal = Normal::new(75., 25.).expect("failed to create delay sample.");
                    let delay: f64 = normal.sample(&mut rand::thread_rng());
                    sleep(Duration::from_millis(delay.round() as u64)).await;
                    Node::recv(mov_to, from, msg).await;
                    mov_unresolved.fetch_sub(1, Ordering::AcqRel);
                });
            }
        });

        Self {
            id,
            to,
            tx,
            unresolved,
            total_messages: 0,
        }
    }

    pub async fn send(&mut self, msg: Message) {
        self.unresolved.fetch_add(1, Ordering::AcqRel);
        self.tx
            .send(msg)
            .expect("Failed to send message in the channel.");
        self.total_messages += 1;
    }

    pub async fn to_label(&self) -> String {
        let node = self.to.read().await;
        node.get_label()
    }

    pub fn get_messages(&self) -> u64 {
        self.total_messages
    }
}

impl<T> PartialEq for Edge<T>
where
    T: Algorithm + Send + Sync + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
