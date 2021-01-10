//! Light sources.
//!
//! TODO: Remove redundant padding once `#[repr(align(...))]` stabilizes.

use amethyst_core::math::Vector3;
use type_uuid::TypeUuid;

/// A light source.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize, TypeUuid)]
#[uuid = "32cf5344-28c1-41c4-a1f9-ea87de4b1a4f"]
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
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct DirectionalLight {
    /// Color of the light in SRGB format.
    #[serde(with = "crate::serde_shim::srgb")]
    pub color: palette::Srgb,
    /// Brightness of the light source, different unit from Spot and PointLight.
    pub intensity: f32,
    /// Direction that the light is pointing.
    pub direction: Vector3<f32>,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        DirectionalLight {
            color: Default::default(),
            intensity: 1.0,
            direction: [-1.0, -1.0, -1.0].into(),
        }
    }
}

impl From<DirectionalLight> for Light {
    fn from(dir: DirectionalLight) -> Self {
        Light::Directional(dir)
    }
}

/// A point light source. Uses the `Transform` set of components for positioning.
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
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PointLight {
    /// Color of the light in SRGB format.
    #[serde(with = "crate::serde_shim::srgb")]
    pub color: palette::Srgb,
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
            color: Default::default(),
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
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SpotLight {
    /// Opening angle of the light cone in radians.
    pub angle: f32,
    /// Color of the light in SRGB format.
    #[serde(with = "crate::serde_shim::srgb")]
    pub color: palette::Srgb,
    /// Direction that the light is pointing.
    pub direction: Vector3<f32>,
    /// Brightness of the light source, in lumens.
    pub intensity: f32,
    /// Range/length of the light source.
    pub range: f32,
    /// Smoothness of the light-to-dark transition from the center to the
    /// radius.
    pub smoothness: f32,
}

impl Default for SpotLight {
    fn default() -> Self {
        SpotLight {
            angle: std::f32::consts::FRAC_PI_3,
            color: Default::default(),
            direction: [0.0, -1.0, 0.0].into(),
            intensity: 10.0,
            range: 10.0,
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
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SunLight {
    /// The sun's angular radius in radians.
    pub angle: f32,
    /// Color of the light in SRGB format.
    #[serde(with = "crate::serde_shim::srgb")]
    pub color: palette::Srgb,
    /// Direction that the light is pointing.
    pub direction: Vector3<f32>,
    /// Brightness of the sun light, in lux.
    pub intensity: f32,
}

impl Default for SunLight {
    fn default() -> Self {
        SunLight {
            angle: 0.0093_f32.to_radians(),
            color: Default::default(),
            direction: [-1.0, -1.0, -1.0].into(),
            intensity: 64_000.0,
        }
    }
}

impl From<SunLight> for Light {
    fn from(sun: SunLight) -> Self {
        Light::Sun(sun)
    }
}
