//! Various helper iterators for efficiently iterating in coordinate encoded spaces in a locality-friendly fashion.

use amethyst_core::math::Point3;

use crate::morton;

/// Axis aligned quantized region of space represented in tile coordinates of `u32`. This behaves
/// like a bounding box volume with `min` and `max` coordinates for iteration. This regions limits are *inclusive*,
/// in that it considers both min and max values as being inside the region.
///
/// The values of this region are stored and computed as morton values instead of `Vector3` values, allowing for
/// fast BMI2 instrinsic use for iteration and comparison.
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Hash)]
pub struct MortonRegion {
    min: u32,
    max: u32,
}
impl MortonRegion {
    /// Create a new `MortonRegion` region.
    #[must_use]
    pub fn new(min: u32, max: u32) -> Self {
        Self { min, max }
    }

    /// Check if this `MortonRegion` contains a given morton coordinate.
    #[inline]
    #[must_use]
    pub fn contains(self, morton: u32) -> bool {
        let target = morton::decode(morton);
        let min = morton::decode(self.min);
        let max = morton::decode(self.max);

        target.0 >= min.0
            && target.0 <= max.0
            && target.1 >= min.1
            && target.1 <= max.1
            && target.2 >= min.2
            && target.2 <= max.2
    }
}
impl PartialOrd for MortonRegion {
    #[must_use]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for MortonRegion {
    #[must_use]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.min == other.min && self.max == other.max {
            std::cmp::Ordering::Equal
        } else if morton::min(self.min, other.min) == self.min {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }
}
impl From<Region> for MortonRegion {
    #[must_use]
    fn from(region: Region) -> Self {
        Self {
            min: morton::encode(region.min.x, region.min.y, region.min.z),
            max: morton::encode(region.max.x, region.max.y, region.max.z),
        }
    }
}
impl<'a> From<&'a Region> for MortonRegion {
    #[must_use]
    fn from(region: &'a Region) -> Self {
        Self {
            min: morton::encode(region.min.x, region.min.y, region.min.z),
            max: morton::encode(region.max.x, region.max.y, region.max.z),
        }
    }
}

/// Axis aligned quantized region of space represented in tile coordinates of `u32`. This behaves
/// like a bounding box volume with `min` and `max` coordinates for iteration. The lower (min) coordinates
/// are inclusive and the upper (max) coordinates are exclusive.
#[derive(Copy, Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Region {
    /// The "lower-right" coordinate of this `Region`.
    pub min: Point3<u32>,
    /// The "Upper-left" coordinate of this `Region`.
    pub max: Point3<u32>,
}

impl Region {
    /// Create a new `Region` with the given top-left and bottom-right cubic coordinates.
    #[must_use]
    pub fn new(min: Point3<u32>, max: Point3<u32>) -> Self {
        Self { min, max }
    }

    /// Returns an empty `Region`
    #[must_use]
    pub fn empty() -> Self {
        Self {
            min: Point3::new(0, 0, 0),
            max: Point3::new(0, 0, 0),
        }
    }

    /// Check if this cube contains the provided coordinate.
    #[inline]
    #[must_use]
    pub fn contains(&self, target: &Point3<u32>) -> bool {
        target.x >= self.min.x
            && target.x < self.max.x
            && target.y >= self.min.y
            && target.y < self.max.y
            && target.z >= self.min.z
            && target.z < self.max.z
    }

    /// Check if this `Region` intersects with the provided `Region`
    #[inline]
    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        (self.min.x < other.max.x && self.max.x > other.min.x)
            && (self.min.y < other.max.y && self.max.y > other.min.y)
            && (self.min.z < other.max.z && self.max.z > other.min.z)
    }

    /// Calculate the volume of this bounding box volume.
    #[must_use]
    pub fn volume(&self) -> u32 {
        (self.max.x - self.min.x) * (self.max.y - self.min.y) * (self.max.z - self.min.z)
    }

    /// Create a linear iterator across this region.
    #[must_use]
    pub fn iter(&self) -> RegionLinearIter {
        RegionLinearIter::new(*self)
    }
}

impl<'a> IntoIterator for &'a Region {
    type Item = Point3<u32>;
    type IntoIter = RegionLinearIter;

    #[must_use]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: AsRef<MortonRegion>> From<T> for Region {
    fn from(region: T) -> Self {
        let region = region.as_ref();
        let min = morton::decode(region.min);
        let max = morton::decode(region.max);

        Self {
            min: Point3::new(min.0, min.1, min.2),
            max: Point3::new(max.0, max.1, max.2),
        }
    }
}

/// Linear iterator across a 3D coordinate space.
/// This iterator is inclusive of minimum and maximum coordinates.
pub struct RegionLinearIter {
    track: Point3<u32>,
    region: Region,
}
impl RegionLinearIter {
    /// Create a new iterator.
    #[must_use]
    pub fn new(region: Region) -> Self {
        Self {
            region,
            track: region.min,
        }
    }
}
impl Iterator for RegionLinearIter {
    type Item = Point3<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.track;

