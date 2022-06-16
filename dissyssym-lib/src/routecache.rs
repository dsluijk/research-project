use std::collections::{HashMap, HashSet, VecDeque};

use sha2::{Digest, Sha512};

pub struct RouteCache {
    cache: HashMap<String, HashMap<usize, Vec<Vec<usize>>>>,
}

impl RouteCache {
    pub fn new() -> Self {
        RouteCache {
            cache: HashMap::new(),
        }
    }

    pub fn gen_routes(
        &mut self,
        nodes: &HashMap<usize, HashSet<usize>>,
        f: usize,
        s: usize,
    ) -> HashMap<usize, Vec<Vec<usize>>> {
        let hash = Self::hash_params(nodes, f, s);

        match self.cache.get(&hash) {
            Some(routes) => routes.clone(),
            None => {
                let routes = Self::get_routes_uncached(nodes, f, s);
                self.cache.insert(hash, routes.clone());

                routes
            }
        }
    }

    pub fn get_routes_uncached(
        nodes: &HashMap<usize, HashSet<usize>>,
        f: usize,
        s: usize,
    ) -> HashMap<usize, Vec<Vec<usize>>> {
        let mut q = VecDeque::new();
        let mut node_paths: HashMap<usize, Vec<Vec<usize>>> = HashMap::new();

        for (n, _) in nodes {
            node_paths.insert(*n, Vec::new());
        }

        node_paths.insert(s, vec![vec![s]]);
        q.push_back(vec![s]);

        while !q.is_empty() {
            let mut possible_paths: HashMap<usize, Vec<Vec<usize>>> = HashMap::new();
            let changed = q.clone();
            q.clear();

            for path in changed {
                let neigh = nodes.get(path.last().unwrap()).unwrap();

                for other in neigh {
                    let mut valid = true;
                    let other_paths = node_paths.get(other).unwrap();

                    if other_paths.len() > f {
                        continue;
                    }

                    for op in other_paths {
                        for path_item in &path {
                            if op.contains(&path_item) && *path_item != s {
                                valid = false;
                                break;
                            }
                        }

                        if !valid {
                            break;
                        }
                    }

                    if valid {
                        let mut new_path = path.clone();
                        new_path.push(*other);

                        if let Some(pl) = possible_paths.get_mut(other) {
                            pl.push(new_path);
                        } else {
                            possible_paths.insert(*other, vec![new_path]);
                        }
                    }
                }
            }

            for (node, paths) in possible_paths {
                let mut ordered_hashed: Vec<(String, Vec<usize>)> = Vec::new();

                for path in paths {
                    let mut hasher = Sha512::new();

                    for entry in &path {
                        hasher.update(entry.to_string());
                    }

                    let hash = format!("{:X}", hasher.finalize());
                    ordered_hashed.push((hash, path.clone()));
                }

                ordered_hashed.sort_by(|(a, _), (b, _)| a.cmp(b));
                let existing = node_paths.get_mut(&node).unwrap();

                for _ in 0..(f + 1 - existing.len()) {
                    if let Some((_, p)) = ordered_hashed.pop() {
                        existing.push(p.to_vec());
                        q.push_back(p.to_vec());
                    } else {
                        break;
                    }
                }
            }
        }

        node_paths
    }

    fn hash_params(nodes: &HashMap<usize, HashSet<usize>>, f: usize, s: usize) -> String {
        let mut hasher = Sha512::new();
        let mut nodes = nodes
            .iter()
            .map(|(k, v)| {
                let mut items = v.iter().map(|v| v.to_string()).collect::<Vec<String>>();
                items.sort();

                format!("{}-{}", k, items.join(","))
            })
            .collect::<Vec<String>>();
        nodes.sort();

        hasher.update(nodes.join("|"));
        hasher.update(f.to_string());
        hasher.update(s.to_string());

        format!("{:X}", hasher.finalize())
    }
}
