//! Various helper iterators for efficiently iterating in coordinate encoded spaces in a locality-friendly fashion.

use crate::morton;
use amethyst_core::math::Point3;

/// Cubic region stored and handled as a Morton value.
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Hash)]
pub struct MortonRegion {
    min: u32,
    max: u32,
}
impl MortonRegion {
    /// Create a new `MortonRegion` iterator.
    pub fn new(min: u32, max: u32) -> Self {
        Self { min, max }
    }

    /// Check if this `MortonRegion` contains a given morton coordinate.
    #[inline]
    pub fn contains(self, morton: u32) -> bool {
        let m1 = morton::min(self.min, morton) != morton;
        let m2 = morton::max(self.max, morton) != morton;

        m1 && m2
    }
}
impl PartialOrd for MortonRegion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for MortonRegion {
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
impl AsRef<MortonRegion> for MortonRegion {
    fn as_ref(&self) -> &Self {
        self
    }
}
impl<T: AsRef<Region>> From<T> for MortonRegion {
    fn from(region: T) -> Self {
        let region = region.as_ref();
        Self {
            min: morton::encode(region.min.x, region.min.y, region.min.z),
            max: morton::encode(region.max.x, region.max.y, region.max.z),
        }
    }
}

/// 3D cubic region space of a 3D coordinate space,
#[derive(Copy, Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Region {
    /// The "lower-right" coordinate of this `Region`.
    pub min: Point3<u32>,
    /// The "Upper-left" coordinate of this `Region`.
    pub max: Point3<u32>,
}

impl AsRef<Region> for Region {
    fn as_ref(&self) -> &Self {
        self
    }
}
impl Region {
    /// Create a new `Region` with the given top-left and bottom-right cubic coordinates.
    pub fn new(min: Point3<u32>, max: Point3<u32>) -> Self {
        Self { min, max }
    }

    /// Returns an empty `Region`
    pub fn empty() -> Self {
        Self {
            min: Point3::new(0, 0, 0),
            max: Point3::new(0, 0, 0),
        }
    }

    /// Check if this cube contains the provided coordinate.
    #[inline]
    pub fn contains(&self, target: &Point3<u32>) -> bool {
        (target.x >= self.min.x
            && target.x <= self.max.x
            && target.y >= self.min.y
            && target.y <= self.max.y
            && target.z >= self.min.z
            && target.z <= self.max.z)
    }

    /// Check if this `Region` intersects with the provided `Region`
    #[inline]
    pub fn intersect(&self, other: &Self) -> bool {
        (self.min.x <= other.max.x && self.max.x >= other.min.x)
            && (self.min.y <= other.max.y && self.max.y >= other.min.y)
            && (self.min.z <= other.max.z && self.max.z >= other.min.z)
    }

    /// Calculate the volume of this bounding box volume.
    pub fn volume(&self) -> u32 {
        (self.max.x - self.min.x) * (self.max.y - self.min.y) * ((self.max.z - self.min.z) + 1)
    }

    /// Create a linear iterator across this region.
    pub fn iter(&self) -> RegionLinearIter {
        RegionLinearIter::new(*self)
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
pub struct RegionLinearIter {
    track: Point3<u32>,
    region: Region,
}
impl RegionLinearIter {
    /// Create a new iterator.
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

        if self.track.z > self.region.max.z {
            return None;
        }

        if self.track.x >= self.region.max.x {
            self.track.y += 1;
            self.track.x = self.region.min.x;
        } else {
            self.track.x += 1;
            return Some(ret);
        }

        if self.track.y >= self.region.max.y {
            self.track.z += 1;

            self.track.y = self.region.min.y;
        }

        Some(ret)
    }
}

#[cfg(test)]
#[allow(clippy::shadow_unrelated)]
mod tests {
    use super::*;

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
