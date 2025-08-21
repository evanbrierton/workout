use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    hash::Hash,
};

use itertools::Itertools;
use petgraph::{graph::NodeIndex, graph::UnGraph};

use crate::{bar::Bar, plate::Plate};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]

pub struct Dumbbell {
    pub plates: Vec<Plate>,
    pub bar: Bar,
}

impl Dumbbell {
    pub fn new(plates: Vec<Plate>, bar: Bar) -> Self {
        Dumbbell {
            plates: plates
                .into_iter()
                .sorted()
                .rev()
                .filter(|p| p.gauge == bar.gauge)
                .collect(),
            bar,
        }
    }

    pub fn weight(&self) -> u32 {
        self.bar.weight + self.plates.clone().into_iter().sum::<Plate>().weight * 2
    }

    pub fn available(plates: Vec<Plate>, bar: Bar) -> HashSet<Dumbbell> {
        plates
            .iter()
            .powerset()
            .into_iter()
            .map(|plates| Dumbbell::new(plates.into_iter().copied().collect(), bar.clone()))
            .collect()
    }

    pub fn available_from_weight_map(
        weights_map: HashMap<Plate, usize>,
        bar: Bar,
    ) -> HashSet<Dumbbell> {
        Dumbbell::available(
            weights_map
                .into_iter()
                .filter(|(_, count)| *count >= bar.kind.required_similar_plates())
                .map(|(plate, count)| (plate, count / bar.kind.required_similar_plates()))
                .flat_map(|(plate, count)| vec![plate; count])
                .collect(),
            bar,
        )
    }

    pub fn sort_and_dedupe(dumbbells: Vec<Dumbbell>) -> Vec<Dumbbell> {
        dumbbells
            .into_iter()
            .sorted_by(|a, b| {
                a.bar
                    .kind
                    .cmp(&b.bar.kind)
                    .then_with(|| a.weight().cmp(&b.weight()))
                    .then_with(|| a.plates.len().cmp(&b.plates.len()))
            })
            .dedup_by(|a, b| a.weight() == b.weight() && a.bar.kind == b.bar.kind)
            .collect::<Vec<_>>()
    }

    pub fn tree(
        dumbbells: &HashSet<Dumbbell>,
    ) -> (UnGraph<Dumbbell, u32>, HashMap<Dumbbell, NodeIndex>) {
        let mut graph = UnGraph::<Dumbbell, u32>::new_undirected();
        let mut nodes = HashMap::new();

        for dumbbell in dumbbells {
            let node_index = graph.add_node(dumbbell.clone());
            nodes.insert(dumbbell.clone(), node_index);
        }

        for (d1, d2) in dumbbells.iter().tuple_combinations() {
            if d1.weight() == d2.weight() {
                continue;
            }

            let d1_plates = d1.plates.clone();
            let d2_plates = d2.plates.clone();

            if (d1_plates.len() as i32 - d2_plates.len() as i32).abs() == 1 {
                let adjacent = d1_plates
                    .iter()
                    .zip(d2_plates)
                    .all(|(p1, p2)| p1.weight == p2.weight);

                if adjacent {
                    let node1 = nodes.get(d1).unwrap();
                    let node2 = nodes.get(d2).unwrap();
                    graph.add_edge(*node1, *node2, 1);
                }
            }
        }

        (graph, nodes)
    }

    pub fn to_node_string(&self) -> String {
        let kg_plates = self
            .plates
            .iter()
            .map(|p| p.weight as f64 / 1000.0)
            .collect::<Vec<_>>();

        format!("{:?}", kg_plates,)
    }
}

impl Display for Dumbbell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kg_plates = self
            .plates
            .iter()
            .map(|p| p.weight as f64 / 1000.0)
            .collect::<Vec<_>>();

        write!(
            f,
            "{} ({}) {:?} ({}kg)",
            self.bar.kind,
            self.bar.gauge,
            kg_plates,
            self.weight() as f64 / 1000.0,
        )
    }
}
