use std::{
    fmt::{Display, Formatter},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
};

use crate::num::*;
use alga::general::*;
use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use nalgebra::Complex;

#[cfg(not(feature = "float64"))]
use std::f32::consts;
#[cfg(feature = "float64")]
use std::f64::consts;

#[cfg(not(feature = "float64"))]
use std::f32 as minmax;
#[cfg(feature = "float64")]
use std::f64 as minmax;

#[cfg(feature = "float64")]
type FloatBase = f64;
#[cfg(not(feature = "float64"))]
type FloatBase = f32;

/// A wrapper type around f32 and f64. It is used to hide the actual type being used internally.
/// Mostly used with the `Transform` type.
/// The default type is f32 and you can switch to the f64 type by enabling the "float64" feature gate.
#[derive(Alga, Clone, Copy, PartialOrd, PartialEq, Serialize, Deserialize, Debug)]
#[alga_traits(Field(Additive, Multiplicative))]
#[serde(transparent)]
pub struct Float(FloatBase);

impl Float {
    /// Get the internal value as a f32. Can cause a loss of precision or a loss of data if using
    /// the "float64" feature.
    pub fn as_f32(self) -> f32 {
        self.0 as f32
    }
    /// Get the internal value as a f64.
    pub fn as_f64(self) -> f64 {
        self.0 as f64
    }
}

impl From<f32> for Float {
    fn from(val: f32) -> Self {
        Float(val as FloatBase)
    }
}

impl From<f64> for Float {
    fn from(val: f64) -> Self {
        Float(val as FloatBase)
    }
}

impl FromPrimitive for Float {
    fn from_i64(n: i64) -> Option<Self> {
        Some(Float(n as FloatBase))
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(Float(n as FloatBase))
    }
}

impl AbsDiffEq for Float {
    type Epsilon = Float;

    fn default_epsilon() -> Self::Epsilon {
        Float(FloatBase::default_epsilon())
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        FloatBase::abs_diff_eq(&self.0, &other.0, epsilon.0)
    }
}

impl UlpsEq for Float {
    fn default_max_ulps() -> u32 {
        FloatBase::default_max_ulps()
    }

    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        FloatBase::ulps_eq(&self.0, &other.0, epsilon.0, max_ulps)
    }
}

impl RelativeEq for Float {
    fn default_max_relative() -> Self::Epsilon {
        Float(FloatBase::default_max_relative())
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        FloatBase::relative_eq(&self.0, &other.0, epsilon.0, max_relative.0)
    }
}

impl Num for Float {
    type FromStrRadixErr = ParseFloatError;
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(Float(FloatBase::from_str_radix(src, radix)?))
    }
}

impl Bounded for Float {
    fn min_value() -> Self {
        Float(minmax::MIN)
    }
    fn max_value() -> Self {
        Float(minmax::MAX)
    }
}

impl One for Float {
    fn one() -> Self {
        Float(1.0)
    }
}

impl Zero for Float {
    fn zero() -> Self {
        Float(0.0)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
}

impl Signed for Float {
    fn abs(&self) -> Self {
        Float(self.0.abs())
    }

    fn abs_sub(&self, other: &Self) -> Self {
        if *self <= *other {
            Float(0.0)
        } else {
            *self - *other
        }
    }

    fn signum(&self) -> Self {
        if self.0 > 0.0 {
            Float(1.0)
        } else if self.0 == 0.0 {
            Float(0.0)
        } else {
            Float(-1.0)
        }
    }

    fn is_positive(&self) -> bool {
        self.0 >= 0.0
    }

    fn is_negative(&self) -> bool {
        self.0 <= 0.0
    }
}

impl Display for Float {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.0)
    }
}

impl Add for Float {
    type Output = Float;

    fn add(self, b: Float) -> Float {
        Float(self.0 + b.0)
    }
}

impl Sub for Float {
    type Output = Float;

    fn sub(self, b: Float) -> Float {
        Float(self.0 - b.0)
    }
}

impl Mul for Float {
    type Output = Float;

    fn mul(self, b: Float) -> Float {
        Float(self.0 * b.0)
    }
}

impl Div for Float {
    type Output = Float;

    fn div(self, b: Float) -> Float {
        Float(self.0 / b.0)
    }
}

impl Rem for Float {
    type Output = Float;

    fn rem(self, b: Float) -> Float {
        Float(self.0 % b.0)
    }
}

