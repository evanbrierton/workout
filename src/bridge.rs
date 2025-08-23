use crate::bar::Bar;
use crate::bar_kind::BarKind;
use crate::gym::Gym;
use crate::gym_error::GymError;
use crate::plate::Plate;
use crate::requirement::Requirement;
use crate::workout::Workout;
use crate::dumbbell::Dumbbell;

#[swift_bridge::bridge]
mod ffi {
    extern "Rust" {
        type Gym;
        type Plate;
        type Bar;
        type Requirement;
        type BarKind;
        type Workout;
        type Dumbbell;
        type GymError;

        #[swift_bridge(init)]
        fn new(plates: Vec<Plate>, bars: Vec<Bar>) -> Gym;
        fn workout(self: &Gym, requirements: Vec<Requirement>) -> Result<Workout, GymError>;

        #[swift_bridge(init)]
        fn new(weight: u32, count: u32) -> Plate;

        #[swift_bridge(init)]
        fn new(weight: u32, gauge: u32, kind: BarKind) -> Bar;

        #[swift_bridge(init)]
        fn new(weight: u32, kind: BarKind) -> Requirement;

        #[swift_bridge(associated_to = BarKind)]
        fn dumbbell() -> BarKind;
        #[swift_bridge(associated_to = BarKind)]
        fn barbell() -> BarKind;

        fn bars(self: &Workout) -> Vec<Bar>;
        fn get(self: &Workout, bar: Bar) -> Vec<Dumbbell>;

        fn plates_owned(self: Dumbbell) -> Vec<Plate>;
    }
}
