use std::{
    fs::{self, File},
    io::Write,
    sync::{Arc, Mutex},
    time::Duration,
};

use dissyssym_lib::{
    algorithms::{Algorithm, FloodingAlgorithm, RoutedAlgorithm},
    Graph, Message, RouteCache, Topology,
};
use rand::prelude::SliceRandom;
use rayon::prelude::*;
use tokio::time::Instant;

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

    let totalf = Mutex::new(0);
    let totalr = Mutex::new(0);
    let results = Mutex::new(File::create("./results.data").unwrap());

    entries.par_iter().for_each(|path| {
        for f in 0.. {
            let top = match handle.block_on(Topology::parse(path.clone(), f)) {
                Some(top) => Arc::new(top),
                None => break,
            };

            for i in 0..5 {
                let resf = handle.block_on(run_simulation::<FloodingAlgorithm>(
                    top.clone(),
                    cache.clone(),
                ));
                let resr = handle.block_on(run_simulation::<RoutedAlgorithm>(
                    top.clone(),
                    cache.clone(),
                ));

                let resf = match resf {
                    Some(r) => r,
                    None => continue,
                };
                let resr = match resr {
                    Some(r) => r,
                    None => continue,
                };

                let result = format!(
                    "[n: {}, f: {}, c: {}, i: {}] f: d {}%, m {}, t: {} | r: d {}%, m {}, t: {}\n",
                    top.get_n(),
                    top.get_faulty().len(),
                    top.get_c(),
                    i,
                    resf.delivered,
                    resf.messages,
                    resf.duration.as_millis(),
                    resr.delivered,
                    resr.messages,
                    resr.duration.as_millis()
                );

                results.lock().unwrap().write(result.as_bytes()).unwrap();
                print!("{}", result);

                *totalf.lock().unwrap() += resf.messages;
                *totalr.lock().unwrap() += resr.messages;

                if resf.delivered < 99.995 || resr.delivered < 99.995 {
                    panic!("oh no, delivery is bad again :(");
                }
            }
        }
    });

    println!(
        "Total messages: {} vs {}",
        totalf.lock().unwrap(),
        totalr.lock().unwrap()
    );
}

async fn run_simulation<T: Algorithm + Send + Sync + 'static>(
    top: Arc<Topology>,
    cache: Arc<Mutex<RouteCache>>,
) -> Option<SimResult> {
    let mut graph: Graph<T> = match Graph::new(top.clone(), cache).await {
        Some(g) => g,
        None => return None,
    };

    let now = Instant::now();
    // Broadcast a message from a random sender.
    let mut rng = rand::thread_rng();
    let sender = graph
        .get_nodes()
        .choose(&mut rng)
        .expect("Failed to get random node.")
        .clone();
    let sender_id = sender.read().await.get_label();
    graph
        .broadcast(sender, Message::new(sender_id, "msg".to_string()))
        .await;

    // Wait till finish and collect results.
    graph.wait_settled().await;
    let duration = now.elapsed();
    let messages = graph.get_total_messages().await;
    let delivered = graph.get_delivered_broadcasts().await;

    Some(SimResult {
        c: top.get_c(),
        f: top.get_faulty().len(),
        n: top.get_n(),
        messages: messages as usize,
        delivered,
        duration,
    })
}

struct SimResult {
    delivered: f64,
    messages: usize,
    duration: Duration,
    c: usize,
    f: usize,
    n: usize,
}
