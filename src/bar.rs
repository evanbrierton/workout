use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq)]
pub enum BarType {
  Dumbbell,
  Barbell
}

#[derive(Clone, Debug)]
pub struct Bar {
  pub weight: u32,
  pub gauge: u32,
  pub bar_type: BarType,
}

impl Bar {
  pub fn new(weight: u32, gauge: u32, bar_type: BarType) -> Self {
    Bar { weight, gauge, bar_type }
  }

  pub fn weight(&self) -> u32 {
    self.weight
  }

  pub fn gauge(&self) -> u32 {
    self.gauge
  }
}

impl Into<usize> for &BarType {
  fn into(self) -> usize {
    match self {
      BarType::Dumbbell => 1,
      BarType::Barbell => 2,
    }
  }
}

impl Ord for BarType {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    let s: usize = self.into();
    let other: usize = other.into();

    s.cmp(&other)
  }
}

impl Display for BarType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      BarType::Dumbbell => write!(f, "Dumbbell"),
      BarType::Barbell => write!(f, "Barbell"),
    }
  }
}
