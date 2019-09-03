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

/// A ray structure providing a position and direction.
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

    /// Returns the ray `Point` at the given distance
    pub fn at_distance(&self, z: f32) -> Point3<f32> {
        self.origin + (self.direction * z)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use approx::assert_ulps_eq;

    #[test]
    fn ray_intersect_plane() {
        let plane = Plane::with_z(0.0);
        assert_ulps_eq!(
            Ray {
                origin: Point3::new(0.020277506, -0.03323653, 51.794),
                direction: Vector3::new(0.17955951, -0.29431304, -0.93868965),
            }
            .intersect_plane(&plane),
            Point3::new(9.927818, -16.272524, 0.0)
        );

        assert_ulps_eq!(
            Ray {
                origin: Point3::new(-0.003106177, 0.03407464, 0.79999995),
                direction: Vector3::new(-0.02938905, 0.32239673, -0.9461483),
            }
            .intersect_plane(&plane),
            Point3::new(-0.0279556, 0.30667183, 0.0)
        );
    }

    #[test]
    fn at_distance() {
        assert_ulps_eq!(
            Ray {
                origin: Point3::new(0.020277506, -0.03323653, 51.794),
                direction: Vector3::new(0.17955951, -0.29431304, -0.93868965),
            }
            .at_distance(5.0),
            Point3::new(0.9180751, -1.5048018, 47.10055)
        );
    }
}
