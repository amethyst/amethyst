use amethyst_core::cgmath::BaseNum;
use amethyst_core::cgmath::num_traits::NumCast;
use minterpolate::*;

use resources::SamplerPrimitive;

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

impl<S> InterpolationPrimitive for SamplerPrimitive<S>
where
    S: BaseNum,
{
    fn add(&self, other: &Self) -> Self {
        use SamplerPrimitive::*;
        match (*self, *other) {
            (Scalar(ref s), Scalar(ref o)) => Scalar(*s + *o),
            (Vec2(ref s), Vec2(ref o)) => Vec2([s[0] + o[0], s[1] + o[1]]),
            (Vec3(ref s), Vec3(ref o)) => Vec3([s[0] + o[0], s[1] + o[1], s[2] + o[2]]),
            (Vec4(ref s), Vec4(ref o)) => {
                Vec4([s[0] + o[0], s[1] + o[1], s[2] + o[2], s[3] + o[3]])
            }
            _ => panic!("Interpolation can not be done between primitives of different types"),
        }
    }

    fn sub(&self, other: &Self) -> Self {
        use SamplerPrimitive::*;
        match (*self, *other) {
            (Scalar(ref s), Scalar(ref o)) => Scalar(*s - *o),
            (Vec2(ref s), Vec2(ref o)) => Vec2([s[0] - o[0], s[1] - o[1]]),
            (Vec3(ref s), Vec3(ref o)) => Vec3([s[0] - o[0], s[1] - o[1], s[2] - o[2]]),
            (Vec4(ref s), Vec4(ref o)) => {
                Vec4([s[0] - o[0], s[1] - o[1], s[2] - o[2], s[3] - o[3]])
            }
            _ => panic!("Interpolation can not be done between primitives of different types"),
        }
    }

    fn mul(&self, scalar: f32) -> Self {
        use SamplerPrimitive::*;
        match *self {
            Scalar(ref s) => Scalar(mul_f32(*s, scalar)),
            Vec2(ref s) => Vec2([mul_f32(s[0], scalar), mul_f32(s[1], scalar)]),
            Vec3(ref s) => Vec3([
                mul_f32(s[0], scalar),
                mul_f32(s[1], scalar),
                mul_f32(s[2], scalar),
            ]),
            Vec4(ref s) => Vec4([
                mul_f32(s[0], scalar),
                mul_f32(s[1], scalar),
                mul_f32(s[2], scalar),
                mul_f32(s[3], scalar),
            ]),
        }
    }

    fn dot(&self, other: &Self) -> f32 {
        use SamplerPrimitive::*;
        match (*self, *other) {
            (Scalar(ref s), Scalar(ref o)) => (*s * *o).to_f32().unwrap(),
            (Vec2(ref s), Vec2(ref o)) => (s[0] * o[0] + s[1] * o[1]).to_f32().unwrap(),
            (Vec3(ref s), Vec3(ref o)) => {
                (s[0] * o[0] + s[1] * o[1] + s[2] * o[2]).to_f32().unwrap()
            }
            (Vec4(ref s), Vec4(ref o)) => (s[0] * o[0] + s[1] * o[1] + s[2] * o[2] + s[3] * o[3])
                .to_f32()
                .unwrap(),
            _ => panic!("Interpolation can not be done between primitives of different types"),
        }
    }

    fn magnitude2(&self) -> f32 {
        self.dot(self)
    }

    fn magnitude(&self) -> f32 {
        use SamplerPrimitive::*;
        match *self {
            Scalar(ref s) => s.to_f32().unwrap(),
            Vec2(_) | Vec3(_) | Vec4(_) => self.magnitude2().sqrt(),
        }
    }

    fn normalize(&self) -> Self {
        use SamplerPrimitive::*;
        match *self {
            Scalar(_) => *self,
            Vec2(_) | Vec3(_) | Vec4(_) => self.mul(1. / self.magnitude()),
        }
    }
}

fn mul_f32<T>(s: T, scalar: f32) -> T
where
    T: BaseNum,
{
    NumCast::from(s.to_f32().unwrap() * scalar).unwrap()
}
