use std::fmt::Display;

use crate::bar_kind::BarKind;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd)]
pub struct Bar {
    weight: u32,
    gauge: u32,
    kind: BarKind,
}

impl Bar {
    pub fn new(weight: u32, gauge: u32, kind: BarKind) -> Self {
        Bar {
            weight,
            gauge,
            kind,
        }
    }

    pub fn weight(&self) -> u32 {
        self.weight
    }

    pub fn gauge(&self) -> u32 {
        self.gauge
    }

    pub fn kind(&self) -> &BarKind {
        &self.kind
    }
}

impl Display for Bar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.kind, self.gauge)
    }
}

impl Ord for Bar {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.weight
            .cmp(&other.weight)
            .then_with(|| self.gauge.cmp(&other.gauge))
            .then_with(|| self.kind.cmp(&other.kind))
    }
}
