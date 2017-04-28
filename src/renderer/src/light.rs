//! Light sources.

use cgmath::{Deg, Point3, Vector3};
use color::Rgba;
use gfx;

/// A light source.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Light {
    /// An area light.
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
#[derive(Clone, ConstantBuffer, Debug, Deserialize, PartialEq, Serialize)]
pub struct DirectionalLight {
    /// Color of the light in RGBA8 format.
    pub color: Rgba,
    /// Direction that the light is pointing.
    pub direction: Vector3<f32>,
}

impl Default for DirectionalLight {
    fn default() -> DirectionalLight {
        DirectionalLight {
            color: Rgba::default(),
            direction: Vector3::new(-1.0, -1.0, -1.0),
        }
    }
}

impl From<DirectionalLight> for Light {
    fn from(dir: DirectionalLight) -> Light {
        Light::Directional(dir)
    }
}

/// A point light source.
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
#[derive(Clone, ConstantBuffer, Debug, Deserialize, PartialEq, Serialize)]
pub struct PointLight {
    /// Location of the light source in three dimensional space.
    pub center: Point3<f32>,
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
    fn default() -> PointLight {
        PointLight {
            center: Point3::new(0.0, 0.0, 0.0),
            color: Rgba::default(),
            intensity: 10.0,
            radius: 10.0,
            smoothness: 4.0,
        }
    }
}

impl From<PointLight> for Light {
    fn from(pt: PointLight) -> Light {
        Light::Point(pt)
    }
}

/// A spot light source.
#[derive(Clone, ConstantBuffer, Debug, Deserialize, PartialEq, Serialize)]
pub struct SpotLight {
    /// Opening angle of the light cone.
    pub angle: Deg<f32>,
    /// Location of the light source in three dimensional space.
    pub center: Point3<f32>,
    /// Color of the light in RGBA8 format.
    pub color: Rgba,
    /// Direction that the light is pointing.
    pub direction: Vector3<f32>,
    /// Brightness of the light source, in lumens.
    pub intensity: f32,
    /// Maximum radius of the point light's affected area.
    pub radius: f32,
    /// Smoothness of the light-to-dark transition from the center to the
    /// radius.
    pub smoothness: f32,
}

impl Default for SpotLight {
    fn default() -> SpotLight {
        SpotLight {
            angle: Deg(60.0),
            center: Point3::new(0.0, 1.0, 0.0),
            color: Rgba::default(),
            direction: Vector3::new(0.0, -1.0, 0.0),
            intensity: 10.0,
            radius: 10.0,
            smoothness: 4.0,
        }
    }
}

impl From<SpotLight> for Light {
    fn from(sp: SpotLight) -> Light {
        Light::Spot(sp)
    }
}

/// A realistic disk-shaped sun light source.
#[derive(Clone, ConstantBuffer, Debug, Deserialize, PartialEq, Serialize)]
pub struct SunLight {
    /// The sun's angular radius.
    pub ang_rad: Deg<f32>,
    /// Color of the light in RGBA8 format.
    pub color: Rgba,
    /// Direction that the light is pointing.
    pub direction: Vector3<f32>,
    /// Brightness of the sun light, in lux.
    pub intensity: f32,
}

impl Default for SunLight {
    fn default() -> SunLight {
        SunLight {
            ang_rad: Deg(0.0093),
            color: Rgba::default(),
            direction: Vector3::new(-1.0, -1.0, -1.0),
            intensity: 64_000.0,
        }
    }
}

impl From<SunLight> for Light {
    fn from(sun: SunLight) -> Light {
        Light::Sun(sun)
    }
}
