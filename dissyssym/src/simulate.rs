use std::{
    fs,
    sync::{Arc, Mutex},
};

use dissyssym_lib::{
    algorithms::{Algorithm, FloodingAlgorithm, RoutedAlgorithm},
    Graph, Message, RouteCache, Topology,
};
use rayon::prelude::*;

#[tokio::main]
async fn main() {
    let mut entries = fs::read_dir("./topologies")
        .expect("Failed to read topologies dir.")
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect directory entries.");
    entries.sort();

    let cache = Arc::new(Mutex::new(RouteCache::new()));
    let handle = tokio::runtime::Handle::current();
    let _ = handle.enter();

    entries.par_iter().for_each(|path| {
        if path
            .file_name()
            .unwrap()
            .to_owned()
            .into_string()
            .unwrap()
            .split("-")
            .next()
            .unwrap()
            .len()
            > 1
        {
            return;
        }

        for f in 1.. {
            let top = match handle.block_on(Topology::parse(path.clone(), f)) {
                Some(top) => Arc::new(top),
                None => break,
            };

            if top.get_n() < 3 {
                continue;
            }

            for _ in 0..5 {
                let resf = handle.block_on(run_simulation::<FloodingAlgorithm>(
                    top.clone(),
                    cache.clone(),
                ));

                if resf.is_none() {
                    println!("Failed to run flooding sim.");
                    std::process::exit(1);
                }

                let resr = handle.block_on(run_simulation::<RoutedAlgorithm>(
                    top.clone(),
                    cache.clone(),
                ));

                if resr.is_none() {
                    println!("Failed to run routed sim.");
                    std::process::exit(1);
                }

                let resf = resf.unwrap();
                let resr = resr.unwrap();

                println!(
                    "f: {} | {}, r: {} | {}",
                    resf.delivered, resf.messages, resr.delivered, resr.messages
                );
            }
        }
    });
}

async fn run_simulation<T: Algorithm + Send + Sync + 'static>(
    top: Arc<Topology>,
    cache: Arc<Mutex<RouteCache>>,
) -> Option<SimResult> {
    let mut graph: Graph<T> = Graph::new(top.clone(), cache).await;

    // Send first message.
    let sender1 = graph
        .get_nodes()
        .first()
        .expect("Failed to get the first node.")
        .clone();
    let sender1_id = sender1.read().await.get_label();
    graph
        .broadcast(
            sender1,
            Message::new(sender1_id, "singlemessage".to_string()),
        )
        .await;

    // Wait till finish and collect results.
    graph.wait_settled().await;
    let messages = graph.get_total_messages().await;
    let delivered = graph.get_delivered_broadcasts().await;

    Some(SimResult {
        c: top.get_c(),
        f: top.get_faulty().len(),
        n: top.get_n(),
        messages: messages as usize,
        delivered,
    })
}

struct SimResult {
    delivered: f64,
    messages: usize,
    c: usize,
    f: usize,
    n: usize,
}
