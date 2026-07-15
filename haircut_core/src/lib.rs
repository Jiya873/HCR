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

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WebSimulator {
    engine: Simulation,
    hair_positions: Vec<f32>,
    hair_lengths: Vec<f32>,
    debris_positions: Vec<f32>,
    debris_lengths: Vec<usize>,
}

#[wasm_bindgen]
impl WebSimulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let config = SimulationConfig::default();
        let mut engine = Simulation::new(config).expect("Failed to start engine");
        let _ = engine.apply_command(RuntimeCommand::Clipper(ClipperCommand::ActivateCutting));
        Self { 
            engine,
            hair_positions: Vec::new(),
            hair_lengths: Vec::new(),
            debris_positions: Vec::new(),
            debris_lengths: Vec::new(),
        }
    }

    pub fn step(&mut self) {
        self.engine.step();
        self.update_buffers();
    }

    fn update_buffers(&mut self) {
        self.hair_positions.clear();
        self.hair_lengths.clear();
        for strand in self.engine.strands() {
            self.hair_lengths.push(strand.active_len as f32);
            for node in &strand.nodes {
                self.hair_positions.push(node.position.x);
                self.hair_positions.push(node.position.y);
                self.hair_positions.push(node.position.z);
            }
        }

        self.debris_positions.clear();
        self.debris_lengths.clear();
        for segment in self.engine.debris() {
            self.debris_lengths.push(segment.points.len());
            for point in &segment.points {
                self.debris_positions.push(point.position.x);
                self.debris_positions.push(point.position.y);
                self.debris_positions.push(point.position.z);
            }
        }
    }

    pub fn get_hair_positions_ptr(&self) -> *const f32 {
        self.hair_positions.as_ptr()
    }

    pub fn get_hair_positions_len(&self) -> usize {
        self.hair_positions.len()
    }

    pub fn get_hair_lengths_ptr(&self) -> *const f32 {
        self.hair_lengths.as_ptr()
    }

    pub fn get_clipper_position(&self) -> Vec<f32> {
        let clipper = self.engine.clipper();
        vec![clipper.actual_pos.x, clipper.actual_pos.y, clipper.actual_pos.z]
    }

    pub fn update_clipper(&mut self, x: f32, y: f32, z: f32) {
        let cmd = RuntimeCommand::Clipper(ClipperCommand::SetTargetXyz { x, y, z });
        let _ = self.engine.apply_command(cmd);
    }

    pub fn set_cutting(&mut self, enabled: bool) {
        let cmd = if enabled {
            ClipperCommand::ActivateCutting
        } else {
            ClipperCommand::DeactivateCutting
        };
        let _ = self.engine.apply_command(RuntimeCommand::Clipper(cmd));
    }

    pub fn get_debris_positions_ptr(&self) -> *const f32 {
        self.debris_positions.as_ptr()
    }

    pub fn get_debris_lengths_ptr(&self) -> *const usize {
        self.debris_lengths.as_ptr()
    }
    
    pub fn get_debris_count(&self) -> usize {
        self.debris_lengths.len()
    }
}