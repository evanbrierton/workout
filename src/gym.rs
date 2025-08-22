use std::{
    collections::{HashMap, HashSet}
};

use itertools::Itertools;
use multimap::MultiMap;
use petgraph::{
    algo,
    graph::{NodeIndex, UnGraph},
};

use crate::{
    bar::Bar, bar_kind::BarKind, dumbbell::{Dumbbell, DumbbellId}, gym_error::GymError, gym_state::{GymState, GymStateId}, plate::Plate, requirement::Requirement
};

pub struct Gym {
    states: Vec<GymState>,
    graph: UnGraph<GymStateId, u32>,
    nodes: HashMap<GymStateId, NodeIndex>,
    weights: HashMap<Bar, Vec<u32>>,
    bar_options: HashMap<BarKind, Vec<Bar>>,
}

impl Gym {
    #[must_use] pub fn new(plates: &HashMap<Plate, usize>, bars: &[Bar]) -> Self {
        let dumbbells: HashMap<Bar, Vec<Dumbbell>> = bars
            .iter()
            .map(|bar| (*bar, Self::dumbbells(plates, bar)))
            .collect();

        let weights = dumbbells
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
        .collect();

       let states = dumbbells.values()
            .multi_cartesian_product()
            .map(|dumbbells| {
                GymState::new(
                dumbbells
                    .into_iter()
                    .map(|dumbbell| (*dumbbell.bar(), dumbbell.clone()))
                    .collect::<HashMap<_, _>>())
            })
            .collect::<Vec<_>>();

        let ids = states.iter().enumerate()
            .map(|(i, _)| GymStateId(i))
            .collect::<HashSet<_>>();

        let (graph, nodes) = Self::graph(&states, &ids);

        let bar_options: HashMap<BarKind, Vec<Bar>> =
            bars.iter().fold(HashMap::new(), |mut acc, bar| {
                acc.entry(*bar.kind()).or_default().push(*bar);
                acc
            });

        Gym {
            states,
            graph,
            nodes,
            weights,
            bar_options,
        }
    }

    ///
    /// # Errors
    /// If it is impossible to construct a dumbbell for a requirement given the user's plates.
    ///
    pub fn order(
        &self,
        requirements: &[Requirement]
    ) -> anyhow::Result<HashMap<Bar, Vec<&Dumbbell>>, GymError> {
        let mut result = HashMap::new();

        let shorted_distances = algo::johnson(&self.graph, |e| *e.weight());

        Result::Ok(result)
    }


    #[must_use] pub fn weights(&self) -> &HashMap<Bar, Vec<u32>> {
        &self.weights
    }

    fn dumbbells(weights_map: &HashMap<Plate, usize>, bar: &Bar) -> Vec<Dumbbell> {
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
        states: &Vec<GymState>,
        ids: &HashSet<GymStateId>,
    ) -> (UnGraph<GymStateId, u32>, HashMap<GymStateId, NodeIndex>) {
        let mut graph = UnGraph::<GymStateId, u32>::new_undirected();
        let mut nodes = HashMap::new();

        for id in ids {
            let node_index = graph.add_node(*id);
            nodes.insert(*id, node_index);
        }

        for ((i1, state1), (i2, state2)) in states.iter().enumerate().tuple_combinations() {
            if state1.adjacent(state2) {
                let n1 = nodes[&GymStateId(i1)];
                let n2 = nodes[&GymStateId(i2)];
                graph.add_edge(n1, n2, 1);
            }
        }


        (graph, nodes)
    }
}
