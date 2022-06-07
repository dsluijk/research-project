use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use pathfinding::prelude::bfs;
use rand::{
    prelude::{IteratorRandom, SliceRandom, ThreadRng},
    thread_rng,
};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Debug)]
pub struct Topology {
    n: usize,
    edges: Vec<(usize, usize)>,
    faulty: Vec<usize>,
}

impl Topology {
    pub fn generate(&mut self, n: usize, c: usize, f: usize) {
        assert!(c > 0, "Connectivity must be at least one.");
        assert!(n > c, "Connectivity has to be lower than total.");

        let c = match n * c % 2 {
            0 => c,
            1 => c + 1,
            _ => unreachable!(),
        };

        self.n = n;
        let mut rng = thread_rng();

        // Try to create a graph until it's valid.
        self.edges = loop {
            let edges = match Self::try_generate(&mut rng, n, c) {
                Some(e) => e,
                None => continue,
            };

            if Self::connectivity(&edges, n) != c {
                continue;
            }

            break edges;
        };

        // Set some random nodes as faulty.
        self.faulty = (0..n).choose_multiple(&mut rng, f.try_into().unwrap());
    }

    pub async fn write(&self, path: impl AsRef<Path>) {
        let mut file = File::create(path)
            .await
            .expect("Failed to create file for topology.");
        let mut result = String::new();

        for (n1, n2) in &self.edges {
            result.push_str(format!("{} {}\n", n1, n2).as_str());
        }

        file.write_all(result.as_bytes())
            .await
            .expect("Failed to write content of the topology.");
    }

    pub fn get_edges(&self) -> Vec<(usize, usize)> {
        self.edges.clone()
    }

    pub fn get_n(&self) -> usize {
        self.n
    }

    pub fn get_faulty(&self) -> Vec<usize> {
        self.faulty.clone()
    }

    fn try_generate(rng: &mut ThreadRng, n: usize, d: usize) -> Option<Vec<(usize, usize)>> {
        let mut edges = Vec::new();
        let mut stubs: Vec<usize> = (0..d)
            .map(|_| (0..n).collect::<Vec<usize>>())
            .flatten()
            .collect::<_>();

        while !stubs.is_empty() {
            let mut possible_edges = HashMap::new();
            stubs.shuffle(rng);

            for i in 0..(stubs.len() / 2) {
                let mut s1 = *stubs.get(i * 2).unwrap();
                let mut s2 = *stubs.get((i * 2) + 1).unwrap();

                if s1 > s2 {
                    let tmp = s1;
                    s1 = s2;
                    s2 = tmp;
                }

                if s1 != s2 && !edges.contains(&(s1, s2)) {
                    edges.push((s1, s2));
                } else {
                    possible_edges.insert(s1, possible_edges.get(&s1).unwrap_or(&0) + 1);
                    possible_edges.insert(s2, possible_edges.get(&s2).unwrap_or(&0) + 1);
                }
            }

            if !Self::suitable_graph(&edges, &possible_edges) {
                return None;
            }

            stubs = Vec::new();
            for (node, potential) in &possible_edges {
                for _ in 0..*potential {
                    stubs.push(*node);
                }
            }
        }

        Some(edges)
    }

    fn suitable_graph(edges: &Vec<(usize, usize)>, possible_edges: &HashMap<usize, usize>) -> bool {
        if possible_edges.is_empty() {
            return true;
        }

        for (s1, _) in possible_edges {
            for (s2, _) in possible_edges {
                if s1 == s2 {
                    continue;
                }

                let n1 = match s1 < s2 {
                    true => *s1,
                    false => *s2,
                };

                let n2 = match s1 < s2 {
                    true => *s2,
                    false => *s1,
                };

                if !edges.contains(&(n1, n2)) {
                    return true;
                }
            }
        }

        false
    }

    fn connectivity(edges: &Vec<(usize, usize)>, n: usize) -> usize {
        let mut min = n;

        let flowgraph = FlowGraph::new(edges);

        for i in 0..n {
            for j in (i + 1)..n {
                let max = flowgraph.max_flow(i, j);
                min = min.min(max);
            }
        }

        min
    }
}

impl Default for Topology {
    fn default() -> Self {
        Self {
            n: 0,
            edges: Vec::new(),
            faulty: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlowGraph {
    nodes: HashMap<usize, HashSet<usize>>,
}

impl FlowGraph {
    pub fn new(edges: &Vec<(usize, usize)>) -> Self {
        let mut nodes: HashMap<usize, HashSet<usize>> = HashMap::with_capacity(edges.len());

        for (a, b) in edges {
            if let Some(node) = nodes.get_mut(&a) {
                node.insert(*b);
            } else {
                let mut new_node = HashSet::new();
                new_node.insert(*b);

                nodes.insert(*a, new_node);
            }

            if let Some(node) = nodes.get_mut(&b) {
                node.insert(*a);
            } else {
                let mut new_node = HashSet::new();
                new_node.insert(*a);

                nodes.insert(*b, new_node);
            }
        }

        FlowGraph { nodes }
    }

    pub fn get_nodes(&self) -> HashMap<usize, HashSet<usize>> {
        self.nodes.clone()
    }

    fn max_flow(&self, s: usize, t: usize) -> usize {
        if s == t {
            return 0;
        }

        let mut used = HashSet::new();
        let mut edges: HashSet<(usize, usize)> = HashSet::new();

        while let Some(path) = bfs(
            &s,
            |&i| {
                self.nodes
                    .get(&i)
                    .unwrap()
                    .into_iter()
                    .copied()
                    .filter(|&n| {
                        !edges.contains(&(i, n)) && ((!used.contains(&n)) || n == s || n == t)
                    })
                    .collect::<Vec<usize>>()
            },
            |&n| n == t,
        ) {
            for i in 0..path.len() {
                used.insert(path[i]);

                if i > 0 {
                    let prev = path[i - 1];

                    if !edges.remove(&(path[i], prev)) {
                        edges.insert((prev, path[i]));
                    }
                }

                if i < path.len() - 1 {
                    let next = path[i + 1];

                    if !edges.remove(&(next, path[i])) {
                        edges.insert((path[i], next));
                    }
                }
            }
        }

        let mut flow = 0;
        for n in self.nodes.get(&t).unwrap() {
            if edges.contains(&(*n, t)) {
                flow += 1;
            }
        }

        flow
    }
}