impl AddAssign for Float {
    fn add_assign(&mut self, b: Float) {
        self.0 += b.0;
    }
}

impl SubAssign for Float {
    fn sub_assign(&mut self, b: Float) {
        self.0 -= b.0;
    }
}

impl MulAssign for Float {
    fn mul_assign(&mut self, b: Float) {
        self.0 *= b.0;
    }
}

impl DivAssign for Float {
    fn div_assign(&mut self, b: Float) {
        self.0 /= b.0;
    }
}

impl RemAssign for Float {
    fn rem_assign(&mut self, b: Float) {
        self.0 %= b.0;
    }
}

impl Neg for Float {
    type Output = Float;

    fn neg(self) -> Float {
        Float(-self.0)
    }
}

impl AbstractMagma<Additive> for Float {
    fn operate(&self, right: &Self) -> Self {
        Float(self.0 + right.0)
    }
}

impl AbstractMagma<Multiplicative> for Float {
    fn operate(&self, right: &Self) -> Self {
        Float(self.0 * right.0)
    }
}

impl TwoSidedInverse<Additive> for Float {
    fn two_sided_inverse(&self) -> Self {
        Float(-self.0)
    }
}

impl TwoSidedInverse<Multiplicative> for Float {
    fn two_sided_inverse(&self) -> Self {
        Float(1. / self.0)
    }
}

impl Identity<Additive> for Float {
    fn identity() -> Self {
        Float(0.)
    }
}

impl Identity<Multiplicative> for Float {
    fn identity() -> Self {
        Float(1.)
    }
}

impl RealField for Float {
    #[inline]
    fn is_sign_positive(self) -> bool {
        FloatBase::is_sign_positive(self.0)
    }

    #[inline]
    fn is_sign_negative(self) -> bool {
        FloatBase::is_sign_negative(self.0)
    }

    #[inline]
    fn max(self, other: Self) -> Self {
        Float(self.0.max(other.0))
    }

    #[inline]
    fn min(self, other: Self) -> Self {
        Float(self.0.min(other.0))
    }

    #[inline]
    fn atan2(self, other: Self) -> Self {
        Float(FloatBase::atan2(self.0, other.0))
    }

    /// Archimedes' constant.
    #[inline]
    fn pi() -> Self {
        Float(consts::PI)
    }

    /// 2.0 * pi.
    #[inline]
    fn two_pi() -> Self {
        Float(consts::PI + consts::PI)
    }

    /// pi / 2.0.
    #[inline]
    fn frac_pi_2() -> Self {
        Float(consts::FRAC_PI_2)
    }

    /// pi / 3.0.
    #[inline]
    fn frac_pi_3() -> Self {
        Float(consts::FRAC_PI_3)
    }

    /// pi / 4.0.
    #[inline]
    fn frac_pi_4() -> Self {
        Float(consts::FRAC_PI_4)
    }

    /// pi / 6.0.
    #[inline]
    fn frac_pi_6() -> Self {
        Float(consts::FRAC_PI_6)
    }

    /// pi / 8.0.
    #[inline]
    fn frac_pi_8() -> Self {
        Float(consts::FRAC_PI_8)
    }

    /// 1.0 / pi.
    #[inline]
    fn frac_1_pi() -> Self {
        Float(consts::FRAC_1_PI)
    }

    /// 2.0 / pi.
    #[inline]
    fn frac_2_pi() -> Self {
        Float(consts::FRAC_2_PI)
    }

    /// 2.0 / sqrt(pi).
    #[inline]
    fn frac_2_sqrt_pi() -> Self {
        Float(consts::FRAC_2_SQRT_PI)
    }

    /// Euler's number.
    #[inline]
    fn e() -> Self {
        Float(consts::E)
    }

    /// log2(e).
    #[inline]
    fn log2_e() -> Self {
        Float(consts::LOG2_E)
    }

    /// log10(e).
    #[inline]
    fn log10_e() -> Self {
        Float(consts::LOG10_E)
    }

    /// ln(2.0).
    #[inline]
    fn ln_2() -> Self {
        Float(consts::LN_2)
    }

    /// ln(10.0).
    #[inline]
    fn ln_10() -> Self {
        Float(consts::LN_10)
    }
}

impl ComplexField for Float {
    type RealField = Float;

    #[inline]
    fn from_real(re: Self::RealField) -> Self {
        re
    }

