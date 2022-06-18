use std::{
    collections::{HashMap, HashSet, VecDeque},
    io::Write,
    path::Path,
};

use rand::{
    prelude::{IteratorRandom, SliceRandom, ThreadRng},
    thread_rng,
};
use tokio::fs::read_to_string;

#[derive(Debug, Clone)]
pub struct Topology {
    n: usize,
    c: usize,
    edges: Vec<(usize, usize)>,
    faulty: Vec<usize>,
}

impl Topology {
    pub fn generate(&mut self, n: usize, c: usize, f: usize) -> bool {
        assert!(c > 0, "Connectivity must be at least one.");
        assert!(n > c, "Connectivity has to be lower than total.");

        if n * c % 2 == 1 {
            return false;
        }

        self.n = n;
        self.c = c;
        let mut rng = thread_rng();
        let mut attempt = 0;

        // Try to create a graph until it's valid.
        self.edges = loop {
            attempt += 1;

            if attempt > 25_000 {
                return false;
            }

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
        true
    }

    pub fn write(&self, path: impl AsRef<Path>) {
        let mut result = String::new();

        for (n1, n2) in &self.edges {
            result.push_str(format!("{} {}\n", n1, n2).as_str());
        }

        std::fs::File::create(path)
            .expect("Failed to create file for topology.")
            .write_all(result.as_bytes())
            .expect("Failed to write content of the topology.")
    }

    pub async fn parse(path: impl AsRef<Path>, f: usize) -> Option<Self> {
        let content = read_to_string(&path)
            .await
            .expect("Failed to read topology file!");
        let mut uniques = HashSet::new();
        let mut edges = Vec::new();

        for line in content.lines() {
            let mut split = line.split(' ');
            let a = split
                .next()
                .expect("Failed to get first edge.")
                .parse::<usize>()
                .expect("Failed to parse first edge as usize.");
            let b = split
                .next()
                .expect("Failed to get second edge.")
                .parse::<usize>()
                .expect("Failed to parse first edge as usize.");

            uniques.insert(a);
            uniques.insert(b);

            if a < b {
                edges.push((a, b));
            } else {
                edges.push((b, a));
            }
        }

        let n = uniques.len();
        let c = Self::connectivity(&edges, n);

        if c <= f {
            return None;
        }

        let mut rng = thread_rng();
        let faulty = (0..n).choose_multiple(&mut rng, f);

        Some(Self {
            n,
            c,
            edges,
            faulty,
        })
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

    pub fn get_c(&self) -> usize {
        self.c
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

    pub fn connectivity(edges: &Vec<(usize, usize)>, n: usize) -> usize {
        let flowgraph = FlowGraph::new(edges);
        let mut min = n;

        for (_, outgoing) in flowgraph.get_nodes() {
            min = min.min(outgoing.len());
        }

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
            c: 0,
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
                nodes.insert(*a, HashSet::from([*b]));
            }

            if let Some(node) = nodes.get_mut(&b) {
                node.insert(*a);
            } else {
                nodes.insert(*b, HashSet::from([*a]));
            }
        }

        FlowGraph { nodes }
    }

    pub fn get_nodes(&self) -> HashMap<usize, HashSet<usize>> {
        self.nodes.clone()
    }

    pub fn max_flow(&self, s: usize, t: usize) -> usize {
        if !self.nodes.contains_key(&s) {
            return 0;
        }

        if !self.nodes.contains_key(&t) {
            return 0;
        }

        if s == t {
            return 0;
        }

        let mut flowing: HashSet<(usize, usize)> = HashSet::new();
        loop {
            let mut q = VecDeque::from([s]);
            let mut pred = HashMap::new();
            let mut coloured = HashSet::new();

            while let Some(n) = q.pop_front() {
                if n == t {
                    break;
                }

                if coloured.contains(&n) {
                    continue;
                }

                coloured.insert(n);
                for &neigh in self.nodes.get(&n).unwrap() {
                    if flowing.contains(&(n, neigh)) {
                        continue;
                    }

                    q.push_back(neigh);

                    if !pred.contains_key(&neigh) {
                        pred.insert(neigh, n);
                    }
                }
            }

            if !pred.contains_key(&t) {
                break;
            }

            let mut path = vec![t];
            let mut c = t;
            while c != s {
                let prev = pred[&c];
                path.push(prev);
                c = prev;
            }

            path.reverse();
            for i in 0..(path.len() - 1) {
                flowing.insert((path[i], path[i + 1]));
                flowing.remove(&(path[i + 1], path[i]));
            }
        }

        let mut flow = 0;
        for (a, b) in flowing {
            if a == s {
                flow += 1;
            }

            if b == s {
                flow -= 1;
            }
        }

        flow
    }
}
