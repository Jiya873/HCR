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
use physics::GpuPhysics;

#[wasm_bindgen]
pub struct WebSimulator {
    engine: Simulation,
    hair_positions: Vec<f32>,
    hair_lengths: Vec<f32>,
    debris_positions: Vec<f32>,
    debris_lengths: Vec<usize>,

    #[wasm_bindgen(skip)]
    pub gpu_physics: Option<GpuPhysics>,
    #[wasm_bindgen(skip)]
    pub gpu_strands_buffer: Option<wgpu::Buffer>,
    #[wasm_bindgen(skip)]
    pub gpu_bind_group: Option<wgpu::BindGroup>,
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

            gpu_physics: None,
            gpu_strands_buffer: None,
            gpu_bind_group: None,
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

    pub fn sync_server_lengths(&mut self, server_lengths: &[f32]) {
        self.engine.sync_server_lengths(server_lengths);
    }

    pub async fn step_async(&mut self) -> Result<(), JsValue> {
        self.engine.update_kinematics_and_debris(); 
        
        if let (Some(gpu), Some(buffer), Some(bind_group)) = (
            &self.gpu_physics, 
            &self.gpu_strands_buffer, 
            &self.gpu_bind_group
        ) {
            let strand_count = self.engine.strands().len() as u32;
            
            gpu.execute_step(buffer, bind_group, strand_count);
                
            let staging_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Readback Buffer"),
                size: buffer.size(),
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_buffer_to_buffer(buffer, 0, &staging_buffer, 0, buffer.size());
            gpu.queue.submit(Some(encoder.finish()));

            let buffer_slice = staging_buffer.slice(..);
            let (sender, receiver) = futures_channel::oneshot::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
                let _ = sender.send(v);
            });

            gpu.device.poll(wgpu::PollType::Poll);
            
            if receiver.await.is_ok() {
                let data = buffer_slice.get_mapped_range().unwrap();
                
                let strands_mut = self.engine.strands_mut();
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        data.as_ptr(),
                        strands_mut.as_mut_ptr() as *mut u8,
                        data.len()
                    );
                }
                drop(data);
                staging_buffer.unmap();
            }
        } else {
            self.engine.step();
        }

        self.update_buffers();
        Ok(())
    }

    pub async fn init_gpu(&mut self) -> Result<(), JsValue> {
        let gpu = physics::GpuPhysics::init().await.map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        let strands = self.engine.strands();
        let strands_bytes = unsafe {
            std::slice::from_raw_parts(
                strands.as_ptr() as *const u8,
                std::mem::size_of_val(strands)
            )
        };

        let strands_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Strands Buffer"),
            size: strands_bytes.len() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });
        strands_buffer.slice(..).get_mapped_range_mut().unwrap().copy_from_slice(strands_bytes);
        strands_buffer.unmap();

        let config = self.engine.config();
        let uniform_data: [f32; 8] = [
            config.tuning.gravity.x, config.tuning.gravity.y, config.tuning.gravity.z, config.tuning.dt,
            config.tuning.damping, config.tuning.rigidity, config.tuning.hair_length / (config.tuning.nodes_per_strand - 1) as f32, 0.0
        ];
        
        let uniform_bytes = unsafe {
            std::slice::from_raw_parts(
                uniform_data.as_ptr() as *const u8,
                std::mem::size_of_val(&uniform_data)
            )
        };

        let uniform_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: uniform_bytes.len() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });
        uniform_buffer.slice(..).get_mapped_range_mut().unwrap().copy_from_slice(uniform_bytes);
        uniform_buffer.unmap();

        let bind_group_layout = gpu.pipeline.get_bind_group_layout(0);
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Physics Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: strands_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        self.gpu_strands_buffer = Some(strands_buffer);
        self.gpu_bind_group = Some(bind_group);
        self.gpu_physics = Some(gpu);
        
        Ok(())
    }
}