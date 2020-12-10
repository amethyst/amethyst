//!
//! Geometry helper functionality.
use nalgebra::{one, zero, Point3, RealField, Vector3};

/// A plane which can be intersected by a ray.
#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Plane<T>
where
    T: RealField,
{
    /// The plane described as x,y,z normal
    normal: Vector3<T>,
    /// dot product of the point and normal, representing the plane position
    bias: T,
}
impl<T> Plane<T>
where
    T: RealField,
{
    /// Create a new `Plane`.
    pub fn new(normal: Vector3<T>, bias: T) -> Self {
        Plane { normal, bias }
    }

    /// Create a new `Plane` from a point normal representation
    pub fn from_point_normal(point: &Point3<T>, normal: &Vector3<T>) -> Self {
        let normalized = normal.normalize();
        Self {
            normal: Vector3::new(normalized.x, normalized.y, normalized.z),
            bias: point.coords.dot(&normalized),
        }
    }

    /// Create a new `Plane` from a point normal representation
    pub fn from_point_vectors(point: &Point3<T>, v1: &Vector3<T>, v2: &Vector3<T>) -> Self {
        Self::from_point_normal(point, &v1.cross(&v2))
    }

    /// Create a `Plane` which is facing along the X-Axis at the provided coordinate.
    pub fn with_x(x: T) -> Self {
        Self::from_point_normal(
            &Point3::new(x, zero(), zero()),
            &Vector3::new(one(), zero(), zero()),
        )
    }

    /// Create a `Plane` which is facing along the Y-Axis at the provided coordinate.
    pub fn with_y(y: T) -> Self {
        Self::from_point_normal(
            &Point3::new(zero(), y, zero()),
            &Vector3::new(zero(), one(), zero()),
        )
    }

    /// Create a `Plane` which is facing along the Z-Axis at the provided coordinate.
    pub fn with_z(z: T) -> Self {
        Self::from_point_normal(
            &Point3::new(zero(), zero(), z),
            &Vector3::new(zero(), zero(), one()),
        )
    }

    /// This `Plane` normal
    pub fn normal(&self) -> &Vector3<T> {
        &self.normal
    }

    /// Normalized representation of this `Plane`
    pub fn normalize(&self) -> Self {
        let distance = self.normal.magnitude();
        Self {
            normal: self.normal / distance,
            bias: self.bias / distance,
        }
    }

    /// Returns the dot product of this `Plane` and a provided `Point3`
    pub fn dot_point(&self, point: &Point3<T>) -> T {
        self.normal.x * point.x + self.normal.y * point.y + self.normal.z * point.z + self.bias
    }

    /// Returns the dot product of this `Plane` and a provided `Vector3`
    pub fn dot(&self, point: &Vector3<T>) -> T {
        self.normal.x * point.x + self.normal.y * point.y + self.normal.z * point.z
    }

    /// Returns the dot product of this `Plane` with another `Plane`
    pub fn dot_plane(&self, plane: &Plane<T>) -> T {
        self.normal.x * plane.normal.x
            + self.normal.y * plane.normal.y
            + self.normal.z * plane.normal.z
            + self.bias * plane.bias
    }

    /// Returns the intersection distance of the provided line given a point and direction, or `None` if none occurs.
    pub fn intersect_line(&self, point: &Point3<T>, direction: &Vector3<T>) -> Option<T> {
        let fv = self.dot(direction);
        let distance = self.dot_point(point) / fv;
        if fv.abs() > T::min_value() {
            Some(distance)
        } else {
            None
        }
    }

    /// Returns the intersection distance of the provided `Ray`, or `None` if none occurs.
    pub fn intersect_ray(&self, ray: &Ray<T>) -> Option<T> {
        self.intersect_line(&ray.origin, &ray.direction)
    }
}

/// A Ray represents and infinite half-line starting at `origin` and going in specified unit length `direction`.
#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Ray<T>
where
    T: RealField,
{
    /// The origin point of the ray
    pub origin: Point3<T>,
    /// The normalized direction vector of the ray
    pub direction: Vector3<T>,
}
impl<T> Ray<T>
where
    T: RealField,
{
    /// Returns the distance along the ray which intersects with the provided `Plane`1
    pub fn intersect_plane(&self, plane: &Plane<T>) -> Option<T> {
        plane.intersect_ray(self)
    }

    /// Returns a `Point` along the ray at a distance `t` from it's origin.
    pub fn at_distance(&self, z: T) -> Point3<T> {
        self.origin - (self.direction * z)
    }
}

#[cfg(test)]
pub mod tests {
    use approx::{assert_relative_eq, assert_ulps_eq};

    use super::*;

    #[test]
    #[allow(clippy::mistyped_literal_suffixes)]
    fn ray_intersect_plane() {
        let plane = Plane::<f32>::with_z(0.0);

        let ray = Ray {
            origin: Point3::new(0.020_277_506, -0.033_236_53, 51.794),
            direction: Vector3::new(0.179_559_51, -0.294_313_04, -0.938_689_65),
        };
        let distance = ray.intersect_plane(&plane).unwrap();
        let point = ray.at_distance(distance);
        assert_ulps_eq!(point, Point3::new(9.927_818, -16.272_524, 0.0));

        let ray = Ray {
            origin: Point3::new(-0.003_106_177, 0.034_074_64, 0.799_999_95),
            direction: Vector3::new(-0.029_389_05, 0.322_396_73, -0.946_148_3),
        };
        let distance = ray.intersect_plane(&plane).unwrap();
        let point = ray.at_distance(distance);
        assert_ulps_eq!(point, Point3::new(-0.027_955_6, 0.306_671_83, 0.0));
    }

    #[test]
    fn at_distance() {
        assert_relative_eq!(
            Ray {
                origin: Point3::new(0.0, 0.0, 50.0),
                direction: Vector3::new(0.2, -0.3, -0.9),
            }
            .at_distance(5.0),
            Point3::new(-1., 1.5, 54.5)
        )
    }
}
