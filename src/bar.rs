use std::fmt::Display;

use crate::bar_kind::BarKind;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Bar {
    pub weight: u32,
    pub gauge: u32,
    pub kind: BarKind,
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
}

impl Display for Bar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.kind, self.gauge)
    }
}
