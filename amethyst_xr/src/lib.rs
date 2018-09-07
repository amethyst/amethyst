extern crate amethyst_assets;
extern crate amethyst_core;
extern crate amethyst_renderer;

pub mod components;
mod systems;

use std::collections::BTreeMap;

use amethyst_core::bundle::{Result, SystemBundle};
use amethyst_core::cgmath::{Quaternion, Vector3};
use amethyst_core::specs::prelude::DispatcherBuilder;

use amethyst_assets::Loader;

use amethyst_renderer::{PosNormTangTex, TextureData};

pub trait XRBackend: Send {
    fn wait(&mut self);

    fn get_new_trackers(&mut self) -> Option<Vec<u32>>;
    fn get_removed_trackers(&mut self) -> Option<Vec<u32>>;

    fn get_tracker_position(&mut self, index: u32) -> TrackerPositionData;

    fn get_area(&mut self) -> Vec<[f32; 3]>;
    fn get_hidden_area_mesh(&mut self) -> Vec<[f32; 3]>;

    fn get_tracker_models(&mut self, index: u32) -> TrackerModelLoadStatus;

    fn get_tracker_capabilities(&mut self, index: u32) -> TrackerCapabilities;
}

pub enum XREvent {
    AreaChanged,
    TrackerAdded(components::TrackingDevice),
    TrackerRemoved(u32),
    TrackerModelLoaded(u32),
}

#[derive(Debug)]
pub struct TrackerPositionData {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub velocity: Vector3<f32>,
    pub angular_velocity: Vector3<f32>,
    pub valid: bool,
}

#[derive(Debug)]
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
    pub vertices: Vec<PosNormTangTex>,
    pub indices: Vec<u16>,
    pub texture: Option<TextureData>,
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
                backend: self.backend,
            },
            "xr_system",
            self.dep,
        );

        Ok(())
    }
}

pub struct XRInfo {
    pub defined_area: Vec<[f32; 3]>,
}
