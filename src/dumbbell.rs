use std::{collections::HashMap, fmt::Display};

use itertools::Itertools;

use crate::{
    bar::{Bar, BarType},
    plate::Plate,
};

#[derive(Clone, Debug)]

pub struct Dumbbell {
    pub plates: Vec<Plate>,
    pub bar: Bar,
}

impl Dumbbell {
    fn new(plates: Vec<Plate>, bar: Bar) -> Self {
        Dumbbell {
            plates: plates
                .into_iter()
                .sorted()
                .filter(|p| p.gauge == bar.gauge)
                .collect(),
            bar,
        }
    }

    pub fn weight(&self) -> u32 {
        self.bar.weight + self.plates.clone().into_iter().sum::<Plate>().weight * 2
    }

    pub fn available(plates: Vec<Plate>, bar: Bar) -> Vec<Dumbbell> {
        plates
            .iter()
            .powerset()
            .into_iter()
            .map(|plates| Dumbbell::new(plates.into_iter().copied().collect(), bar.clone()))
            .collect()
    }

    pub fn available_from_weight_map(
        weights_map: HashMap<Plate, usize>,
        bar: Bar,
    ) -> Vec<Dumbbell> {
        match bar.bar_type {
            BarType::Dumbbell => Dumbbell::available(
                weights_map
                    .into_iter()
                    .filter(|(_, count)| *count >= 4)
                    .map(|(plate, count)| (plate, count / 4))
                    .flat_map(|(plate, count)| vec![plate; count])
                    .collect(),
                bar,
            ),
            BarType::Barbell => {
                Dumbbell::available(
                    weights_map
                        .into_iter()
                        .filter(|(_, count)| *count >= 2)
                        .map(|(plate, count)| (plate, count / 2))
                        .flat_map(|(plate, count)| vec![plate; count])
                        .collect(),
                    bar,
                )
            },
        }
    }

    pub fn sort(dumbbells: Vec<Dumbbell>) -> Vec<Dumbbell> {
        dumbbells
            .into_iter()
            .sorted_by(|a, b| {
                a.bar.bar_type
                    .cmp(&b.bar.bar_type)
                    .then_with(|| a.weight().cmp(&b.weight()))
                    .then_with(|| a.plates.len().cmp(&b.plates.len()))
            })
            .dedup_by(|a, b| a.weight() == b.weight() && a.bar.bar_type == b.bar.bar_type)
            .collect::<Vec<_>>()
    }
}

impl Display for Dumbbell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kg_plates = self
            .plates
            .iter()
            .map(|p| p.weight as f64 / 1000.0)
            .rev()
            .collect::<Vec<_>>();

        write!(f, "{} ({}) {:?} ({}kg)", self.bar.bar_type, self.bar.gauge, kg_plates, self.weight() as f64 / 1000.0,)
    }
}
