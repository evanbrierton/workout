use petgraph::algo::min_spanning_tree;
use petgraph::data::FromElements;
use petgraph::graph::{EdgeIndex, NodeIndex, UnGraph};
use std::collections::{HashMap, HashSet, VecDeque};

// function that returns the cycle length of the passed route
fn measure_route(route: &VecDeque<usize>, ddv: &[Vec<u32>]) -> u32 {
    let mut len = 0;
    for i in 1..route.len() {
        len += ddv[route[i - 1]][route[i]];
    }
    len + ddv[route[0]][route[route.len() - 1]]
}

// Travelling salesman solver - the strategy is:
// 1) create a minimal spanning tree
// 2) reduce all nodes to two or fewer connections by deleting the most expensive connections
// 3) connect all nodes with 0 or 1 connections to each other via the least expensive connections
fn tsp<N: std::clone::Clone>(g: &UnGraph<N, u32>) -> u32 {
    // translation collections: NodeIndex <-> usize
    let n_to_ni: Vec<NodeIndex> = g.node_indices().collect();
    let mut ni_to_n: HashMap<NodeIndex, usize> = HashMap::new();
    for (n, ni) in g.node_indices().enumerate() {
        ni_to_n.insert(ni, n);
    }

    // the original distance data in a vector
    let mut ddv: Vec<Vec<u32>> = vec![vec![u32::MAX; g.node_count()]; g.node_count()];
    for x in 0..g.node_count() {
        ddv[x][x] = 0;
        for y in x + 1..g.node_count() {
            let mut edges = g.edges_connecting(n_to_ni[x], n_to_ni[y]);
            let mut shortest_edge = u32::MAX;
            while let Some(edge) = edges.next() {
                if *edge.weight() < shortest_edge {
                    shortest_edge = *edge.weight();
                }
            }
            ddv[x][y] = shortest_edge;
            ddv[y][x] = shortest_edge;
        }
    }

    // create a graph with only the needed edges to form a minimum spanning tree
    let mut mst = UnGraph::<_, _>::from_elements(min_spanning_tree(&g));

    // delete most expensive connections to reduce connections to 2 or fewer for each node
    'rem_loop: loop {
        for ni1 in mst.node_indices() {
            let mut ev: Vec<(u32, EdgeIndex)> = vec![];
            for ni2 in mst.node_indices() {
                if let Some(ei) = mst.find_edge(ni1, ni2) {
                    ev.push((*mst.edge_weight(ei).unwrap(), ei));
                }
            }
            if ev.len() > 2 {
                ev.sort();
                mst.remove_edge(ev[2].1);
                // since we modified mst, need to start over as one other EdgeIndex will be invalid
                continue 'rem_loop;
            }
        }
        break;
    }

    // build a vector of routes from the nodes
    let mut routes: Vec<VecDeque<usize>> = vec![];
    let mut no_edges: Vec<usize> = vec![];
    let mut visited: HashSet<usize> = HashSet::new();
    let mut stack: VecDeque<usize> = VecDeque::default();
    for n in 0..mst.node_count() {
        if !visited.contains(&n) {
            stack.push_back(n);
        }

        while !stack.is_empty() {
            let n2 = stack.pop_front().unwrap();
            let mut eflag = false;
            visited.insert(n2);

            for n3 in 0..mst.node_count() {
                if mst.find_edge(n_to_ni[n2], n_to_ni[n3]).is_some() {
                    eflag = true;
                    if !visited.contains(&n3) {
                        stack.push_back(n3);
                        let mut fflag = false;
                        for r in routes.iter_mut() {
                            if r[0] == n2 {
                                r.push_front(n3);
                                fflag = true;
                            } else if r[r.len() - 1] == n2 {
                                r.push_back(n3);
                                fflag = true;
                            } else if r[0] == n3 {
                                r.push_front(n2);
                                fflag = true;
                            } else if r[r.len() - 1] == n3 {
                                r.push_back(n2);
                                fflag = true;
                            }
                        }
                        if !fflag {
                            // not found, create a new VecDeque
                            let mut vd = VecDeque::default();
                            vd.push_back(n2);
                            vd.push_back(n3);
                            routes.push(vd);
                        }
                    }
                }
            }
            if !eflag {
                no_edges.push(n2);
            }
        }
    }

    // put each node with no edges on the end of a route based on cost
    for n in &no_edges {
        let mut route_num = usize::MAX;
        let mut insert_loc = 0;
        let mut shortest = u32::MAX;
        for ridx in 0..routes.len() {
            if ddv[*n][routes[ridx][0]] < shortest {
                shortest = ddv[*n][routes[ridx][0]];
                route_num = ridx;
                insert_loc = 0;
            }
            if ddv[routes[ridx][routes[ridx].len() - 1]][*n] < shortest {
                shortest = ddv[routes[ridx][routes[ridx].len() - 1]][*n];
                route_num = ridx;
                insert_loc = routes[ridx].len() - 1;
            }
        }
        if route_num == usize::MAX || shortest == u32::MAX {
            panic!("unable to deal with singleton node {}", *n);
        } else if insert_loc != 0 {
            routes[route_num].push_back(*n);
        } else {
            routes[route_num].push_front(*n);
        }
    }

    // merge routes into a single route based on cost - this could be improved by doing comparisons
    // between routes[n] and routes[m] where m != 0 and n != 0
    let mut tour = routes[0].clone();
    for ridx in 1..routes.len() {
        let mut v: Vec<(u32, bool, bool)> = vec![];
        v.push((ddv[routes[ridx][routes[ridx].len() - 1]][tour[0]], true, false));
        v.push((ddv[routes[ridx][routes[ridx].len() - 1]][tour[tour.len() - 1]], true, true));
        v.push((ddv[routes[ridx][0]][tour[0]], false, false));
        v.push((ddv[routes[ridx][0]][tour[tour.len() - 1]], false, true));
        v.sort_unstable();
        match v[0] {
            (_, true, false) => {
                // end to beginning of tour
                for (insert_loc, n) in routes[ridx].iter().enumerate() {
                    tour.insert(insert_loc, *n);
                }
            }

            (_, true, true) => {
                // end to end of tour
                let insert_loc = tour.len();
                for n in &routes[ridx] {
                    tour.insert(insert_loc, *n);
                }
            }

            (_, false, false) => {
                // beginning to beginning of tour
                for n in &routes[ridx] {
                    tour.push_front(*n);
                }
            }

            (_, false, true) => {
                // beginning to end of tour
                for n in &routes[ridx] {
                    tour.push_back(*n);
                }
            }
        }
    }

    // print out the tour and return its length
    dbg!(tour.clone());
    measure_route(&tour, &ddv)
}