    #[inline]
    fn real(self) -> Self::RealField {
        self
    }

    #[inline]
    fn imaginary(self) -> Self::RealField {
        Self::zero()
    }

    #[inline]
    fn norm1(self) -> Self::RealField {
        Float(FloatBase::abs(self.0))
    }

    #[inline]
    fn modulus(self) -> Self::RealField {
        Float(FloatBase::abs(self.0))
    }

    #[inline]
    fn modulus_squared(self) -> Self::RealField {
        Float(self.0 * self.0)
    }

    #[inline]
    fn argument(self) -> Self::RealField {
        if self >= Self::zero() {
            Self::zero()
        } else {
            Self::pi()
        }
    }

    #[inline]
    fn to_exp(self) -> (Self, Self) {
        if self >= Self::zero() {
            (self, Self::one())
        } else {
            (-self, -Self::one())
        }
    }

    #[inline]
    fn recip(self) -> Self {
        Float(FloatBase::recip(self.0))
    }

    #[inline]
    fn conjugate(self) -> Self {
        Float(self.0)
    }

    #[inline]
    fn scale(self, factor: Self::RealField) -> Self {
        Float(self.0 * factor.0)
    }

    #[inline]
    fn unscale(self, factor: Self::RealField) -> Self {
        Float(self.0 / factor.0)
    }

    #[inline]
    fn floor(self) -> Self {
        Float(FloatBase::floor(self.0))
    }

    #[inline]
    fn ceil(self) -> Self {
        Float(FloatBase::ceil(self.0))
    }

    #[inline]
    fn round(self) -> Self {
        Float(FloatBase::round(self.0))
    }

    #[inline]
    fn trunc(self) -> Self {
        Float(FloatBase::trunc(self.0))
    }

    #[inline]
    fn fract(self) -> Self {
        Float(FloatBase::fract(self.0))
    }

    #[inline]
    fn abs(self) -> Self {
        Float(FloatBase::abs(self.0))
    }

    #[inline]
    fn signum(self) -> Self {
        Float(Signed::signum(&self.0))
    }

    #[inline]
    fn mul_add(self, a: Self, b: Self) -> Self {
        Float(FloatBase::mul_add(self.0, a.0, b.0))
    }

    #[cfg(feature = "std")]
    #[inline]
    fn powi(self, n: i32) -> Self {
        Float(self.0.powi(n))
    }

    #[cfg(not(feature = "std"))]
    #[inline]
    fn powi(self, n: i32) -> Self {
        Float(FloatBase::powf(self.0, n as FloatBase))
    }

    #[inline]
    fn powf(self, n: Self) -> Self {
        Float(FloatBase::powf(self.0, n.0))
    }

    #[inline]
    fn powc(self, n: Self) -> Self {
        // Same as powf.
        Float(FloatBase::powf(self.0, n.0))
    }

    #[inline]
    fn sqrt(self) -> Self {
        Float(FloatBase::sqrt(self.0))
    }

    #[inline]
    fn try_sqrt(self) -> Option<Self> {
        if self >= Self::zero() {
            Some(Float(FloatBase::sqrt(self.0)))
        } else {
            None
        }
    }

    #[inline]
    fn exp(self) -> Self {
        Float(FloatBase::exp(self.0))
    }

    #[inline]
    fn exp2(self) -> Self {
        Float(FloatBase::exp2(self.0))
    }

    #[inline]
    fn exp_m1(self) -> Self {
        Float(FloatBase::exp_m1(self.0))
    }

    #[inline]
    fn ln_1p(self) -> Self {
        Float(FloatBase::ln_1p(self.0))
    }

    #[inline]
    fn ln(self) -> Self {
        Float(FloatBase::ln(self.0))
    }

    #[inline]
    fn log(self, base: Self) -> Self {
        Float(FloatBase::log(self.0, base.0))
    }

    #[inline]
    fn log2(self) -> Self {
        Float(FloatBase::log2(self.0))
    }

    #[inline]
    fn log10(self) -> Self {
        Float(FloatBase::log10(self.0))
    }

    #[inline]
    fn cbrt(self) -> Self {
        Float(FloatBase::cbrt(self.0))
    }

    #[inline]
    fn hypot(self, other: Self) -> Self::RealField {
        Float(FloatBase::hypot(self.0, other.0))
    }

    #[inline]
    fn sin(self) -> Self {
        Float(FloatBase::sin(self.0))
    }

