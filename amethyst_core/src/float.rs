use std::{
    fmt::{Display, Formatter},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
};

use crate::num::{Bounded, FromPrimitive, Num, One, ParseFloatError, Signed, Zero};
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
pub struct f32(FloatBase);

impl f32 {
    /// Returns a new `Float` from a `f64`.
    pub const fn from_f64(value: f64) -> Self {
        f32(value as FloatBase)
    }

    /// Returns a new `Float` from a `f32`.
    pub const fn from_f32(value: f32) -> Self {
        f32(value as FloatBase)
    }

    /// Get the internal value as a f32. Will cause a loss in precision if using
    /// the "float64" feature.
    pub const fn as_f32(self) -> f32 {
        self.0 as f32
    }

    /// Get the internal value as a f64. Guaranteed to be lossless.
    pub const fn as_f64(self) -> f64 {
        self.0 as f64
    }
}

impl From<f32> for f32 {
    fn from(val: f32) -> Self {
        f32(val as FloatBase)
    }
}

impl From<f64> for f32 {
    fn from(val: f64) -> Self {
        f32(val as FloatBase)
    }
}

impl FromPrimitive for f32 {
    fn from_i64(n: i64) -> Option<Self> {
        Some(f32(n as FloatBase))
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(f32(n as FloatBase))
    }
}

impl AbsDiffEq for f32 {
    type Epsilon = f32;

    fn default_epsilon() -> Self::Epsilon {
        f32(FloatBase::default_epsilon())
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        FloatBase::abs_diff_eq(&self.0, &other.0, epsilon.0)
    }
}

impl UlpsEq for f32 {
    fn default_max_ulps() -> u32 {
        FloatBase::default_max_ulps()
    }

    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        FloatBase::ulps_eq(&self.0, &other.0, epsilon.0, max_ulps)
    }
}

impl RelativeEq for f32 {
    fn default_max_relative() -> Self::Epsilon {
        f32(FloatBase::default_max_relative())
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

impl Num for f32 {
    type FromStrRadixErr = ParseFloatError;
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(f32(FloatBase::from_str_radix(src, radix)?))
    }
}

impl Bounded for f32 {
    fn min_value() -> Self {
        f32(minmax::MIN)
    }
    fn max_value() -> Self {
        f32(minmax::MAX)
    }
}

impl One for f32 {
    fn one() -> Self {
        f32(1.0)
    }
}

impl Zero for f32 {
    fn zero() -> Self {
        f32(0.0)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
}

impl Signed for f32 {
    fn abs(&self) -> Self {
        f32(self.0.abs())
    }

    fn abs_sub(&self, other: &Self) -> Self {
        if *self <= *other {
            f32(0.0)
        } else {
            *self - *other
        }
    }

    fn signum(&self) -> Self {
        if self.0 > 0.0 {
            f32(1.0)
        } else if self.0 == 0.0 {
            f32(0.0)
        } else {
            f32(-1.0)
        }
    }

    fn is_positive(&self) -> bool {
        self.0 >= 0.0
    }

    fn is_negative(&self) -> bool {
        self.0 <= 0.0
    }
}

impl Display for f32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.0)
    }
}

impl Add for f32 {
    type Output = f32;

    fn add(self, b: f32) -> f32 {
        f32(self.0 + b.0)
    }
}

impl Sub for f32 {
    type Output = f32;

    fn sub(self, b: f32) -> f32 {
        f32(self.0 - b.0)
    }
}

impl Mul for f32 {
    type Output = f32;

    fn mul(self, b: f32) -> f32 {
        f32(self.0 * b.0)
    }
}

impl Div for f32 {
    type Output = f32;

    fn div(self, b: f32) -> f32 {
        f32(self.0 / b.0)
    }
}

impl Rem for f32 {
    type Output = f32;

    fn rem(self, b: f32) -> f32 {
        f32(self.0 % b.0)
    }
}

impl AddAssign for f32 {
    fn add_assign(&mut self, b: f32) {
        self.0 += b.0;
    }
}

