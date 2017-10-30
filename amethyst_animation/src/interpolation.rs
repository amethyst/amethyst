use std::fmt::Debug;

use minterpolate::*;

/// Interpolate over data set of the given type.
pub trait Interpolate<T> {
    /// Interpolation function, `f(input) -> T`
    ///
    /// ## Parameters
    ///
    /// - `input`: the input value to the function
    /// - `inputs`: list of discrete input values for each keyframe
    /// - `outputs`: list of output values to interpolate between, note that this data set is
    ///              tied to the interpolation function, and there is no guarantee or requirement
    ///              that it is the same size as the inputs.
    /// - `normalize`: if true, normalize the interpolated value before returning it
    fn interpolate(&self, input: f32, inputs: &[f32], outputs: &[T], normalize: bool) -> T;
}

/// Trait used if an outside user wants to supply their own interpolation function
pub trait InterpolationFunction
    : Interpolate<[f32; 3]> + Interpolate<[f32; 4]> + Send + Sync + Debug {
}

/// Supported interpolation functions
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum InterpolationType {
    /// Linear interpolation
    Linear,
    /// Step interpolation
    Step,
    /// Catmull-Rom spline interpolation
    CatmullRomSpline,
    /// Cubic Hermite spline interpolation
    CubicSpline,
    // User supplied interpolation function
    //#[serde(skip_serializing, skip_deserializing)]
    //TODO: Function(Box<InterpolationFunction>),
}

impl Interpolate<[f32; 3]> for InterpolationType {
    fn interpolate(
        &self,
        input: f32,
        inputs: &[f32],
        outputs: &[[f32; 3]],
        normalize: bool,
    ) -> [f32; 3] {
        match *self {
            InterpolationType::Linear => linear_interpolate(input, inputs, outputs, normalize),
            InterpolationType::Step => step_interpolate(input, inputs, outputs, normalize),
            InterpolationType::CubicSpline => {
                cubic_spline_interpolate(input, inputs, outputs, normalize)
            }
            InterpolationType::CatmullRomSpline => {
                catmull_rom_spline_interpolate(input, inputs, outputs, normalize)
            }
            //InterpolationType::Function(ref f) => f.interpolate(input, inputs, outputs, normalize),
        }
    }
}

impl Interpolate<[f32; 4]> for InterpolationType {
    fn interpolate(
        &self,
        input: f32,
        inputs: &[f32],
        outputs: &[[f32; 4]],
        normalize: bool,
    ) -> [f32; 4] {
        match *self {
            InterpolationType::Linear => linear_interpolate(input, inputs, outputs, normalize),
            InterpolationType::Step => step_interpolate(input, inputs, outputs, normalize),
            InterpolationType::CubicSpline => {
                cubic_spline_interpolate(input, inputs, outputs, normalize)
            }
            InterpolationType::CatmullRomSpline => {
                catmull_rom_spline_interpolate(input, inputs, outputs, normalize)
            }
            //InterpolationType::Function(ref f) => f.interpolate(input, inputs, outputs, normalize),
        }
    }
}
