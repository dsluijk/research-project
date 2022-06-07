use std::sync::Arc;

use dissyssym_lib::{
    algorithms::{FloodingAlgorithm, RoutedAlgorithm},
    Graph, Message, Topology,
};

#[tokio::main]
async fn main() {
    let mut topology = Topology::default();
    topology.generate(20, 3, 1);
    topology.write("./topology.txt").await;

    let topology = Arc::new(topology);

    let g1: Graph<FloodingAlgorithm> = Graph::new(topology.clone()).await;
    g1.broadcast(
        g1.get_nodes()
            .first()
            .expect("Failed to get the first node.")
            .clone(),
        Message::new("Hello world".to_string()),
    )
    .await;
    g1.wait_settled().await;
    let g1_messages = g1.get_total_messages().await;
    drop(g1);

    let g2: Graph<RoutedAlgorithm> = Graph::new(topology.clone()).await;
    g2.broadcast(
        g2.get_nodes()
            .first()
            .expect("Failed to get the first node.")
            .clone(),
        Message::new("Hello world".to_string()),
    )
    .await;
    g2.wait_settled().await;
    let g2_messages = g2.get_total_messages().await;
    drop(g2);

    println!("Total messages send (flooding): {}.", g1_messages);
    println!("Total messages send (routed): {}.", g2_messages);
}