impl SubAssign for f32 {
    fn sub_assign(&mut self, b: f32) {
        self.0 -= b.0;
    }
}

impl MulAssign for f32 {
    fn mul_assign(&mut self, b: f32) {
        self.0 *= b.0;
    }
}

impl DivAssign for f32 {
    fn div_assign(&mut self, b: f32) {
        self.0 /= b.0;
    }
}

impl RemAssign for f32 {
    fn rem_assign(&mut self, b: f32) {
        self.0 %= b.0;
    }
}

impl Neg for f32 {
    type Output = f32;

    fn neg(self) -> f32 {
        f32(-self.0)
    }
}

impl AbstractMagma<Additive> for f32 {
    fn operate(&self, right: &Self) -> Self {
        f32(self.0 + right.0)
    }
}

impl AbstractMagma<Multiplicative> for f32 {
    fn operate(&self, right: &Self) -> Self {
        f32(self.0 * right.0)
    }
}

impl TwoSidedInverse<Additive> for f32 {
    fn two_sided_inverse(&self) -> Self {
        f32(-self.0)
    }
}

impl TwoSidedInverse<Multiplicative> for f32 {
    fn two_sided_inverse(&self) -> Self {
        f32(1. / self.0)
    }
}

impl Identity<Additive> for f32 {
    fn identity() -> Self {
        f32(0.)
    }
}

impl Identity<Multiplicative> for f32 {
    fn identity() -> Self {
        f32(1.)
    }
}

impl RealField for f32 {
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
        f32(self.0.max(other.0))
    }

    #[inline]
    fn min(self, other: Self) -> Self {
        f32(self.0.min(other.0))
    }

    #[inline]
    fn atan2(self, other: Self) -> Self {
        f32(FloatBase::atan2(self.0, other.0))
    }

    /// Archimedes' constant.
    #[inline]
    fn pi() -> Self {
        f32(consts::PI)
    }

    /// 2.0 * pi.
    #[inline]
    fn two_pi() -> Self {
        f32(consts::PI + consts::PI)
    }

    /// pi / 2.0.
    #[inline]
    fn frac_pi_2() -> Self {
        f32(consts::FRAC_PI_2)
    }

    /// pi / 3.0.
    #[inline]
    fn frac_pi_3() -> Self {
        f32(consts::FRAC_PI_3)
    }

    /// pi / 4.0.
    #[inline]
    fn frac_pi_4() -> Self {
        f32(consts::FRAC_PI_4)
    }

    /// pi / 6.0.
    #[inline]
    fn frac_pi_6() -> Self {
        f32(consts::FRAC_PI_6)
    }

    /// pi / 8.0.
    #[inline]
    fn frac_pi_8() -> Self {
        f32(consts::FRAC_PI_8)
    }

    /// 1.0 / pi.
    #[inline]
    fn frac_1_pi() -> Self {
        f32(consts::FRAC_1_PI)
    }

    /// 2.0 / pi.
    #[inline]
    fn frac_2_pi() -> Self {
        f32(consts::FRAC_2_PI)
    }

    /// 2.0 / sqrt(pi).
    #[inline]
    fn frac_2_sqrt_pi() -> Self {
        f32(consts::FRAC_2_SQRT_PI)
    }

    /// Euler's number.
    #[inline]
    fn e() -> Self {
        f32(consts::E)
    }

    /// log2(e).
    #[inline]
    fn log2_e() -> Self {
        f32(consts::LOG2_E)
    }

    /// log10(e).
    #[inline]
    fn log10_e() -> Self {
        f32(consts::LOG10_E)
    }

    /// ln(2.0).
    #[inline]
    fn ln_2() -> Self {
        f32(consts::LN_2)
    }

    /// ln(10.0).
    #[inline]
    fn ln_10() -> Self {
        f32(consts::LN_10)
    }
}

