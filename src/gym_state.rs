use std::collections::HashMap;

use crate::{bar::Bar, dumbbell::Dumbbell};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GymStateId(pub usize);

pub struct GymState {
    state: HashMap<Bar, Dumbbell>,
}

impl GymState {
    pub fn new(state: HashMap<Bar, Dumbbell>) -> Self {
        GymState { state }
    }

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

        false
    }

    pub fn value(&self) -> &HashMap<Bar, Dumbbell> {
        &self.state
    }

    pub fn get(&self, bar: &Bar) -> Option<&Dumbbell> {
        self.state.get(bar)
    }

    pub fn insert(&mut self, bar: Bar, dumbbell: Dumbbell) {
        self.state.insert(bar, dumbbell);
    }

    pub fn remove(&mut self, bar: &Bar) -> Option<Dumbbell> {
        self.state.remove(bar)
    }

    pub fn contains_key(&self, bar: &Bar) -> bool {
        self.state.contains_key(bar)
    }
}
