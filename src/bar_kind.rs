use std::{fmt::Display, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum BarKind {
    Dumbbell,
    Barbell,
}

impl BarKind {
    pub fn required_similar_plates(&self) -> usize {
        match self {
            BarKind::Dumbbell => 4,
            BarKind::Barbell => 2,
        }
    }
}

impl Display for BarKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BarKind::Dumbbell => write!(f, "Dumbbell"),
            BarKind::Barbell => write!(f, "Barbell"),
        }
    }
}

impl FromStr for BarKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "d" => Ok(BarKind::Dumbbell),
            "b" => Ok(BarKind::Barbell),
            _ => Err("Invalid bar kind.".to_string()),
        }
    }
}
