use std::str::FromStr;

use crate::{bar_kind::BarKind, dumbbell::Dumbbell};

#[derive(Debug, Clone)]
pub struct Requirement {
    pub weight: u32,
    pub bar_kind: BarKind,
}

impl Requirement {
    pub fn matches(&self, dumbbell: &Dumbbell) -> bool {
        self.weight == dumbbell.weight() && self.bar_kind == dumbbell.bar.kind
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
            weight: (weight * 1000.0) as u32,
            bar_kind,
        })
    }
}