impl ComplexField for f32 {
    type RealField = f32;

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
        f32(FloatBase::abs(self.0))
    }

    #[inline]
    fn modulus(self) -> Self::RealField {
        f32(FloatBase::abs(self.0))
    }

    #[inline]
    fn modulus_squared(self) -> Self::RealField {
        f32(self.0 * self.0)
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
        f32(FloatBase::recip(self.0))
    }

    #[inline]
    fn conjugate(self) -> Self {
        f32(self.0)
    }

    #[inline]
    fn scale(self, factor: Self::RealField) -> Self {
        f32(self.0 * factor.0)
    }

    #[inline]
    fn unscale(self, factor: Self::RealField) -> Self {
        f32(self.0 / factor.0)
    }

    #[inline]
    fn floor(self) -> Self {
        f32(FloatBase::floor(self.0))
    }

    #[inline]
    fn ceil(self) -> Self {
        f32(FloatBase::ceil(self.0))
    }

    #[inline]
    fn round(self) -> Self {
        f32(FloatBase::round(self.0))
    }

    #[inline]
    fn trunc(self) -> Self {
        f32(FloatBase::trunc(self.0))
    }

    #[inline]
    fn fract(self) -> Self {
        f32(FloatBase::fract(self.0))
    }

    #[inline]
    fn abs(self) -> Self {
        f32(FloatBase::abs(self.0))
    }

    #[inline]
    fn signum(self) -> Self {
        f32(Signed::signum(&self.0))
    }

    #[inline]
    fn mul_add(self, a: Self, b: Self) -> Self {
        f32(FloatBase::mul_add(self.0, a.0, b.0))
    }

    #[cfg(feature = "std")]
    #[inline]
    fn powi(self, n: i32) -> Self {
        f32(self.0.powi(n))
    }

    #[cfg(not(feature = "std"))]
    #[inline]
    fn powi(self, n: i32) -> Self {
        f32(FloatBase::powf(self.0, n as FloatBase))
    }

    #[inline]
    fn powf(self, n: Self) -> Self {
        f32(FloatBase::powf(self.0, n.0))
    }

    #[inline]
    fn powc(self, n: Self) -> Self {
        // Same as powf.
        f32(FloatBase::powf(self.0, n.0))
    }

    #[inline]
    fn sqrt(self) -> Self {
        f32(FloatBase::sqrt(self.0))
    }

    #[inline]
    fn try_sqrt(self) -> Option<Self> {
        if self >= Self::zero() {
            Some(f32(FloatBase::sqrt(self.0)))
        } else {
            None
        }
    }

    #[inline]
    fn exp(self) -> Self {
        f32(FloatBase::exp(self.0))
    }

    #[inline]
    fn exp2(self) -> Self {
        f32(FloatBase::exp2(self.0))
    }

    #[inline]
    fn exp_m1(self) -> Self {
        f32(FloatBase::exp_m1(self.0))
    }

    #[inline]
    fn ln_1p(self) -> Self {
        f32(FloatBase::ln_1p(self.0))
    }

    #[inline]
    fn ln(self) -> Self {
        f32(FloatBase::ln(self.0))
    }

    #[inline]
    fn log(self, base: Self) -> Self {
        f32(FloatBase::log(self.0, base.0))
    }

    #[inline]
    fn log2(self) -> Self {
        f32(FloatBase::log2(self.0))
    }

    #[inline]
    fn log10(self) -> Self {
        f32(FloatBase::log10(self.0))
    }

    #[inline]
    fn cbrt(self) -> Self {
        f32(FloatBase::cbrt(self.0))
    }

    #[inline]
    fn hypot(self, other: Self) -> Self::RealField {
        f32(FloatBase::hypot(self.0, other.0))
    }

    #[inline]
    fn sin(self) -> Self {
        f32(FloatBase::sin(self.0))
    }

    #[inline]
    fn cos(self) -> Self {
        f32(FloatBase::cos(self.0))
    }

    #[inline]
    fn tan(self) -> Self {
        f32(FloatBase::tan(self.0))
    }

    #[inline]
    fn asin(self) -> Self {
        f32(FloatBase::asin(self.0))
    }

    #[inline]
    fn acos(self) -> Self {
        f32(FloatBase::acos(self.0))
    }

    #[inline]
    fn atan(self) -> Self {
        f32(FloatBase::atan(self.0))
    }

    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        let vals = FloatBase::sin_cos(self.0);
        (f32(vals.0), f32(vals.1))
    }

    #[inline]
    fn sinh(self) -> Self {
        f32(FloatBase::sinh(self.0))
    }

    #[inline]
    fn cosh(self) -> Self {
        f32(FloatBase::cosh(self.0))
    }

    #[inline]
    fn tanh(self) -> Self {
        f32(FloatBase::tanh(self.0))
    }

    #[inline]
    fn asinh(self) -> Self {
        f32(FloatBase::asinh(self.0))
    }

    #[inline]
    fn acosh(self) -> Self {
        f32(FloatBase::acosh(self.0))
    }

    #[inline]
    fn atanh(self) -> Self {
        f32(FloatBase::atanh(self.0))
    }

    #[inline]
    fn is_finite(&self) -> bool {
        self.0.is_finite()
    }
}

