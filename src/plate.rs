use std::{collections::HashMap, iter::repeat_n};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Plate {
    weight: u32,
    gauge: u32,
}

impl Plate {
    #[must_use]
    pub fn new(weight: u32, gauge: u32) -> Self {
        Plate { weight, gauge }
    }

    #[must_use]
    pub fn weight(&self) -> u32 {
        self.weight
    }

    #[must_use]
    pub fn gauge(&self) -> u32 {
        self.gauge
    }

    #[must_use]
    pub fn from_weights_map(weights_map: HashMap<u32, usize>, gauge: u32) -> Vec<Plate> {
        weights_map
            .into_iter()
            .map(|(weight, count)| (Plate::new(weight, gauge), count))
            .flat_map(|(plate, count)| repeat_n(plate, count))
            .collect()
    }
}