    #[inline]
    fn cos(self) -> Self {
        Float(FloatBase::cos(self.0))
    }

    #[inline]
    fn tan(self) -> Self {
        Float(FloatBase::tan(self.0))
    }

    #[inline]
    fn asin(self) -> Self {
        Float(FloatBase::asin(self.0))
    }

    #[inline]
    fn acos(self) -> Self {
        Float(FloatBase::acos(self.0))
    }

    #[inline]
    fn atan(self) -> Self {
        Float(FloatBase::atan(self.0))
    }

    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        let vals = FloatBase::sin_cos(self.0);
        (Float(vals.0), Float(vals.1))
    }

    #[inline]
    fn sinh(self) -> Self {
        Float(FloatBase::sinh(self.0))
    }

    #[inline]
    fn cosh(self) -> Self {
        Float(FloatBase::cosh(self.0))
    }

    #[inline]
    fn tanh(self) -> Self {
        Float(FloatBase::tanh(self.0))
    }

    #[inline]
    fn asinh(self) -> Self {
        Float(FloatBase::asinh(self.0))
    }

    #[inline]
    fn acosh(self) -> Self {
        Float(FloatBase::acosh(self.0))
    }

    #[inline]
    fn atanh(self) -> Self {
        Float(FloatBase::atanh(self.0))
    }

    #[inline]
    fn is_finite(&self) -> bool {
        self.0.is_finite()
    }
}

impl SubsetOf<f32> for Float {
    #[inline]
    fn to_superset(&self) -> f32 {
        self.0 as f32
    }

    #[inline]
    unsafe fn from_superset_unchecked(element: &f32) -> Self {
        Float(*element as FloatBase)
    }

    #[inline]
    fn is_in_subset(_: &f32) -> bool {
        true
    }
}

impl SubsetOf<f64> for Float {
    #[inline]
    fn to_superset(&self) -> f64 {
        self.0 as f64
    }

    #[inline]
    unsafe fn from_superset_unchecked(element: &f64) -> Self {
        Float(*element as FloatBase)
    }

    #[inline]
    fn is_in_subset(_: &f64) -> bool {
        true
    }
}

impl SubsetOf<Float> for f32 {
    #[inline]
    fn to_superset(&self) -> Float {
        Float(*self as FloatBase)
    }

    #[inline]
    unsafe fn from_superset_unchecked(element: &Float) -> Self {
        element.0 as Self
    }

    #[inline]
    fn is_in_subset(_: &Float) -> bool {
        true
    }
}

impl SubsetOf<Float> for f64 {
    #[inline]
    fn to_superset(&self) -> Float {
        Float(*self as FloatBase)
    }

    #[inline]
    unsafe fn from_superset_unchecked(element: &Float) -> Self {
        element.0 as Self
    }

    #[inline]
    fn is_in_subset(_: &Float) -> bool {
        true
    }
}

impl SubsetOf<Float> for Float {
    #[inline]
    fn to_superset(&self) -> Float {
        Float(self.0 as FloatBase)
    }

    #[inline]
    unsafe fn from_superset_unchecked(element: &Float) -> Self {
        Float(element.0)
    }

    #[inline]
    fn is_in_subset(_: &Float) -> bool {
        true
    }
}

impl MeetSemilattice for Float {
    #[inline]
    fn meet(&self, other: &Self) -> Self {
        if *self <= *other {
            *self
        } else {
            *other
        }
    }
}

impl JoinSemilattice for Float {
    #[inline]
    fn join(&self, other: &Self) -> Self {
        if *self >= *other {
            *self
        } else {
            *other
        }
    }
}

impl Lattice for Float {
    #[inline]
    fn meet_join(&self, other: &Self) -> (Self, Self) {
        if *self >= *other {
            (*other, *self)
        } else {
            (*self, *other)
        }
    }
}

impl<N2: Zero + SupersetOf<Float>> SubsetOf<Complex<N2>> for Float {
    #[inline]
    fn to_superset(&self) -> Complex<N2> {
        Complex {
            re: N2::from_subset(self),
            im: N2::zero(),
        }
    }

    #[inline]
    unsafe fn from_superset_unchecked(element: &Complex<N2>) -> Float {
        element.re.to_subset_unchecked()
    }

    #[inline]
    fn is_in_subset(c: &Complex<N2>) -> bool {
        c.re.is_in_subset() && c.im.is_zero()
    }
}
