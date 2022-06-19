use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    sync::Mutex,
};

use dissyssym_lib::{FlowGraph, RouteCache, Topology};
use rayon::prelude::*;
use tokio::time::Instant;

#[tokio::main]
async fn main() {
    let mut entries = fs::read_dir("./topologies")
        .expect("Failed to read topologies dir.")
        .map(|res| res.unwrap().path())
        .filter(|p| {
            let file = p.file_name().unwrap().to_str().unwrap();
            let mut split = file.split("-");
            let n = split.next().unwrap().parse::<usize>().unwrap();
            n <= 30
        })
        .collect::<Vec<PathBuf>>();
    entries.sort();

    let handle = tokio::runtime::Handle::current();
    let _ = handle.enter();

    let results = Mutex::new(File::create("./pathtime.data").unwrap());

    entries.par_iter().for_each(|path| {
        for f in 0.. {
            let top = match handle.block_on(Topology::parse(path.clone(), f)) {
                Some(top) => top,
                None => break,
            };
            let nodes = FlowGraph::new(&top.get_edges()).get_nodes();

            let cache_path = RouteCache::new(String::from("pathfind"));
            let start_path = Instant::now();
            for (&i, _) in &nodes {
                cache_path.gen_routes_uncached(&nodes, f, i);
            }
            let timings_path = start_path.elapsed().as_millis();

            let cache_fast = RouteCache::new(String::from("unreliable"));
            let start_fast = Instant::now();
            for (&i, _) in &nodes {
                cache_fast.gen_routes_uncached(&nodes, f, i);
            }
            let timings_fast = start_fast.elapsed().as_millis();

            let result = format!(
                "[n: {}, f: {}, c: {}] p {} | f {}\n",
                top.get_n(),
                f,
                top.get_c(),
                timings_path,
                timings_fast
            );

            results.lock().unwrap().write(result.as_bytes()).unwrap();
            print!("{}", result);
        }
    });
}
