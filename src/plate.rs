use std::{collections::HashMap, iter::Sum};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Plate {
    weight: u32,
    gauge: u32,
}

impl Plate {
    pub fn new(weight: u32, gauge: u32) -> Self {
        Plate { weight, gauge }
    }

    pub fn weight(&self) -> u32 {
        self.weight
    }

    pub fn gauge(&self) -> u32 {
        self.gauge
    }

    pub fn from_weights(weights: Vec<u32>, gauge: u32) -> Vec<Plate> {
        weights.into_iter().map(|w| Plate::new(w, gauge)).collect()
    }

    pub fn from_weights_map(weights_map: HashMap<u32, usize>, gauge: u32) -> HashMap<Plate, usize> {
        weights_map
            .into_iter()
            .map(|(weight, count)| (Plate::new(weight, gauge), count))
            .collect()
    }
}

impl Sum for Plate {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Plate::new(0, 0), |acc, plate| {
            Plate::new(acc.weight + plate.weight, acc.gauge)
        })
    }
}
