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

        let states = dumbbells
            .values()
            .multi_cartesian_product()
            .map(|dumbbells| {
                GymState::new(
                    dumbbells
                        .into_iter()
                        .map(|dumbbell| (*dumbbell.bar(), dumbbell.clone()))
                        .collect::<HashMap<_, _>>(),
                )
            })
            .collect::<Vec<_>>();

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
    ) -> anyhow::Result<HashMap<Bar, Vec<&Dumbbell>>, GymError> {
        if requirements.is_empty() {
            return Ok(HashMap::new());
        }

        // Find all states that satisfy each requirement
        let requirement_states: Vec<Vec<GymStateId>> = requirements
            .iter()
            .map(|req| self.find_states_for_requirement(*req))
            .collect::<Result<Vec<_>, _>>()?;

        // Compute shortest distances between all state pairs
        let shortest_distances: HashMap<(NodeIndex, NodeIndex), u32> =
            algo::johnson(&self.graph, |e| *e.weight())
                .map_err(|_| GymError::InvalidRequirement(requirements[0]))?;

        // Find optimal path through requirement states using dynamic programming
        let start_states = requirement_states
            .first()
            .ok_or(GymError::InvalidRequirement(requirements[0]))?;

        let optimal_sequence = start_states.iter()
            .map(|start_state| {
                self.find_optimal_sequence(*start_state, &requirement_states, &shortest_distances)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .min_by_key(Vec::len)
            .ok_or(GymError::InvalidRequirement(requirements[0]))?;

        // Convert the complete path to the expected result format
        let mut result = HashMap::<Bar, Vec<&Dumbbell>>::new();
        let mut requirement_index = 0;

        for state_id in optimal_sequence {
            let state = &self.states[state_id.0];
            let bars = self.bar_options.get(&requirements[requirement_index].bar_kind())
                .ok_or(GymError::InvalidRequirement(requirements[requirement_index]))?;

            for bar in bars {
                if let Some(dumbbell) = state.get(bar) {
                    if dumbbell.weight() == requirements[requirement_index].weight() {
                        result.entry(*bar).or_default().push(dumbbell);
                    }
                }
            }

            // Move to the next requirement if we have satisfied the current one
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

    /// Find all gym states that satisfy a given requirement
    /// Returns states sorted by a consistent criteria to ensure deterministic behavior
    fn find_states_for_requirement(
        &self,
        requirement: Requirement,
    ) -> Result<Vec<GymStateId>, GymError> {
        let mut matching_states: Vec<(GymStateId, u32)> = self
            .states
            .iter()
            .enumerate()
            .filter_map(|(i, state)| {
                // Check if any bar of the required kind has a dumbbell with the required weight
                let bars = self.bar_options.get(&requirement.bar_kind())?;

                for bar in bars {
                    if let Some(dumbbell) = state.get(bar) {
                        if dumbbell.weight() == requirement.weight() {
                            // Calculate a "complexity score" for consistent ordering
                            // States with fewer total plates are preferred (simpler states)
                            let total_plates: u32 = state.value().values()
                                .map(|d| d.plates().len() as u32)
                                .sum();
                            return Some((GymStateId(i), total_plates));
                        }
                    }
                }
                None
            })
            .collect();

        if matching_states.is_empty() {
            Err(GymError::InvalidRequirement(requirement))
        } else {
            Ok(matching_states.into_iter().map(|(id, _)| id).collect())
        }
    }

    /// Find optimal sequence through requirement states using dynamic programming
    fn find_optimal_sequence(
        &self,
        start_state: GymStateId,
        requirement_states: &[Vec<GymStateId>],
        distances: &HashMap<(NodeIndex, NodeIndex), u32>,
    ) -> Result<Vec<GymStateId>, GymError> {
        let n = requirement_states.len();
        if n == 0 {
            return Ok(vec![]);
        }
        if n == 1 {
            return Ok(vec![requirement_states[0][0]]);
        }

        let start_node = self.nodes[&start_state];

        // Dynamic programming: dp[i][state] = minimum cost to reach requirement i ending at state
        let mut dp: Vec<HashMap<GymStateId, (u32, Option<GymStateId>)>> = vec![HashMap::new(); n];

        // Initialize first requirement
        for &state in &requirement_states[0] {
            let state_node = self.nodes[&state];
            if let Some(&cost) = distances.get(&(start_node, state_node)) {
                dp[0].insert(state, (cost, None));
            }
        }

        // Fill DP table
        for i in 1..n {
            for &current_state in &requirement_states[i] {
                let current_node = self.nodes[&current_state];
                let mut min_cost = u32::MAX;
                let mut best_prev = None;

                // Try all states from previous requirement
                for (&prev_state, &(prev_cost, _)) in &dp[i - 1] {
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
        // Find optimal final state
        let (&final_state, _) = dp[n - 1]
            .iter()
            .min_by_key(|(_, (cost, _))| *cost)
            .ok_or(GymError::InvalidRequirement(Requirement::new(0, BarKind::Dumbbell)))?;

        // Reconstruct path
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

    /// Find the actual shortest path between two specific states
    fn find_shortest_path_between_states(
        &self,
        start: GymStateId,
        end: GymStateId
    ) -> std::vec::Vec<GymStateId> {
        if start == end {
            return vec![];
        }

        let start_node = self.nodes[&start];
        let end_node = self.nodes[&end];

        // Use A* to find the actual path (not just the distance)
        if let Some((_, path)) = algo::astar(
            &self.graph,
            start_node,
            |n| n == end_node,
            |e| *e.weight(),
            |_| 0, // No heuristic needed since all edges have weight 1
        ) {
            // Convert node path back to state IDs
            let state_path: Vec<GymStateId> = path
                .into_iter()
                .filter_map(|node| {
                    // Find the state ID for this node
                    self.nodes.iter()
                        .find(|(_, n)| **n == node)
                        .map(|(&state_id, _)| state_id)
                })
                .collect();

            state_path
        } else {
            // Fallback: if no path found, just return the start and end states
            vec![start, end]
        }
    }
}
