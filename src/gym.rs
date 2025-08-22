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
        let optimal_sequence =
            self.find_optimal_sequence(&requirement_states, &shortest_distances)?;

        // Get the complete path with all intermediate transitions
        let complete_path = self.get_complete_transition_path(&optimal_sequence, &shortest_distances);

        // Convert the complete path to the expected result format
        let mut result = HashMap::new();
        let mut requirement_index = 0;

        for (path_index, &state_id) in complete_path.iter().enumerate() {
            let state = &self.states[state_id.0];

            if path_index == 0 {
                // Starting position - add empty dumbbells
                for (bar, dumbbell) in state.value() {
                    if dumbbell.plates().is_empty() {
                        result.entry(*bar).or_insert_with(Vec::new).push(dumbbell);
                    }
                }
            } else {
                // Check if this state satisfies the next requirement
                if requirement_index < requirements.len() {
                    let requirement = requirements[requirement_index];
                    let bars = &self.bar_options[&requirement.bar_kind()];

                    for bar in bars {
                        if let Some(dumbbell) = state.get(bar) {
                            if dumbbell.weight() == requirement.weight() {
                                result.entry(*bar).or_insert_with(Vec::new).push(dumbbell);
                                requirement_index += 1;
                                break;
                            }
                        }
                    }
                }
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
            // Sort by complexity (fewer plates first) then by state ID for determinism
            matching_states.sort_by(|(id1, plates1), (id2, plates2)| {
                plates1.cmp(plates2).then_with(|| id1.0.cmp(&id2.0))
            });

            Ok(matching_states.into_iter().map(|(id, _)| id).collect())
        }
    }

    /// Find optimal sequence through requirement states using dynamic programming
    fn find_optimal_sequence(
        &self,
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

        // Start from the initial state (empty dumbbells)
        let start_state = GymStateId(0); // Assuming first state is the empty state
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

    /// Get the complete path including all intermediate transitions
    /// This ensures we don't skip any necessary plate changes
    fn get_complete_transition_path(
        &self,
        optimal_sequence: &[GymStateId],
        distances: &HashMap<(NodeIndex, NodeIndex), u32>,
    ) -> std::vec::Vec<GymStateId> {
        if optimal_sequence.is_empty() {
            return vec![GymStateId(0)]; // Just return the starting state
        }

        let mut complete_path = vec![GymStateId(0)]; // Start with empty state

        for &target_state in optimal_sequence {
            let current_state = *complete_path.last().unwrap();

            if current_state != target_state {
                // Find the actual shortest path between current and target states
                let intermediate_path = self.find_shortest_path_between_states(
                    current_state,
                    target_state,
                    distances
                );

                // Add intermediate states (skip the first one as it's already in complete_path)
                complete_path.extend(intermediate_path.into_iter().skip(1));
            }
        }

        complete_path
    }

    /// Find the actual shortest path between two specific states
    fn find_shortest_path_between_states(
        &self,
        start: GymStateId,
        end: GymStateId,
        distances: &HashMap<(NodeIndex, NodeIndex), u32>,
    ) -> std::vec::Vec<GymStateId> {
        if start == end {
            return vec![start];
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
