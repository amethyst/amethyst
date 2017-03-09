//! Light sources.

use color::Rgba;
use cgmath::{Deg, Point3, Vector3};

/// A light source.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Light {
    /// An area light.
    Area,
    /// A directional light.
    Directional(Directional),
    /// A point light.
    Point(Point),
    /// A spot light.
    Spot(Spot),
    /// A sun light.
    Sun(Sun),
}

/// A directional light source.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Directional {
    /// Color of the light in RGBA8 format.
    pub color: Rgba,
    /// Direction that the light is pointing.
    pub direction: Vector3<f32>,
}

impl Default for Directional {
    fn default() -> Directional {
        Directional {
            color: Rgba::default(),
            direction: Vector3::new(-1.0, -1.0, -1.0),
        }
    }
}

impl From<Directional> for Light {
    fn from(dir: Directional) -> Light {
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
///     <img src="https://latex.codecogs.com/gif.latex?\dpi{100}&space;E_{window1}&space;=&space;(\frac{I}{distance^{2}})&space;\cdot&space;saturate(1&space;-&space;\frac{x^{n}}{lightRadius^{n}})^{2}" alt="equation" />
/// </p>
///
/// The `Point` properties below map like so:
///
/// * *I* = `intensity`
/// * *lightRadius* = `radius`
/// * *n* = `smoothness`
///
/// [fb]: http://www.frostbite.com/wp-content/uploads/2014/11/course_notes_moving_frostbite_to_pbr.pdf
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    /// Location of the light source in three dimensional space.
    pub center: Point3<f32>,
    /// Color of the light.
    pub color: Rgba,
    /// Brightness of the light source, in lumens.
    pub intensity: f32,
    /// Maximum radius of the point light's affected area.
    pub radius: f32,
    /// Smoothness of the light-to-dark transition from the center to the
    /// radius.
    pub smoothness: f32,
}

impl Default for Point {
    fn default() -> Point {
        Point {
            center: Point3::new(0.0, 0.0, 0.0),
            color: Rgba::default(),
            intensity: 10.0,
            radius: 10.0,
            smoothness: 4.0,
        }
    }
}

impl From<Point> for Light {
    fn from(pt: Point) -> Light {
        Light::Point(pt)
    }
}

/// A spot light source.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spot {
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

impl Default for Spot {
    fn default() -> Spot {
        Spot {
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

impl From<Spot> for Light {
    fn from(sp: Spot) -> Light {
        Light::Spot(sp)
    }
}

/// A realistic sun light source.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sun {
    /// The sun's angular radius.
    pub ang_rad: Deg<f32>,
    /// Color of the light in RGBA8 format.
    pub color: Rgba,
    /// Direction that the light is pointing.
    pub direction: Vector3<f32>,
    /// Brightness of the sun light, in lux.
    pub intensity: f32,
}

impl Default for Sun {
    fn default() -> Sun {
        Sun {
            ang_rad: Deg(0.0093),
            color: Rgba::default(),
            direction: [-1.0; 3].into(),
            intensity: 64_000.0,
        }
    }
}

impl From<Sun> for Light {
    fn from(sun: Sun) -> Light {
        Light::Sun(sun)
    }
}