impl SubsetOf<f32> for f32 {
    #[inline]
    fn to_superset(&self) -> f32 {
        self.0 as f32
    }

    #[inline]
    unsafe fn from_superset_unchecked(element: &f32) -> Self {
        f32(*element as FloatBase)
    }

    #[inline]
    fn is_in_subset(_: &f32) -> bool {
        true
    }
}

impl SubsetOf<f64> for f32 {
    #[inline]
    fn to_superset(&self) -> f64 {
        f64::from(self.0)
    }

    #[inline]
    unsafe fn from_superset_unchecked(element: &f64) -> Self {
        f32(*element as FloatBase)
    }

    #[inline]
    fn is_in_subset(_: &f64) -> bool {
        true
    }
}

impl SubsetOf<f32> for f32 {
    #[inline]
    fn to_superset(&self) -> f32 {
        f32(*self as FloatBase)
    }

    #[inline]
    unsafe fn from_superset_unchecked(element: &f32) -> Self {
        element.0 as Self
    }

    #[inline]
    fn is_in_subset(_: &f32) -> bool {
        true
    }
}

impl SubsetOf<f32> for f64 {
    #[inline]
    fn to_superset(&self) -> f32 {
        f32(*self as FloatBase)
    }

    #[inline]
    unsafe fn from_superset_unchecked(element: &f32) -> Self {
        f64::from(element.0)
    }

    #[inline]
    fn is_in_subset(_: &f32) -> bool {
        true
    }
}

impl SubsetOf<f32> for f32 {
    #[inline]
    fn to_superset(&self) -> f32 {
        f32(self.0 as FloatBase)
    }

    #[inline]
    unsafe fn from_superset_unchecked(element: &f32) -> Self {
        f32(element.0)
    }

    #[inline]
    fn is_in_subset(_: &f32) -> bool {
        true
    }
}

impl MeetSemilattice for f32 {
    #[inline]
    fn meet(&self, other: &Self) -> Self {
        if *self <= *other {
            *self
        } else {
            *other
        }
    }
}

impl JoinSemilattice for f32 {
    #[inline]
    fn join(&self, other: &Self) -> Self {
        if *self >= *other {
            *self
        } else {
            *other
        }
    }
}

impl Lattice for f32 {
    #[inline]
    fn meet_join(&self, other: &Self) -> (Self, Self) {
        if *self >= *other {
            (*other, *self)
        } else {
            (*self, *other)
        }
    }
}

impl<N2: Zero + SupersetOf<f32>> SubsetOf<Complex<N2>> for f32 {
    #[inline]
    fn to_superset(&self) -> Complex<N2> {
        Complex {
            re: N2::from_subset(self),
            im: N2::zero(),
        }
    }

    #[inline]
    unsafe fn from_superset_unchecked(element: &Complex<N2>) -> f32 {
        element.re.to_subset_unchecked()
    }

    #[inline]
    fn is_in_subset(c: &Complex<N2>) -> bool {
        c.re.is_in_subset() && c.im.is_zero()
    }
}
