//! Light sources.
//!
//! TODO: Remove redundant padding once `#[repr(align(...))]` stabilizes.

use crate::resources::AmbientColor;
use amethyst_assets::{PrefabData, ProgressCounter};
use amethyst_core::{
    ecs::prelude::{Component, DenseVecStorage, Entity, WriteStorage},
    math::Vector3,
};
use amethyst_error::Error;
pub use punctual::*;
pub use area::*;

/// A light source.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize, PrefabData)]
#[prefab(Component)]
pub enum Light {
    /// A directional light.
    Directional(DirectionalLight),
    /// A point light.
    Point(PointLight),
    /// A spot light.
    Spot(SpotLight),
    // /// A sun light.
    // Sun(SunLight),
    // An area light.
    Area(AreaLight),
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

pub mod punctual {
    use super::Light;
    use amethyst_core::{
        math::Vector3,
    };
    
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
        /// Brightness of the light source, in lumens (lm).
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
}

/// Submodule for Area Lights
pub mod area {
    /// An area light source.
    #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    pub enum AreaLight {
        /// A Sperical Area Light
        Sphere(Sphere),
        /// A Disk Area Light
        Disk(Disk),
        /// A Rectangular Area Light
        Rectangle(Rectangle),
    }

    /// Different Intensity units.
    /// 
    /// @todo Add EV values to intensity.
    #[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    pub enum Intensity {
        /// Intensity in lumens (lm).
        /// Power
        Power(f32),
        /// Intensity in nits (cd/m^2).
        /// Power per area per solid angle.
        Luminance(f32),
    }
    impl Intensity {
        /// Return the intensitiy in luminance (nits) or convert the power (lm) to luminance.
        /// 
        /// # Example
        /// ```
        /// let intensity = Intensity::Power(1200.0);
        /// let light_area = 2.0;
        /// 
        /// let _common_intensity = intensity.luminance_or(|x| x/4 * light_area);
        /// ```
        pub fn luminance_or<F>(self, conversion: F) -> f32
        where F: FnOnce(f32) -> f32 {
            match self {
                Intensity::Power(v) => conversion(v),
                Intensity::Luminance(v) => v
            }
        }
        /// DO NOT USE!
        pub fn get(self) -> f32{
            match self {
                Intensity::Power(v) => v,
                Intensity::Luminance(v) => v
            }
        }
    }

    impl Default for Intensity {
        /// Default value of a 75W/120V ~ 100W/230V incandescent light bulb.
        fn default() -> Self {
            Intensity::Power(1_200.0)
        }
    }

    /// A disk shaped area light source.
    #[repr(C)]
    #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    #[serde(default)]
    pub struct Disk {
        /// Diffuse color of the light source.
        #[serde(with = "crate::serde_shim::srgb")]
        pub diffuse_color: palette::Srgb,
        /// Specular color of the light source.
        #[serde(with = "crate::serde_shim::srgb")]
        pub spec_color: palette::Srgb,
        /// Intensity of the light source.
        pub intensity: Intensity,
        // /// Radius of the disk light source. Maybe removed to use the transform component for area size computations.
        // pub radius: f32,
        /// Does the light source output light on both sides of the disk.
        pub two_sided: bool
    }

    impl Default for Disk {
        /// Default value of a 75W/120V ~ 100W/230V incandescent light bulb.
        fn default() -> Self {
            Disk {
                diffuse_color: Default::default(),
                spec_color: Default::default(),
                intensity: Default::default(),
                // radius: 1.0,
                two_sided: false
            }
        }
    }

    /// A sphere shaped area light source.
    #[repr(C)]
    #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    #[serde(default)]
    pub struct Sphere {
        /// Diffuse color of the light source.
        #[serde(with = "crate::serde_shim::srgb")]
        pub diffuse_color: palette::Srgb,
        /// Specular color of the light source.
        #[serde(with = "crate::serde_shim::srgb")]
        pub spec_color: palette::Srgb,
        /// Intensity of the light source.
        pub intensity: Intensity,
        // pub radius: f32,
    }

    impl Default for Sphere {
        /// Default value of a 75W/120V ~ 100W/230V incandescent light bulb.
        fn default() -> Self {
            Sphere {
                diffuse_color: Default::default(),
                spec_color: Default::default(),
                intensity: Default::default(),
                // radius: 1.0
            }
        }
    }

    /// A rectangular area light source.
    #[repr(C)]
    #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    #[serde(default)]
    pub struct Rectangle {
        #[serde(with = "crate::serde_shim::srgb")]
        pub diffuse_color: palette::Srgb,
        #[serde(with = "crate::serde_shim::srgb")]
        pub spec_color: palette::Srgb,
        pub intensity: Intensity,
        // pub width: f32,
        // pub height: f32,
        pub two_sided: bool
    }

    impl Default for Rectangle {
        /// Default value of a 75W/120V ~ 100W/230V incandescent light bulb.
        fn default() -> Self {
            Rectangle {
                diffuse_color: Default::default(),
                spec_color: Default::default(),
                intensity: Default::default(),
                // width: 2.0,
                // height: 1.0,
                two_sided: false
            }
        }
    }
}

impl Component for Light {
    type Storage = DenseVecStorage<Self>;
}

/// Prefab for lighting
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, PrefabData)]
#[serde(default)]
pub struct LightPrefab {
    light: Option<Light>,
    ambient_color: Option<AmbientColor>,
}
