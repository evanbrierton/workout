use std::fmt::Display;

use hashbrown::HashMap;

use crate::{bar::Bar, dumbbell::Dumbbell};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GymStateId(pub usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GymState {
    state: HashMap<Bar, Dumbbell>,
}

impl GymState {
    #[must_use]
    pub fn new(state: HashMap<Bar, Dumbbell>) -> Self {
        GymState { state }
    }

    #[must_use]
    pub fn adjacent(&self, other: &Self) -> bool {
        let mut adjacencies = 0;

        for (bar, dumbbell) in &self.state {
            if let Some(other_dumbbell) = other.get(bar) {
                let same = dumbbell == other_dumbbell;
                let adjacent = dumbbell.adjacent(other_dumbbell);

                if !(same || adjacent) {
                    return false;
                }

                if adjacent {
                    adjacencies += 1;
                }

                if adjacencies > 1 {
                    return false;
                }
            }
        }

        adjacencies == 1
    }

    #[must_use]
    pub fn value(&self) -> &HashMap<Bar, Dumbbell> {
        &self.state
    }

    #[must_use]
    pub fn get(&self, bar: &Bar) -> Option<&Dumbbell> {
        self.state.get(bar)
    }

    #[must_use]
    pub fn plates(&self) -> usize {
        self.state
            .values()
            .map(|dumbbell| dumbbell.plates().len())
            .sum()
    }
}

impl Display for GymState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (bar, dumbbell) in &self.state {
            writeln!(f, "{bar}: {dumbbell}")?;
        }
        Ok(())
    }
}

impl Ord for GymState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.plates().cmp(&other.plates())
    }
}

impl PartialOrd for GymState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
