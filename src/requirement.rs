use std::{fmt::Display, str::FromStr};

use crate::{bar_kind::BarKind, dumbbell::Dumbbell};

#[derive(Debug, Clone, Copy)]
pub struct Requirement {
    weight: u32,
    bar_kind: BarKind,
}

impl Requirement {
    #[must_use]
    pub fn new(weight: u32, bar_kind: BarKind) -> Self {
        Requirement { weight, bar_kind }
    }

    #[must_use]
    pub fn matches(&self, dumbbell: &Dumbbell) -> bool {
        self.weight == dumbbell.weight() && self.bar_kind == *dumbbell.bar().kind()
    }

    #[must_use]
    pub fn bar_kind(&self) -> BarKind {
        self.bar_kind
    }

    #[must_use]
    pub fn weight(&self) -> u32 {
        self.weight
    }
}

impl FromStr for Requirement {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (weight, bar_kind) = s.split_at(s.len() - 1);
        let weight = weight
            .parse::<f64>()
            .map_err(|_| "Invalid weight".to_string())?;
        let bar_kind = BarKind::from_str(bar_kind.to_lowercase().as_str())?;

        Ok(Requirement {
            weight: kgs_to_grams(weight),
            bar_kind,
        })
    }
}

impl Display for Requirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}kg {}", f64::from(self.weight) / 1000.0, self.bar_kind)
    }
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
fn kgs_to_grams(kgs: f64) -> u32 {
    (kgs * 1000.0) as u32
}
