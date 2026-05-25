pub mod clipper;
pub mod commands;
pub mod config;
pub mod debris;
pub mod geometry;
pub mod hair;
pub mod init;
pub mod math;
pub mod physics;
pub mod simulation;

pub use clipper::{ClipperCommand, ClipperState};
pub use commands::RuntimeCommand;
pub use config::{HeadConfig, SimulationConfig, TuningConfig, WorldBounds};
pub use debris::DebrisSegment;
pub use geometry::Ellipsoid;
pub use hair::{HairNode, HairRoot, HairStrand};
pub use math::Vec3;
pub use simulation::{Simulation, SimulationSnapshot, StepSummary};

#[cfg(test)]
mod tests;
