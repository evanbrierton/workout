use std::collections::HashMap;

use crate::{bar::Bar, dumbbell::Dumbbell};

#[derive(Default)]
pub struct Workout {
    dumbbells: HashMap<Bar, Vec<Dumbbell>>,
}

impl Workout {
    #[must_use]
    pub fn new(dumbbells: HashMap<Bar, Vec<Dumbbell>>) -> Self {
        Workout { dumbbells }
    }

    #[must_use]
    pub fn get(&self, bar: Bar) -> Vec<Dumbbell> {
        self.dumbbells.get(&bar).cloned().unwrap_or_default()
    }

    #[must_use]
    pub fn bars(&self) -> Vec<Bar> {
        self.dumbbells.keys().copied().collect()
    }
}

impl IntoIterator for Workout {
    type Item = (Bar, Vec<Dumbbell>);
    type IntoIter = std::collections::hash_map::IntoIter<Bar, Vec<Dumbbell>>;

    fn into_iter(self) -> Self::IntoIter {
        self.dumbbells.into_iter()
    }
}