        if self.track.z >= self.region.max.z {
            return None;
        }

        self.track.x += 1;
        if self.track.x >= self.region.max.x {
            self.track.x = self.region.min.x;
            self.track.y += 1;
            if self.track.y >= self.region.max.y {
                self.track.y = self.region.min.y;
                self.track.z += 1;
            }
        }

        Some(ret)
    }
}

#[cfg(test)]
#[allow(clippy::shadow_unrelated)]
mod tests {
    use super::*;

    #[test]
    fn morton_region_edge_cases() {
        let region = Region::new(Point3::new(0, 0, 0), Point3::new(64, 64, 64));
        let morton_region = MortonRegion::new(
            morton::encode(region.min.x, region.min.y, region.min.z),
            morton::encode(region.max.x, region.max.y, region.max.z),
        );

        // min/max corners
        assert!(morton_region.contains(morton::encode(0, 0, 0)));
        assert!(morton_region.contains(morton::encode(64, 64, 64)));

        // edges
        assert!(morton_region.contains(morton::encode(64, 0, 0)));
        assert!(morton_region.contains(morton::encode(0, 64, 0)));
        assert!(morton_region.contains(morton::encode(0, 0, 64)));

        assert!(morton_region.contains(morton::encode(64, 64, 0)));
        assert!(morton_region.contains(morton::encode(0, 64, 64)));
        assert!(morton_region.contains(morton::encode(64, 0, 64)));
    }

    #[test]
    fn region_edge_cases() {
        let region = Region::new(Point3::new(0, 0, 0), Point3::new(64, 64, 64));

        // min/max corners
        assert!(region.contains(&Point3::new(0, 0, 0)));
        assert!(region.contains(&Point3::new(63, 63, 63)));
        assert!(!region.contains(&Point3::new(64, 64, 64)));

        // edges
        assert!(region.contains(&Point3::new(63, 0, 0)));
        assert!(region.contains(&Point3::new(0, 63, 0)));
        assert!(region.contains(&Point3::new(0, 0, 63)));

        assert!(region.contains(&Point3::new(63, 63, 0)));
        assert!(region.contains(&Point3::new(0, 63, 63)));
        assert!(region.contains(&Point3::new(63, 0, 63)));
    }

    #[test]
    fn region_iterator() {
        let region = Region::new(Point3::new(10, 20, 30), Point3::new(12, 22, 32));
        let expected_points = [
            Point3::new(10, 20, 30),
            Point3::new(11, 20, 30),
            Point3::new(10, 21, 30),
            Point3::new(11, 21, 30),
            Point3::new(10, 20, 31),
            Point3::new(11, 20, 31),
            Point3::new(10, 21, 31),
            Point3::new(11, 21, 31),
        ];
        let mut count = 0;
        for (i, point) in region.into_iter().enumerate() {
            count += 1;
            assert_eq!(point, expected_points[i]);
        }
        assert_eq!(count, 8);
    }

    #[test]
    fn region_volume() {
        let region = Region::new(Point3::new(0, 0, 0), Point3::new(0, 0, 0));
        assert_eq!(region.volume(), 0);
        let region = Region::new(Point3::new(10, 20, 30), Point3::new(10, 20, 30));
        assert_eq!(region.volume(), 0);
        let region = Region::new(Point3::new(10, 10, 10), Point3::new(10, 15, 15));
        assert_eq!(region.volume(), 0);
        let region = Region::new(Point3::new(10, 10, 10), Point3::new(20, 20, 20));
        assert_eq!(region.volume(), 1000);
    }

    #[test]
    fn region_morton_match() {
        let region = Region::new(Point3::new(0, 0, 0), Point3::new(64, 64, 64));
        let morton_region = MortonRegion::new(
            morton::encode(region.min.x, region.min.y, region.min.z),
            morton::encode(region.max.x, region.max.y, region.max.z),
        );
        let morton_region_from = region.into();

        assert_eq!(morton_region, morton_region_from);

        let d = morton::decode(morton_region.min);
        assert_eq!(region.min.x, d.0);
        assert_eq!(region.min.y, d.1);
        assert_eq!(region.min.z, d.2);

        let d = morton::decode(morton_region.max);
        assert_eq!(region.max.x, d.0);
        assert_eq!(region.max.y, d.1);
        assert_eq!(region.max.z, d.2);

        let c = Point3::new(5, 5, 5);
        assert_eq!(
            morton_region.contains(morton::encode(c.x, c.y, c.z)),
            region.contains(&c)
        );

        let c = Point3::new(11, 62, 0);
        assert_eq!(
            morton_region.contains(morton::encode(c.x, c.y, c.z)),
            region.contains(&c)
        );

        let c = Point3::new(128, 55, 565);
        assert_eq!(
            morton_region.contains(morton::encode(c.x, c.y, c.z)),
            region.contains(&c)
        );
    }
}
