use std::path::Path;

use dissyssym_lib::Topology;

#[tokio::main]
async fn main() {
    for n in 2..18 {
        let n = n * n;
        let mut c = n - 1;

        while c > 1 {
            for t in 0..5 {
                let file = format!("./topologies/{}-{}-{}.tpgy", n, c, t);

                if Path::new(file.as_str()).exists() {
                    println!("Topology {}-{}-{} already exists, skipping..", n, c, t);
                    continue;
                }

                println!(
                    "Generating topology {}-{}-{}, this can take a while..",
                    n, c, t
                );
                let mut topology = Topology::default();
                topology.generate(n, c, 0);
                topology.write(file).await;
            }

            c -= (n / 20).max(1);
        }
    }
}
