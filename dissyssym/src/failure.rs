use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use dissyssym_lib::{
    algorithms::{Algorithm, RoutedAlgorithm},
    Graph, Message, RouteCache, Topology,
};
use rand::prelude::SliceRandom;
use rayon::prelude::*;

#[tokio::main]
async fn main() {
    let mut entries = fs::read_dir("./topologies")
        .expect("Failed to read topologies dir.")
        .map(|res| res.unwrap().path())
        .filter(|p| {
            let file = p.file_name().unwrap().to_str().unwrap();
            let mut split = file.split("-");
            let n = split.next().unwrap().parse::<usize>().unwrap();
            n <= 20
        })
        .collect::<Vec<PathBuf>>();
    entries.sort();

    let cache_p = Arc::new(Mutex::new(RouteCache::new(String::from("pathfind"))));
    let cache_f = Arc::new(Mutex::new(RouteCache::new(String::from("unreliable"))));
    let handle = tokio::runtime::Handle::current();
    let _ = handle.enter();

    let total = Arc::new(Mutex::new(0));
    let fails_p = Arc::new(Mutex::new(0));
    let fails_f = Arc::new(Mutex::new(0));
    let results = Mutex::new(File::create("./failures.data").unwrap());

    entries.par_iter().for_each(|path| {
        for f in 0.. {
            let top = match handle.block_on(Topology::parse(path.clone(), f)) {
                Some(top) => Arc::new(top),
                None => break,
            };

            let resr_p = handle.block_on(run_simulation::<RoutedAlgorithm>(
                top.clone(),
                cache_p.clone(),
            ));

            let resr_f = handle.block_on(run_simulation::<RoutedAlgorithm>(
                top.clone(),
                cache_f.clone(),
            ));

            *total.lock().unwrap() += 1;

            if !resr_p {
                let result = format!(
                    "n: {}, f: {}, c: {}, a: p\n",
                    top.get_n(),
                    top.get_faulty().len(),
                    top.get_c()
                );

                *fails_p.lock().unwrap() += 1;
                results.lock().unwrap().write(result.as_bytes()).unwrap();
                print!("{}", result);
            }

            if !resr_f {
                let result = format!(
                    "n: {}, f: {}, c: {}, a: f\n",
                    top.get_n(),
                    top.get_faulty().len(),
                    top.get_c()
                );

                *fails_f.lock().unwrap() += 1;
                results.lock().unwrap().write(result.as_bytes()).unwrap();
                print!("{}", result);
            }
        }
    });

    println!(
        "Total: {} | P Fails: {} | F Fails: {}",
        total.lock().unwrap(),
        fails_p.lock().unwrap(),
        fails_f.lock().unwrap(),
    )
}

async fn run_simulation<T: Algorithm + Send + Sync + 'static>(
    top: Arc<Topology>,
    cache: Arc<Mutex<RouteCache>>,
) -> bool {
    let mut graph: Graph<T> = match Graph::new(top.clone(), cache).await {
        Some(g) => g,
        None => return false,
    };

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
    let delivered = graph.get_delivered_broadcasts().await;

    delivered > 99.95
}
