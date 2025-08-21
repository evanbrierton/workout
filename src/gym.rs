use std::{
    collections::{HashMap, HashSet}
};

use itertools::Itertools;
use petgraph::{
    algo,
    graph::{NodeIndex, UnGraph},
};

use crate::{
    bar::Bar, bar_kind::BarKind, dumbbell::{Dumbbell, DumbbellId}, plate::Plate, requirement::Requirement,
};

pub struct Gym {
    dumbbells: HashMap<Bar, Vec<Dumbbell>>,
    graphs: HashMap<Bar, UnGraph<DumbbellId, u32>>,
    nodes: HashMap<Bar, HashMap<DumbbellId, NodeIndex>>,
    bar_options: HashMap<BarKind, Vec<Bar>>,
}

impl Gym {
    #[must_use] pub fn new(plates: &HashMap<Plate, usize>, bars: &[Bar]) -> Self {
        let dumbbells: HashMap<Bar, Vec<Dumbbell>> = bars
            .iter()
            .map(|bar| (*bar, Self::dumbells(plates, bar)))
            .collect();

        let mut graphs = HashMap::new();
        let mut nodes = HashMap::new();

        let ids = dumbbells
            .iter()
            .map(|(bar, dumbbells)| {
                (
                    *bar,
                    dumbbells
                        .iter()
                        .enumerate()
                        .map(|(index, _)| DumbbellId(index))
                        .collect::<HashSet<_>>(),
                )
            })
            .collect::<HashMap<_, _>>();

        for (bar, ids) in &ids {
            let (graph, node_map) = Self::graph(&dumbbells[bar], ids);
            graphs.insert(*bar, graph);
            nodes.insert(*bar, node_map);
        }

        let bar_options: HashMap<BarKind, Vec<Bar>> =
            bars.iter().fold(HashMap::new(), |mut acc, bar| {
                acc.entry(*bar.kind()).or_default().push(*bar);
                acc
            });

        Gym {
            dumbbells,
            graphs,
            nodes,
            bar_options,
        }
    }

    /**
     * # Errors
     * If it is impossible to construct a dumbbell for a requirement given the user's plates.
     */
    pub fn order(
        &self,
        requirements: &HashMap<BarKind, Vec<Requirement>>,
    ) -> anyhow::Result<HashMap<Bar, Vec<&Dumbbell>>> {
        let mut bar_states = HashMap::new();
        let mut result = HashMap::new();

        for (bar_kind, reqs) in requirements {
            let bars = &self.bar_options[bar_kind];

            for req in reqs {
                let (bar, id) = bars
                    .iter()
                    .map(|bar| {
                        let start = bar_states.get(bar).unwrap_or_default();
                        (bar, self.path(*start, bar, req.weight))
                    })
                    .filter_map(|(bar, path)| {
                        path.map(|(weight, id)| (bar, (weight, id)))
                    })
                    .min_by_key(|(_, (weight, _))| *weight)
                    .map(|(bar, (_, id))| (bar, id))
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "There is no dumbbell for requirement {:?} with bar kind {:?}",
                            req,
                            bar_kind
                        )
                    })?;

                bar_states.insert(*bar, id);
                result.entry(*bar).or_insert_with(Vec::new).push(&self.dumbbells[bar][id.0]);
            }
        }

        Result::Ok(result)
    }


    #[must_use] pub fn weights(&self) -> HashMap<Bar, Vec<u32>> {
        self.dumbbells
            .iter()
            .map(|(bar, dumbbells)| {
                let weights = dumbbells
                    .iter()
                    .map(Dumbbell::weight)
                    .sorted()
                    .dedup()
                    .collect();
                (*bar, weights)
            })
            .collect()
    }

    fn path(
        &self,
        start: DumbbellId,
        bar: &Bar,
        target_weight: u32,
    ) -> Option<(u32, DumbbellId)> {
        let graph = self.graphs.get(bar)?;
        let nodes = self.nodes.get(bar)?;
        let start_node = nodes.get(&start)?;

        let path = algo::astar(
            graph,
            *start_node,
            |n| self.dumbbells[bar][graph[n].0].weight() == target_weight,
            |e| *e.weight(),
            |_| 0,
        )?;

        let last_node_index = path.1.last()?;
        let last_node = graph.node_weight(*last_node_index)?;
        Option::Some((path.0, *last_node))
    }

    fn dumbells(weights_map: &HashMap<Plate, usize>, bar: &Bar) -> Vec<Dumbbell> {
        Self::available_dumbbells(
            &weights_map
                .iter()
                .filter(|(_, count)| *count >= &bar.kind().required_similar_plates())
                .map(|(plate, count)| (*plate, count / bar.kind().required_similar_plates()))
                .flat_map(|(plate, count)| vec![plate; count])
                .collect::<Vec<_>>(),
            bar,
        ).into_iter().sorted().collect()
    }

    fn available_dumbbells(plates: &[Plate], bar: &Bar) -> HashSet<Dumbbell> {
        plates
            .iter()
            .powerset()
            .map(|plates| Dumbbell::new(plates.into_iter().copied().collect(), *bar))
            .collect::<HashSet<_>>()
    }

    fn graph(
        dumbbells: &[Dumbbell],
        ids: &HashSet<DumbbellId>,
    ) -> (UnGraph<DumbbellId, u32>, HashMap<DumbbellId, NodeIndex>) {
        let mut graph = UnGraph::<DumbbellId, u32>::new_undirected();
        let mut nodes = HashMap::new();

        for id in ids {
            let node_index = graph.add_node(*id);
            nodes.insert(*id, node_index);
        }

        for ((i1, d1), (i2, d2)) in dumbbells.iter().enumerate().tuple_combinations() {
            let d1_plates = d1.plates();
            let d2_plates = d2.plates();

            if (d1_plates.len() as i128 - d2_plates.len() as i128).abs() == 1 {
                let adjacent = d1_plates
                    .iter()
                    .zip(d2_plates)
                    .all(|(p1, p2)| p1.weight() == p2.weight());

                if adjacent {
                    let n1 = nodes[&DumbbellId(i1)];
                    let n2 = nodes[&DumbbellId(i2)];
                    graph.add_edge(n1, n2, 1);
                }
            }
        }

        (graph, nodes)
    }
}
