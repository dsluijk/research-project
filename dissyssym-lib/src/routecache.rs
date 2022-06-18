use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use priority_queue::PriorityQueue;
use sha2::{Digest, Sha512};

use crate::topology::FlowGraph;

pub struct RouteCache {
    method: String,
    cache: HashMap<String, Option<Arc<HashMap<usize, HashSet<usize>>>>>,
}

impl RouteCache {
    pub fn new(method: String) -> Self {
        RouteCache {
            method,
            cache: HashMap::new(),
        }
    }

    pub fn gen_routes(
        &mut self,
        nodes: &HashMap<usize, HashSet<usize>>,
        f: usize,
        s: usize,
    ) -> Option<Arc<HashMap<usize, HashSet<usize>>>> {
        let hash = Self::hash_params(nodes, f, s);

        match self.cache.get(&hash) {
            Some(routes) => routes.clone(),
            None => {
                let routes = match self.gen_routes_uncached(nodes, f, s) {
                    Some(r) => Some(Arc::new(r)),
                    None => None,
                };
                self.cache.insert(hash, routes.clone());

                routes
            }
        }
    }

    pub fn gen_routes_uncached(
        &self,
        nodes: &HashMap<usize, HashSet<usize>>,
        f: usize,
        s: usize,
    ) -> Option<HashMap<usize, HashSet<usize>>> {
        match self.method.as_str() {
            "pathfind" => Self::gen_method_pathfind(nodes, f, s),
            "unreliable" => Self::gen_method_unreliable(nodes, f, s),
            _ => panic!("Invalid path generation method!"),
        }
    }

    pub fn gen_method_pathfind(
        nodes: &HashMap<usize, HashSet<usize>>,
        f: usize,
        s: usize,
    ) -> Option<HashMap<usize, HashSet<usize>>> {
        let mut routes = HashMap::new();
        let mut used = HashMap::new();
        let mut accepted = HashMap::new();

        for (&n, _) in nodes {
            routes.insert(n, HashSet::new());
            used.insert(n, HashSet::new());
            accepted.insert(n, 0);
        }

        let s_nodes = nodes.get(&s).unwrap();
        routes.insert(s, s_nodes.clone());

        for &neigh in s_nodes {
            *accepted.get_mut(&neigh).unwrap() = 1;
            used.get_mut(&neigh).unwrap().insert(s);
        }

        while !accepted.iter().all(|(&i, &a)| a > f || i == s) {
            let consider = HashSet::from([s]);
            let coloured = &mut HashSet::from([s]);

            let t = match Self::gen_method_pathfind_target(f, consider, nodes, &accepted, coloured)
            {
                Some(t) => t,
                None => break,
            };

            let path = match Self::gen_method_pathfind_path(s, t, nodes, &used, &routes) {
                Some(p) => p,
                None => return None,
            };

            accepted.insert(t, accepted.get(&t).unwrap() + 1);

            for i in 0..(path.len() - 1) {
                routes.get_mut(&path[i]).unwrap().insert(path[i + 1]);
                used.get_mut(&t).unwrap().insert(path[i]);
            }
        }

        Some(routes)
    }

    fn gen_method_pathfind_target(
        f: usize,
        consider: HashSet<usize>,
        nodes: &HashMap<usize, HashSet<usize>>,
        accepted: &HashMap<usize, usize>,
        coloured: &mut HashSet<usize>,
    ) -> Option<usize> {
        if consider.is_empty() {
            return None;
        }

        let mut best = None;
        for &i in &consider {
            if coloured.contains(&i) {
                continue;
            }

            coloured.insert(i);

            let accepted = *accepted.get(&i).unwrap();
            if accepted > f {
                continue;
            }

            if let Some((_, b)) = best {
                if accepted > b {
                    continue;
                }
            }

            best = Some((i, accepted));
        }

        if let Some((i, _)) = best {
            return Some(i);
        }

        let mut new_consider = HashSet::new();
        for i in &consider {
            for neigh in nodes.get(i).unwrap() {
                new_consider.insert(*neigh);
            }
        }

        Self::gen_method_pathfind_target(f, new_consider, nodes, accepted, coloured)
    }

    fn gen_method_pathfind_path(
        s: usize,
        t: usize,
        nodes: &HashMap<usize, HashSet<usize>>,
        used: &HashMap<usize, HashSet<usize>>,
        routes: &HashMap<usize, HashSet<usize>>,
    ) -> Option<Vec<usize>> {
        let curr_used = used.get(&t).unwrap();

        for d in 1..nodes.len() {
            let potential = Self::gen_method_pathfind_path_potential(s, t, d, nodes, curr_used);
            let mut best = None;

            for path in &potential {
                let conn = Self::gen_method_pathfind_partial_connectivity(nodes, used, &path, s, t);
                let mut overlap = 0;

                for other in &potential {
                    for i in 1..(other.len() - 1) {
                        if path.contains(&other[i]) {
                            overlap += 1;
                        }
                    }
                }

                let mut add = 0;
                for i in 0..(path.len() - 1) {
                    if routes.get(&path[i]).unwrap().contains(&path[i + 1]) {
                        continue;
                    }

                    add += 1;
                }

                if let Some((_, best_conn, best_overlap, best_add)) = best {
                    if best_conn > conn {
                        continue;
                    }

                    if best_overlap < overlap && best_conn == conn {
                        continue;
                    }

                    if best_add <= add && best_overlap == overlap && best_conn == conn {
                        continue;
                    }

                    best = Some((path, conn, overlap, add));
                } else {
                    best = Some((path, conn, overlap, add));
                }
            }

            if let Some((path, _, _, _)) = best {
                return Some(path.clone());
            }
        }

        None
    }

