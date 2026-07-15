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
}

#[wasm_bindgen]
impl WebSimulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let config = SimulationConfig::default();
        let mut engine = Simulation::new(config).expect("Failed to start engine");
        let _ = engine.apply_command(RuntimeCommand::Clipper(ClipperCommand::ActivateCutting));
        Self { engine }
    }

    pub fn step(&mut self) {
        self.engine.step();
    }

    pub fn get_hair_lengths(&self) -> Vec<f32> {
        self.engine
            .strands()
            .iter()
            .map(|strand| strand.active_len as f32)
            .collect()
    }

    pub fn get_hair_positions(&self) -> Vec<f32> {
        let mut positions = Vec::new();
        
        for strand in self.engine.strands() {
            for node in &strand.nodes {
                positions.push(node.position.x);
                positions.push(node.position.y);
                positions.push(node.position.z);
            }
        }
        
        positions
    }

    pub fn get_clipper_position(&self) -> Vec<f32> {
        let clipper = self.engine.clipper();
        
        let pos = clipper.actual_pos; 
        vec![pos.x, pos.y, pos.z]
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

    pub fn get_debris_positions(&self) -> Vec<f32> {
        let mut positions = Vec::new();
        
        for segment in self.engine.debris() {
            for point in &segment.points {
                positions.push(point.position.x);
                positions.push(point.position.y);
                positions.push(point.position.z);
            }
        }
        
        positions
    }
    
    pub fn get_debris_count(&self) -> usize {
        self.engine.debris().len()
    }

    pub fn get_debris_lengths(&self) -> Vec<usize> {
        self.engine
            .debris()
            .iter()
            .map(|segment| segment.points.len())
            .collect()
    }
}