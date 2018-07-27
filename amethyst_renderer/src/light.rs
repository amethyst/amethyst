//! Light sources.
//!
//! TODO: Remove redundant padding once `#[repr(align(...))]` stabilizes.

use amethyst_assets::{PrefabData, PrefabError};
use amethyst_core::specs::prelude::{Component, DenseVecStorage, Entity, Write, WriteStorage};
use gfx;

use color::Rgba;
use resources::AmbientColor;

/// A light source.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Light {
    /// An area light.
    /// FIXME: Missing implementation!
    Area,
    /// A directional light.
    Directional(DirectionalLight),
    /// A point light.
    Point(PointLight),
    /// A spot light.
    Spot(SpotLight),
    /// A sun light.
    Sun(SunLight),
}

/// A directional light source.
#[repr(C)]
#[derive(Clone, ConstantBuffer, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct DirectionalLight {
    /// Color of the light in RGBA8 format.
    pub color: Rgba,
    /// Direction that the light is pointing.
    pub direction: [f32; 3], //TODO: Replace with a cgmath type when gfx version > 0.16
}

impl Default for DirectionalLight {
    fn default() -> Self {
        DirectionalLight {
            color: Rgba::default(),
            direction: [-1.0, -1.0, -1.0],
        }
    }
}

impl From<DirectionalLight> for Light {
    fn from(dir: DirectionalLight) -> Self {
        Light::Directional(dir)
    }
}

/// A point light source. Uses the `Transform` set of components for
/// positioning, and requires a `GlobalTransform` component to be included
/// in rendering.
///
/// Lighting calculations are based off of the Frostbite engine's lighting,
/// which is explained in detail here in [this presentation][fb]. Below is
/// equation 26, which we used for the lighting evaluation.
///
/// <p align="center">
///     <img src="https://latex.codecogs.com/gif.latex?\dpi{100}&space;E_
///     {window1}&space;=&space;(\frac{I}{distance^{2}})&space;\cdot&space;
///     saturate(1&space;-&space;\frac{x^{n}}{lightRadius^{n}})^{2}"
///     alt="equation" />
/// </p>
///
/// The `Point` properties below map like so:
///
/// * *I* = `intensity`
/// * *lightRadius* = `radius`
/// * *n* = `smoothness`
///
/// [fb]: http://www.frostbite.com/wp-content/uploads/2014/11/course_notes_moving_frostbite_to_pbr.pdf
#[repr(C)]
#[derive(Clone, ConstantBuffer, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct PointLight {
    /// Color of the light in RGBA8 format.
    pub color: Rgba,
    /// Brightness of the light source, in lumens.
    pub intensity: f32,
    /// Maximum radius of the point light's affected area.
    pub radius: f32,
    /// Smoothness of the light-to-dark transition from the center to the
    /// radius.
    pub smoothness: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        PointLight {
            color: Rgba::default(),
            intensity: 10.0,
            radius: 10.0,
            smoothness: 4.0,
        }
    }
}

impl From<PointLight> for Light {
    fn from(pt: PointLight) -> Self {
        Light::Point(pt)
    }
}

/// A spot light source.
#[repr(C)]
#[derive(Clone, ConstantBuffer, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct SpotLight {
    /// Opening angle of the light cone in degrees.
    pub angle: f32, //TODO: Replace with a cgmath type when gfx version > 0.16
    /// Location of the light source in three dimensional space.
    pub center: [f32; 3], //TODO: Replace with a cgmath type when gfx version > 0.16
    /// Color of the light in RGBA8 format.
    pub color: Rgba,
    /// Direction that the light is pointing.
    pub direction: [f32; 3], //TODO: Replace with a cgmath type when gfx version > 0.16
    /// Brightness of the light source, in lumens.
    pub intensity: f32,
    /// Maximum radius of the point light's affected area.
    pub radius: f32,
    /// Smoothness of the light-to-dark transition from the center to the
    /// radius.
    pub smoothness: f32,
}

impl Default for SpotLight {
    fn default() -> Self {
        SpotLight {
            angle: 60.0,
            center: [0.0, 1.0, 0.0],
            color: Rgba::default(),
            direction: [0.0, -1.0, 0.0],
            intensity: 10.0,
            radius: 10.0,
            smoothness: 4.0,
        }
    }
}

impl From<SpotLight> for Light {
    fn from(sp: SpotLight) -> Self {
        Light::Spot(sp)
    }
}

/// A realistic disk-shaped sun light source.
#[repr(C)]
#[derive(Clone, ConstantBuffer, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct SunLight {
    /// The sun's angular radius in degrees.
    pub ang_rad: f32, //TODO: Replace with a cgmath type when gfx version > 0.16
    /// Color of the light in RGBA8 format.
    pub color: Rgba,
    /// Direction that the light is pointing.
    pub direction: [f32; 3], //TODO: Replace with a cgmath type when gfx version > 0.16
    /// Brightness of the sun light, in lux.
    pub intensity: f32,
}

impl Default for SunLight {
    fn default() -> Self {
        SunLight {
            ang_rad: 0.0093,
            color: Rgba::default(),
            direction: [-1.0, -1.0, -1.0],
            intensity: 64_000.0,
        }
    }
}

impl From<SunLight> for Light {
    fn from(sun: SunLight) -> Self {
        Light::Sun(sun)
    }
}

impl Component for Light {
    type Storage = DenseVecStorage<Self>;
}

impl<'a> PrefabData<'a> for Light {
    type SystemData = WriteStorage<'a, Light>;
    type Result = ();

    fn load_prefab(
        &self,
        entity: Entity,
        storage: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), PrefabError> {
        storage.insert(entity, self.clone()).map(|_| ())
    }
}

/// Prefab for lighting
#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LightPrefab {
    light: Option<Light>,
    ambient_color: Option<AmbientColor>,
}

impl<'a> PrefabData<'a> for LightPrefab {
    type SystemData = (
        <Light as PrefabData<'a>>::SystemData,
        Write<'a, AmbientColor>,
    );
    type Result = ();

    fn load_prefab(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), PrefabError> {
        if let Some(ref light) = self.light {
            light.load_prefab(entity, &mut system_data.0, &[])?;
        }
        if let Some(ref ambient_color) = self.ambient_color {
            *system_data.1 = ambient_color.clone();
        }
        Ok(())
    }
}
