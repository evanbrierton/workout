use std::{fmt::Display, hash::Hash, rc::Rc};

use itertools::Itertools;

use crate::{bar::Bar, plate::Plate};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]

pub struct Dumbbell {
    plates: Vec<Plate>,
    bar: Bar,
}

impl Dumbbell {
    #[must_use]
    pub fn new(plates: Vec<Plate>, bar: Bar) -> Self {
        Dumbbell {
            plates: plates
                .into_iter()
                .sorted()
                .rev()
                .filter(|p| p.gauge() == bar.gauge())
                .collect(),
            bar,
        }
    }

    #[must_use]
    pub fn new_rc(plates: Vec<Plate>, bar: &Bar) -> Rc<Self> {
        Rc::new(Dumbbell::new(plates, *bar))
    }

    #[must_use]
    pub fn plates(&self) -> &[Plate] {
        &self.plates
    }

    #[must_use]
    pub fn bar(&self) -> &Bar {
        &self.bar
    }

    #[must_use]
    pub fn weight(&self) -> u32 {
        self.bar.weight() + self.plates.iter().map(Plate::weight).sum::<u32>() * 2
    }

    #[must_use]
    pub fn adjacent(&self, other: &Self) -> bool {
        if self.bar != other.bar {
            return false;
        }

        if self.plates.len().abs_diff(other.plates.len()) != 1 {
            return false;
        }

        self.plates
            .iter()
            .zip(&other.plates)
            .all(|(p1, p2)| p1.weight() == p2.weight())
    }
}

impl PartialOrd for Dumbbell {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Dumbbell {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.weight().cmp(&other.weight())
    }
}

impl Display for Dumbbell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kg_plates = self
            .plates
            .iter()
            .map(|p| f64::from(p.weight()) / 1000.0)
            .collect::<Vec<_>>();

        write!(
            f,
            "{:?} ({}kg)",
            kg_plates,
            f64::from(self.weight()) / 1000.0,
        )
    }
}
