use minterpolate::*;

/// Interpolate over data set of the given type.
pub trait Interpolate<T>
where
    T: InterpolationPrimitive + Copy,
{
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

/// Supported interpolation functions
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum InterpolationType {
    /// Linear interpolation
    Linear,
    /// Spherical linear interpolation
    SphericalLinear,
    /// Step interpolation
    Step,
    /// Catmull-Rom spline interpolation
    CatmullRomSpline,
    /// Cubic Hermite spline interpolation
    CubicSpline,
}

impl<T> Interpolate<T> for InterpolationType
where
    T: InterpolationPrimitive + Copy,
{
    fn interpolate(&self, input: f32, inputs: &[f32], outputs: &[T], normalize: bool) -> T {
        match *self {
            InterpolationType::Linear => linear_interpolate(input, inputs, outputs, normalize),
            InterpolationType::SphericalLinear => {
                spherical_linear_interpolate(input, inputs, outputs, normalize)
            }
            InterpolationType::Step => step_interpolate(input, inputs, outputs, normalize),
            InterpolationType::CubicSpline => {
                cubic_spline_interpolate(input, inputs, outputs, normalize)
            }
            InterpolationType::CatmullRomSpline => {
                catmull_rom_spline_interpolate(input, inputs, outputs, normalize)
            }
        }
    }
}
