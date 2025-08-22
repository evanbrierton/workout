use hashbrown::HashMap;
use petgraph::adj;

use crate::{bar::Bar, dumbbell::Dumbbell};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GymStateId(pub usize);

#[derive(Debug)]
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
                if dumbbell.adjacent(other_dumbbell) {
                    adjacencies += 1;
                    if adjacencies > 1 {
                        return false;
                    }
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
}
