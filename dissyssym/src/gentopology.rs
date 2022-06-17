use std::path::Path;

use dissyssym_lib::Topology;
use rayon::prelude::*;

fn main() {
    (2..100)
        .into_par_iter()
        .for_each(|n| (1..n).into_par_iter().for_each(|c| gen_n_c(n, c)));
}

fn gen_n_c(n: usize, c: usize) {
    if n * c % 2 == 1 {
        return;
    }

    for i in 0..5 {
        let file = format!("./topologies/{}-{}-{}.tpgy", n, c, i);

        if Path::new(file.as_str()).exists() {
            println!("Topology {}-{}-{} already exists, skipping..", n, c, i);
            continue;
        }

        println!(
            "Generating topology {}-{}-{}, this can take a while..",
            n, c, i
        );

        let mut topology = Topology::default();
        if !topology.generate(n, c, 0) {
            println!("Failed to generate topology, going to the next connectivity.");
            break;
        }

        topology.write(file);
    }
}
