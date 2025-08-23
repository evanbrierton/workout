use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use petgraph::{algo, prelude::UnGraphMap};

use crate::{
    bar::Bar,
    bar_kind::BarKind,
    dumbbell::Dumbbell,
    gym_error::GymError,
    gym_state::{GymState, GymStateId},
    plate::Plate,
    requirement::Requirement,
    workout::Workout,
};

pub struct Gym {
    states: HashMap<BarKind, Vec<GymState>>,
    graphs: HashMap<BarKind, UnGraphMap<GymStateId, u32>>,
    weights: HashMap<Bar, Vec<u32>>,
    bar_options: HashMap<BarKind, Vec<Bar>>,
}

impl Gym {
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(plates: Vec<Plate>, bars: Vec<Bar>) -> Self {
        let plate_counts: HashMap<Plate, usize> =
            plates.iter().fold(HashMap::new(), |mut acc, plate| {
                *acc.entry(*plate).or_default() += 1;
                acc
            });

        let dumbbells: HashMap<Bar, Vec<Dumbbell>> = bars
            .iter()
            .map(|bar| (*bar, Self::dumbbells(&plate_counts, bar)))
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

        let dumbbells: HashMap<BarKind, HashMap<Bar, Vec<Dumbbell>>> =
            dumbbells
                .into_iter()
                .fold(HashMap::new(), |mut acc, (bar, dumbbells)| {
                    acc.entry(*bar.kind()).or_default().insert(bar, dumbbells);
                    acc
                });

        let states: HashMap<BarKind, Vec<GymState>> = dumbbells
            .into_iter()
            .map(|(kind, dumbbells)| {
                let states = dumbbells
                    .into_values()
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
                (kind, states)
            })
            .collect();

        let ids = states
            .iter()
            .enumerate()
            .map(|(i, _)| GymStateId(i))
            .collect::<HashSet<_>>();

        let graphs = states
            .iter()
            .map(|(kind, states)| {
                let graph = Self::graph(states, &ids);
                (*kind, graph)
            })
            .collect();

        let bar_options: HashMap<BarKind, Vec<Bar>> =
            bars.iter().fold(HashMap::new(), |mut acc, bar| {
                acc.entry(*bar.kind()).or_default().push(*bar);
                acc
            });

        Gym {
            states,
            graphs,
            weights,
            bar_options,
        }
    }

    #[must_use]
    pub fn weights(&self) -> &HashMap<Bar, Vec<u32>> {
        &self.weights
    }

    ///
    /// # Errors
    /// If it is impossible to construct a dumbbell for a requirement given the user's plates.
    ///
    #[allow(clippy::needless_pass_by_value)]
    pub fn workout(&self, requirements: Vec<Requirement>) -> Result<Workout, GymError> {
        let requirements_by_kind: HashMap<BarKind, Vec<Requirement>> =
            requirements.iter().fold(HashMap::new(), |mut acc, req| {
                acc.entry(req.bar_kind()).or_default().push(*req);
                acc
            });

        let mut result = HashMap::<Bar, Vec<Dumbbell>>::new();

        for (bar_kind, reqs) in requirements_by_kind {
            let ordered_dumbbells = self.order_by_kind(bar_kind, &reqs)?;
            for (bar, dumbbells) in ordered_dumbbells {
                result
                    .entry(bar)
                    .or_default()
                    .extend(dumbbells.into_iter().cloned().collect::<Vec<_>>());
            }
        }

        Ok(Workout::new(result))
    }

    ///
    /// # Errors
    /// If it is impossible to construct a dumbbell for a requirement given the user's plates.
    ///
    fn order_by_kind(
        &self,
        bar_kind: BarKind,
        requirements: &[Requirement],
    ) -> Result<HashMap<Bar, Vec<&Dumbbell>>, GymError> {
        if requirements.is_empty() {
            return Ok(HashMap::new());
        }

        let optimal_sequence = self.find_optimal_sequence(bar_kind, requirements)?;

        let mut result = HashMap::<Bar, Vec<&Dumbbell>>::new();
        let mut requirement_index = 0;

        for state_id in optimal_sequence {
            let state = &self.states[&bar_kind][state_id.0];
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

    fn graph(states: &[GymState], ids: &HashSet<GymStateId>) -> UnGraphMap<GymStateId, u32> {
        let mut graph = UnGraphMap::<GymStateId, u32>::new();

        for id in ids {
            graph.add_node(*id);
        }

        for ((i1, state1), (i2, state2)) in states.iter().enumerate().tuple_combinations() {
            if state1.adjacent(state2) {
                graph.add_edge(GymStateId(i1), GymStateId(i2), 1);
            }
        }

        graph
    }

    fn find_states_for_requirement(&self, requirement: Requirement) -> Vec<GymStateId> {
        let matching_states: Vec<(GymStateId, usize)> = self.states[&requirement.bar_kind()]
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
        bar_kind: BarKind,
        requirements: &[Requirement],
    ) -> Result<Vec<GymStateId>, GymError> {
        let requirement_states: Vec<Vec<GymStateId>> = requirements
            .iter()
            .map(|req| self.find_states_for_requirement(*req))
            .collect();

        let n = requirement_states.len();

        match n {
            0 => return Ok(vec![]),
            1 => {
                return requirement_states[0]
                    .iter()
                    .min_by_key(|id| self.states[&bar_kind][id.0].clone())
                    .ok_or(GymError::InvalidRequirement(requirements[0]))
                    .map(|id| vec![*id]);
            }
            _ => {}
        }

        let distances: HashMap<(GymStateId, GymStateId), u32> =
            algo::johnson(&self.graphs[&bar_kind], |e| *e.2)
                .map_err(|_| GymError::InvalidRequirement(requirements[0]))?
                .into_iter()
                .collect();

        let mut dp: Vec<HashMap<GymStateId, (u32, Option<GymStateId>)>> = vec![HashMap::new(); n];

        for &state in &requirement_states[0] {
            dp[0].insert(state, (0, None));
        }

        for i in 1..n {
            for &current_state in &requirement_states[i] {
                let mut min_cost = u32::MAX;
                let mut best_prev = None;

                let mut prev_states: Vec<_> = dp[i - 1]
                    .iter()
                    .map(|(&state, &(cost, _))| (state, cost))
                    .collect();
                prev_states.sort_by_key(|&(state, _)| state);

                for (prev_state, prev_cost) in prev_states {
                    if let Some(&transition_cost) = distances.get(&(prev_state, current_state)) {
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
