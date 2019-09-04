//!
//! Geometry helper functionality.
use nalgebra::{Point3, Vector3};

/// A plane which can be intersected by a ray.
#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Plane {
    /// The forward normal of the plane.
    pub normal: Vector3<f32>,
    /// The origin/point of the plane.
    pub position: Vector3<f32>,
}
impl Plane {
    /// Create a new plane.
    pub fn new(normal: Vector3<f32>, position: Vector3<f32>) -> Self {
        Plane { normal, position }
    }

    /// Create a plane which is facing along the X-Axis at the provided coordinate.
    pub fn with_x(x: f32) -> Self {
        Self {
            normal: Vector3::new(1.0, 0.0, 0.0),
            position: Vector3::new(x, 0.0, 0.0),
        }
    }

    /// Create a plane which is facing along the Y-Axis at the provided coordinate.
    pub fn with_y(y: f32) -> Self {
        Self {
            normal: Vector3::new(0.0, 1.0, 0.0),
            position: Vector3::new(0.0, y, 0.0),
        }
    }

    /// Create a plane which is facing along the Z-Axis at the provided coordinate.
    pub fn with_z(z: f32) -> Self {
        Self {
            normal: Vector3::new(0.0, 0.0, 1.0),
            position: Vector3::new(0.0, 0.0, z),
        }
    }
}

/// A Ray represents and infinite half-line starting at `origin` and going in specified unit length `direction`.
#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Ray {
    /// The origin point of the ray
    pub origin: Point3<f32>,
    /// The normalized direction vector of the ray
    pub direction: Vector3<f32>,
}
impl Ray {
    /// Returns where a ray line segment intersects the provided plane.
    pub fn intersect_plane(&self, plane: &Plane) -> Point3<f32> {
        let diff = self.origin - plane.position;
        let prod1 = diff.coords.dot(&plane.normal);
        let prod2 = self.direction.dot(&plane.normal);
        let prod3 = prod1 / prod2;

        Point3::from(self.origin.coords - self.direction.scale(prod3))
    }

    /// Returns a `Point` along the ray at a distance `t` from it's origin.
    pub fn at_distance(&self, z: f32) -> Point3<f32> {
        self.origin + (self.direction * z)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use approx::assert_ulps_eq;

    #[test]
    #[allow(clippy::mistyped_literal_suffixes)]
    fn ray_intersect_plane() {
        let plane = Plane::with_z(0.0);
        assert_ulps_eq!(
            Ray {
                origin: Point3::new(0.020_277_506, -0.033_236_53, 51.794),
                direction: Vector3::new(0.179_559_51, -0.294_313_04, -0.938_689_65),
            }
            .intersect_plane(&plane),
            Point3::new(9.927_818, -16.272_524, 0.0)
        );

        assert_ulps_eq!(
            Ray {
                origin: Point3::new(-0.003_106_177, 0.034_074_64, 0.799_999_95),
                direction: Vector3::new(-0.029_389_05, 0.322_396_73, -0.946_148_3),
            }
            .intersect_plane(&plane),
            Point3::new(-0.027_955_6, 0.306_671_83, 0.0)
        );
    }

    #[test]
    fn at_distance() {
        assert_ulps_eq!(
            Ray {
                origin: Point3::new(0.020_277_506, -0.033_236_53, 51.794),
                direction: Vector3::new(0.179_559_51, -0.294_313_04, -0.938_689_65),
            }
            .at_distance(5.0),
            Point3::new(0.918_075_1, -1.504_801_8, 47.100_55)
        );
    }
}
