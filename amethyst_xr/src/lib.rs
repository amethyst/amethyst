extern crate amethyst_assets;
extern crate amethyst_core;

pub mod components;
mod systems;

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

use amethyst_core::bundle::{Result, SystemBundle};
use amethyst_core::cgmath::{Quaternion, Vector3, Matrix4};
use amethyst_core::specs::prelude::DispatcherBuilder;

use amethyst_assets::Loader;

pub trait XRBackend: Send {
    fn wait(&mut self);

    fn get_new_trackers(&mut self) -> Option<Vec<(u32, TrackerCapabilities)>>;
    fn get_removed_trackers(&mut self) -> Option<Vec<u32>>;

    fn get_tracker_position(&mut self, index: u32) -> TrackerPositionData;

    fn get_area(&mut self) -> Vec<[f32; 3]>;
    fn get_hidden_area_mesh(&mut self) -> Vec<[f32; 3]>;

    fn get_tracker_models(&mut self, index: u32) -> TrackerModelLoadStatus;

    fn get_gl_target_info(&mut self, near: f32, far: f32) -> Vec<XRTargetInfo>;
    fn submit_gl_target(&mut self, target_index: usize, gl_target: usize);
}

#[derive(Debug, Clone)]
pub enum XREvent {
    AreaChanged,
    TrackerAdded(components::TrackingDevice),
    TrackerRemoved(u32),
}

#[derive(Debug, Clone)]
pub struct TrackerPositionData {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub velocity: Vector3<f32>,
    pub angular_velocity: Vector3<f32>,
    pub valid: bool,
}

#[derive(Debug, Clone)]
pub struct TrackerCapabilities {
    pub render_model_components: u32,
    pub is_camera: bool,
}

#[derive(Debug)]
pub enum TrackerModelLoadStatus {
    Unavailable,
    Pending,
    Available(Vec<TrackerComponentModelInfo>),
}

#[derive(Debug)]
pub struct TrackerComponentModelInfo {
    pub component_name: Option<String>,
    pub vertices: Vec<TrackerComponentVertex>,
    pub indices: Vec<u16>,
    pub texture: Option<TrackerComponentTextureData>,
}

#[derive(Debug, Clone)]
pub struct TrackerComponentVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub tex_coord: [f32; 2],
}

/// Assumed RGBA8 for now
#[derive(Debug)]
pub struct TrackerComponentTextureData {
    pub data: Vec<u8>,
    pub size: (u16, u16),
}

pub struct XRBundle<'a> {
    dep: &'a [&'a str],
    backend: Box<dyn XRBackend>,
}

impl<'a> XRBundle<'a> {
    pub fn new(backend: impl XRBackend + Send + 'static) -> XRBundle<'a> {
        XRBundle {
            dep: &[],
            backend: Box::new(backend),
        }
    }

    pub fn with_dep(mut self, dep: &'a [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c> SystemBundle<'a, 'b> for XRBundle<'c> {
    fn build(self, dispatcher: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        dispatcher.add(
            systems::XRSystem {
                backend: Some(self.backend),
            },
            "xr_system",
            self.dep,
        );

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct XRTargetInfo {
    pub size: (u32, u32),
    pub view_offset: Matrix4<f32>,
    pub projection: Matrix4<f32>,
}

pub struct XRInfo {
    targets: Vec<XRTargetInfo>,

    defined_area: Vec<[f32; 3]>,
    backend: Mutex<Box<dyn XRBackend>>,
}

impl XRInfo {
    pub fn targets(&self) -> &[XRTargetInfo] {
        &self.targets
    }

    pub fn backend(&mut self) -> MutexGuard<Box<dyn XRBackend>> {
        self.backend.lock().unwrap()
    }
}
