//! Arrow-free, deterministic infrastructure for the R/S/P research consumers.
//! Every estimate carries its method assumptions and requires explicit bounds.
pub mod confirmation;
pub mod knowledge;
pub mod population;
pub mod portfolio;
pub mod provenance;
pub mod question_templates;
pub mod questions;
pub mod reporting;
pub mod research;
pub mod role;
pub mod simulation;
pub mod stats;
pub mod strategy;
pub mod walk_forward;

pub use confirmation::*;
pub use knowledge::*;
pub use population::*;
pub use portfolio::*;
pub use provenance::*;
pub use question_templates::*;
pub use questions::*;
pub use reporting::*;
pub use research::*;
pub use role::*;
pub use simulation::*;
pub use stats::*;
pub use strategy::*;
pub use walk_forward::*;
