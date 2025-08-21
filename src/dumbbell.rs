use std::{
    fmt::Display,
    hash::Hash, rc::Rc,
};

use itertools::Itertools;

use crate::{bar::Bar, plate::Plate};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]

pub struct Dumbbell {
    plates: Vec<Plate>,
    bar: Bar,
}

impl Dumbbell {
    pub fn new(plates: Vec<Plate>, bar: &Bar) -> Rc<Self> {
        Rc::new(
        Dumbbell {
            plates: plates
                .into_iter()
                .sorted()
                .rev()
                .filter(|p| p.gauge() == bar.gauge())
                .collect(),
            bar: *bar,
        })
    }

    pub fn plates(&self) -> &[Plate] {
        &self.plates
    }

    pub fn bar(&self) -> &Bar {
        &self.bar
    }

    pub fn weight(&self) -> u32 {
        self.bar.weight() + self.plates.iter().map(|p| p.weight()).sum::<u32>() * 2
    }

}

impl Display for Dumbbell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kg_plates = self
            .plates
            .iter()
            .map(|p| p.weight() as f64 / 1000.0)
            .collect::<Vec<_>>();

        write!(
            f,
            "{} ({}) {:?} ({}kg)",
            self.bar.kind(),
            self.bar.gauge(),
            kg_plates,
            self.weight() as f64 / 1000.0,
        )
    }
}
