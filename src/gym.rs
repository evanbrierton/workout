use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use itertools::Itertools;
use petgraph::{
    algo,
    graph::{NodeIndex, UnGraph},
};

use crate::{
    bar::Bar, bar_kind::BarKind, dumbbell::Dumbbell, plate::Plate, requirement::Requirement,
};

pub struct Gym {
    dumbbells: HashMap<Bar, HashSet<Rc<Dumbbell>>>,
    graphs: HashMap<Bar, UnGraph<Rc<Dumbbell>, u32>>,
    nodes: HashMap<Bar, HashMap<Rc<Dumbbell>, NodeIndex>>,
    zeroes: HashMap<Bar, Rc<Dumbbell>>,
    bar_options: HashMap<BarKind, Vec<Bar>>,
}

impl Gym {
    pub fn new(plates: &HashMap<Plate, usize>, bars: &[Bar]) -> Self {
        let dumbbells: HashMap<Bar, HashSet<Rc<Dumbbell>>> = bars
            .iter()
            .map(|bar| (*bar, Self::dumbells(plates, bar)))
            .collect();

        let mut graphs = HashMap::new();
        let mut nodes = HashMap::new();

        for (bar, dumbbells_set) in &dumbbells {
            let (graph, node_map) = Self::graph(dumbbells_set);
            graphs.insert(*bar, graph);
            nodes.insert(*bar, node_map);
        }

        let zeroes = bars
            .iter()
            .map(|bar| (*bar, Dumbbell::new_rc(vec![], bar)))
            .collect();

        let bar_options: HashMap<BarKind, Vec<Bar>> =
            bars.iter().fold(HashMap::new(), |mut acc, bar| {
                acc.entry(*bar.kind()).or_insert_with(Vec::new).push(*bar);
                acc
            });

        Gym {
            dumbbells,
            graphs,
            nodes,
            zeroes,
            bar_options,
        }
    }

    pub fn order(
        &self,
        requirements: &HashMap<BarKind, Vec<Requirement>>,
    ) -> anyhow::Result<HashMap<Bar, Vec<Rc<Dumbbell>>>> {
        let mut bar_states = HashMap::new();
        let mut result = HashMap::new();

        for (bar_kind, reqs) in requirements {
            let bars = &self.bar_options[bar_kind];

            for req in reqs {
                let (bar, dumbbell) = bars
                    .iter()
                    .map(|bar| {
                        let start = bar_states.get(bar).unwrap_or_else(|| &self.zeroes[bar]);
                        (bar, self.path(start, bar, req.weight))
                    })
                    .filter_map(|(bar, path)| {
                        path.map(|(weight, dumbbell)| (bar, (weight, dumbbell)))
                    })
                    .min_by_key(|(_, (weight, _))| *weight)
                    .map(|(bar, (_, dumbbell))| (bar, dumbbell))
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "No path found for {} with required weight {}",
                            bar_kind,
                            req.weight
                        )
                    })?;

                bar_states.insert(*bar, dumbbell.clone());
                result.entry(*bar).or_insert_with(Vec::new).push(dumbbell);
            }
        }

        Result::Ok(result)
    }

    pub fn weights(&self) -> HashMap<Bar, Vec<u32>> {
        self.dumbbells
            .iter()
            .map(|(bar, dumbbells)| {
                let weights = dumbbells
                    .iter()
                    .map(|dumbbell| dumbbell.weight())
                    .sorted()
                    .dedup()
                    .collect();
                (*bar, weights)
            })
            .collect()
    }

    fn path(
        &self,
        start: &Rc<Dumbbell>,
        bar: &Bar,
        target_weight: u32,
    ) -> Option<(u32, Rc<Dumbbell>)> {
        let graph = self.graphs.get(bar)?;
        let nodes = self.nodes.get(bar)?;
        let start_node = nodes.get(start)?;

        let path = algo::astar(
            graph,
            *start_node,
            |n| graph[n].weight() == target_weight,
            |e| *e.weight(),
            |_| 0,
        )?;

        let last_node_index = path.1.last()?;
        let last_node = graph.node_weight(*last_node_index)?;
        Option::Some((path.0, last_node.clone()))
    }

    fn dumbells(weights_map: &HashMap<Plate, usize>, bar: &Bar) -> HashSet<Rc<Dumbbell>> {
        Self::available_dumbbells(
            &weights_map
                .iter()
                .filter(|(_, count)| *count >= &bar.kind().required_similar_plates())
                .map(|(plate, count)| (*plate, count / bar.kind().required_similar_plates()))
                .flat_map(|(plate, count)| vec![plate; count])
                .collect::<Vec<_>>(),
            bar,
        )
    }

    fn available_dumbbells(plates: &[Plate], bar: &Bar) -> HashSet<Rc<Dumbbell>> {
        plates
            .iter()
            .powerset()
            .into_iter()
            .map(|plates| Dumbbell::new_rc(plates.into_iter().copied().collect(), bar))
            .collect()
    }

    fn graph(
        dumbbells: &HashSet<Rc<Dumbbell>>,
    ) -> (UnGraph<Rc<Dumbbell>, u32>, HashMap<Rc<Dumbbell>, NodeIndex>) {
        let mut graph = UnGraph::<Rc<Dumbbell>, u32>::new_undirected();
        let mut nodes = HashMap::new();

        for dumbbell in dumbbells {
            let node_index = graph.add_node(dumbbell.clone());
            nodes.insert(dumbbell.clone(), node_index);
        }

        for (d1, d2) in dumbbells.iter().tuple_combinations() {
            if d1.weight() == d2.weight() {
                continue;
            }

            let d1_plates = d1.plates();
            let d2_plates = d2.plates();

            if (d1_plates.len() as i32 - d2_plates.len() as i32).abs() == 1 {
                let adjacent = d1_plates
                    .iter()
                    .zip(d2_plates)
                    .all(|(p1, p2)| p1.weight() == p2.weight());

                if adjacent {
                    let node1 = nodes[d1];
                    let node2 = nodes[d2];
                    graph.add_edge(node1, node2, 1);
                }
            }
        }

        (graph, nodes)
    }
}