    fn gen_method_pathfind_path_potential(
        s: usize,
        t: usize,
        d: usize,
        nodes: &HashMap<usize, HashSet<usize>>,
        used: &HashSet<usize>,
    ) -> Vec<Vec<usize>> {
        let mut pq = PriorityQueue::new();
        pq.push(vec![s], Reverse(0));

        let mut paths = Vec::new();

        while let Some((path, Reverse(p))) = pq.pop() {
            if p > d {
                break;
            }

            let &last = path.last().unwrap();
            if last == t {
                paths.push(path.clone());
            }

            for &neigh in nodes.get(&last).unwrap() {
                if path.contains(&neigh) {
                    continue;
                }

                if used.contains(&neigh) {
                    continue;
                }

                let mut new_path = path.clone();
                new_path.push(neigh);

                pq.push(new_path, Reverse(p + 1));
            }
        }

        paths
    }

    fn gen_method_pathfind_partial_connectivity(
        nodes: &HashMap<usize, HashSet<usize>>,
        used: &HashMap<usize, HashSet<usize>>,
        proposed: &Vec<usize>,
        s: usize,
        t: usize,
    ) -> usize {
        let mut edges = Vec::new();
        let used_t = used.get(&t).unwrap();

        for (&n, neighbours) in nodes {
            if proposed.contains(&n) && n != s && n != t {
                continue;
            }

            if used_t.contains(&n) && n != s && n != t {
                continue;
            }

            for &neigh in neighbours {
                if proposed.contains(&neigh) && neigh != s && neigh != t {
                    continue;
                }

                if used_t.contains(&neigh) && neigh != s && neigh != t {
                    continue;
                }

                edges.push((n, neigh));
            }
        }

        FlowGraph::new(&edges).max_flow(s, t)
    }

    pub fn gen_method_unreliable(
        nodes: &HashMap<usize, HashSet<usize>>,
        f: usize,
        s: usize,
    ) -> Option<HashMap<usize, HashSet<usize>>> {
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

        let mut routes = HashMap::new();
        for (&n, _) in nodes {
            let mut to = HashSet::new();

            for neigh in nodes.get(&n).unwrap() {
                for path in node_paths.get(neigh).unwrap() {
                    for i in 0..path.len() {
                        if path[i] != n {
                            continue;
                        }

                        if path[i + 1] == s {
                            continue;
                        }

                        to.insert(path[i + 1]);
                    }
                }
            }

            routes.insert(n, to);
        }

        Some(routes)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pathfind_connectivity_cutoff() {
        let mut nodes = HashMap::new();
        nodes.insert(0, HashSet::from([3, 4, 6, 8]));
        nodes.insert(1, HashSet::from([2, 4, 5, 6]));
        nodes.insert(2, HashSet::from([1, 3, 6, 7]));
        nodes.insert(3, HashSet::from([0, 2, 5, 8]));
        nodes.insert(4, HashSet::from([0, 1, 6, 7]));
        nodes.insert(5, HashSet::from([1, 3, 7, 8]));
        nodes.insert(6, HashSet::from([0, 1, 2, 4]));
        nodes.insert(7, HashSet::from([2, 4, 5, 8]));
        nodes.insert(8, HashSet::from([0, 3, 5, 7]));

        let used = HashMap::from([
            (0, HashSet::new()),
            (1, HashSet::from([3, 5, 2])),
            (2, HashSet::new()),
            (3, HashSet::new()),
            (4, HashSet::new()),
            (5, HashSet::new()),
            (6, HashSet::new()),
            (7, HashSet::new()),
            (8, HashSet::new()),
        ]);

        let connect =
            RouteCache::gen_method_pathfind_partial_connectivity(&nodes, &used, &vec![], 3, 1);
        assert_eq!(connect, 2);

        let connect = RouteCache::gen_method_pathfind_partial_connectivity(
            &nodes,
            &used,
            &vec![3, 0, 4, 1],
            3,
            1,
        );
        assert_eq!(connect, 0);

        let connect = RouteCache::gen_method_pathfind_partial_connectivity(
            &nodes,
            &used,
            &vec![3, 0, 6, 1],
            3,
            1,
        );
        assert_eq!(connect, 1);
    }
}
