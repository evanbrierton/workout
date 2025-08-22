use std::collections::HashSet;

use hashbrown::HashMap;
use itertools::Itertools;
use petgraph::{
    algo,
    graph::{NodeIndex, UnGraph},
};

use crate::{
    bar::Bar,
    bar_kind::BarKind,
    dumbbell::Dumbbell,
    gym_error::GymError,
    gym_state::{GymState, GymStateId},
    plate::Plate,
    requirement::Requirement,
};

pub struct Gym {
    states: Vec<GymState>,
    graph: UnGraph<GymStateId, u32>,
    nodes: HashMap<GymStateId, NodeIndex>,
    weights: HashMap<Bar, Vec<u32>>,
    bar_options: HashMap<BarKind, Vec<Bar>>,
}

impl Gym {
    #[must_use]
    pub fn new(plates: &HashMap<Plate, usize>, bars: &[Bar]) -> Self {
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

        let states: Vec<_> = dumbbells
            .into_iter()
            .map(|(_, d)| d)
            .multi_cartesian_product()
            .map(|dumbbells| {
                GymState::new(
                    dumbbells
                        .into_iter()
                        .map(|dumbbell| (*dumbbell.bar(), dumbbell))
                        .collect::<HashMap<_, _>>(),
                )
            })
            .collect();

        let ids = states
            .iter()
            .enumerate()
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
        requirements: &[Requirement],
    ) -> Result<HashMap<Bar, Vec<&Dumbbell>>, GymError> {
        if requirements.is_empty() {
            return Ok(HashMap::new());
        }

        let optimal_sequence = self.find_optimal_sequence(requirements)?;

        let mut result = HashMap::<Bar, Vec<&Dumbbell>>::new();
        let mut requirement_index = 0;

        for state_id in optimal_sequence {
            let state = &self.states[state_id.0];
            let bars = self
                .bar_options
                .get(&requirements[requirement_index].bar_kind())
                .ok_or(GymError::InvalidRequirement(
                    requirements[requirement_index],
                ))?;

            for bar in bars {
                if let Some(dumbbell) = state.get(bar) {
                    if dumbbell.weight() == requirements[requirement_index].weight() {
                        result.entry(*bar).or_default().push(dumbbell);
                    }
                }
            }

            if requirement_index < requirements.len() - 1 {
                requirement_index += 1;
            }
        }

        Ok(result)
    }

    #[must_use]
    pub fn weights(&self) -> &HashMap<Bar, Vec<u32>> {
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
        )
        .into_iter()
        .sorted()
        .collect()
    }

    fn available_dumbbells(plates: &[Plate], bar: &Bar) -> HashSet<Dumbbell> {
        plates
            .iter()
            .powerset()
            .map(|plates| Dumbbell::new(plates.into_iter().copied().collect(), *bar))
            .collect::<HashSet<_>>()
    }

    fn graph(
        states: &[GymState],
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

    fn find_states_for_requirement(&self, requirement: Requirement) -> Vec<GymStateId> {
        let matching_states: Vec<(GymStateId, usize)> = self
            .states
            .iter()
            .enumerate()
            .filter_map(|(i, state)| {
                let bars = self.bar_options.get(&requirement.bar_kind())?;

                for bar in bars {
                    if let Some(dumbbell) = state.get(bar) {
                        if dumbbell.weight() == requirement.weight() {
                            return Some((GymStateId(i), dumbbell.plates().len()));
                        }
                    }
                }
                None
            })
            .collect();

        matching_states
            .into_iter()
            .sorted_by_key(|(_, complexity): &(GymStateId, usize)| *complexity)
            .map(|(id, _)| id)
            .collect()
    }

    fn find_optimal_sequence(
        &self,
        requirements: &[Requirement],
    ) -> Result<Vec<GymStateId>, GymError> {
        let requirement_states: Vec<Vec<GymStateId>> = requirements
            .iter()
            .map(|req| self.find_states_for_requirement(*req))
            .collect();

        let n = requirement_states.len();

        match n {
            0 => return Ok(vec![]),
            1 => return requirement_states[0].iter().min_by_key(|id| self.states[id.0].clone()).ok_or(GymError::InvalidRequirement(requirements[0])).map(|id| vec![*id]),
            _ => {}
        }

        let distances: HashMap<(NodeIndex, NodeIndex), u32> =
            algo::johnson(&self.graph, |e| *e.weight())
                .map_err(|_| GymError::InvalidRequirement(requirements[0]))?;

        let mut dp: Vec<HashMap<GymStateId, (u32, Option<GymStateId>)>> = vec![HashMap::new(); n];

        for &state in &requirement_states[0] {
            dp[0].insert(state, (0, None));
        }

        for i in 1..n {
            for &current_state in &requirement_states[i] {
                let current_node = self.nodes[&current_state];
                let mut min_cost = u32::MAX;
                let mut best_prev = None;

                let mut prev_states: Vec<_> = dp[i - 1]
                    .iter()
                    .map(|(&state, &(cost, _))| (state, cost))
                    .collect();
                prev_states.sort_by_key(|&(state, _)| state);

                for (prev_state, prev_cost) in prev_states {
                    let prev_node = self.nodes[&prev_state];
                    if let Some(&transition_cost) = distances.get(&(prev_node, current_node)) {
                        let total_cost = prev_cost.saturating_add(transition_cost);

                        if total_cost < min_cost {
                            min_cost = total_cost;
                            best_prev = Some(prev_state);
                        }
                    }
                }

                if min_cost != u32::MAX {
                    dp[i].insert(current_state, (min_cost, best_prev));
                }
            }
        }

        let (&final_state, _) = dp[n - 1]
            .iter()
            .min_by_key(|(_, (cost, _))| *cost)
            .ok_or(GymError::InvalidRequirement(requirements[n - 1]))?;

        let mut path = Vec::new();
        let mut current = final_state;
        path.push(current);

        for i in (0..n - 1).rev() {
            if let Some((_, Some(prev))) = dp[i + 1].get(&current) {
                current = *prev;
                path.push(current);
            }
        }

        path.reverse();
        Ok(path)
    }
}
