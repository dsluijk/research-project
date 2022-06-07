use std::sync::Arc;

use dissyssym_lib::{
    algorithms::{FloodingAlgorithm, RoutedAlgorithm},
    Graph, Message, Topology,
};

#[tokio::main]
async fn main() {
    let mut topology = Topology::default();
    topology.generate(25, 10, 2);
    topology.write("./topology.txt").await;

    let topology = Arc::new(topology);

    let mut g1: Graph<FloodingAlgorithm> = Graph::new(topology.clone()).await;
    let sender1 = g1
        .get_nodes()
        .first()
        .expect("Failed to get the first node.")
        .clone();
    let sender1_id = sender1.read().await.get_label();
    g1.broadcast(sender1, Message::new(sender1_id, "Hello world".to_string()))
        .await;
    g1.wait_settled().await;
    let g1_messages = g1.get_total_messages().await;
    let g1_delivered = g1.get_delivered_broadcasts().await;
    drop(g1);

    let mut g2: Graph<RoutedAlgorithm> = Graph::new(topology.clone()).await;
    let sender2 = g2
        .get_nodes()
        .first()
        .expect("Failed to get the first node.")
        .clone();
    let sender2_id = sender2.read().await.get_label();
    g2.broadcast(sender2, Message::new(sender2_id, "Hello world".to_string()))
        .await;
    g2.wait_settled().await;
    let g2_messages = g2.get_total_messages().await;
    let g2_delivered = g2.get_delivered_broadcasts().await;
    drop(g2);

    println!("Total messages send (flooding): {}.", g1_messages);
    println!("Total messages send (routed): {}.", g2_messages);
    println!("Delivery (flooding): {}%.", g1_delivered);
    println!("Delivery (routed): {}%.", g2_delivered);
}
