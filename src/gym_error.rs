use thiserror::Error;

use crate::requirement::Requirement;

#[derive(Error, Debug)]
pub enum GymError {
    #[error("Cannot construct {0} with available plates and bars.")]
    InvalidRequirement(Requirement),
}
