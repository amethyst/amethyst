use std::ops::Range;

use amethyst_assets::{
    erased_serde::private::serde::{de, de::SeqAccess, ser::SerializeSeq},
    prefab::{
        register_component_type,
        serde_diff::{ApplyContext, DiffContext},
        SerdeDiff,
    },
    Handle,
};
use amethyst_core::math::{convert, Point3, Vector3};
use amethyst_rendy::{visibility::BoundingSphere, Material, Mesh};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

/// MeshHandle is a component that will handle the fact that we attach
/// a mesh to an entity as an asset handle that will later be loaded.
#[derive(Serialize, Deserialize, TypeUuid, Clone)]
#[uuid = "34310974-b4cf-4dc2-a81b-40627c20543a"]
pub struct MeshHandle(pub Handle<Mesh>);
impl Default for MeshHandle {
    fn default() -> Self {
        unimplemented!()
    }
}

impl SerdeDiff for MeshHandle {
    fn diff<'a, S: SerializeSeq>(
        &self,
        _ctx: &mut DiffContext<'a, S>,
        _other: &Self,
    ) -> Result<bool, <S as SerializeSeq>::Error> {
        unimplemented!()
    }

    fn apply<'de, A>(
        &mut self,
        _seq: &mut A,
        _ctx: &mut ApplyContext,
    ) -> Result<bool, <A as SeqAccess<'de>>::Error>
    where
        A: de::SeqAccess<'de>,
    {
        unimplemented!()
    }
}

register_component_type!(MeshHandle);

/// MaterialHandle is a component that will handle the fact that we attach
/// a material handle to an entity as an asset handle that will later be loaded.
#[derive(Serialize, Deserialize, TypeUuid, Clone)]
#[uuid = "40a2d8f7-54e8-46ad-b668-66d759feb806"]
pub struct MaterialHandle(pub Handle<Material>);
impl Default for MaterialHandle {
    fn default() -> Self {
        unimplemented!()
    }
}

impl SerdeDiff for MaterialHandle {
    fn diff<'a, S: SerializeSeq>(
        &self,
        _ctx: &mut DiffContext<'a, S>,
        _other: &Self,
    ) -> Result<bool, <S as SerializeSeq>::Error> {
        unimplemented!()
    }

    fn apply<'de, A>(
        &mut self,
        _seq: &mut A,
        _ctx: &mut ApplyContext,
    ) -> Result<bool, <A as SeqAccess<'de>>::Error>
    where
        A: de::SeqAccess<'de>,
    {
        unimplemented!()
    }
}

register_component_type!(MaterialHandle);

/// A GLTF node extent
#[derive(Serialize, Deserialize, TypeUuid, Clone, Debug)]
#[uuid = "e569daf6-f391-4235-b75b-28d55b26b0a1"]
pub struct GltfNodeExtent {
    /// The beginning of this extent
    pub start: Point3<f32>,
    /// The end of this extent
    pub end: Point3<f32>,
}

impl SerdeDiff for GltfNodeExtent {
    fn diff<'a, S: SerializeSeq>(
        &self,
        _ctx: &mut DiffContext<'a, S>,
        _other: &Self,
    ) -> Result<bool, <S as SerializeSeq>::Error> {
        unimplemented!()
    }

    fn apply<'de, A>(
        &mut self,
        _seq: &mut A,
        _ctx: &mut ApplyContext,
    ) -> Result<bool, <A as SeqAccess<'de>>::Error>
    where
        A: de::SeqAccess<'de>,
    {
        unimplemented!()
    }
}

register_component_type!(GltfNodeExtent);

impl Default for GltfNodeExtent {
    fn default() -> Self {
        Self {
            start: Point3::from(Vector3::from_element(std::f32::MAX)),
            end: Point3::from(Vector3::from_element(std::f32::MIN)),
        }
    }
}

impl GltfNodeExtent {
    /// Extends this to include the input range.
    pub fn extend_range(&mut self, other: &Range<[f32; 3]>) {
        for i in 0..3 {
            if other.start[i] < self.start[i] {
                self.start[i] = other.start[i];
            }
            if other.end[i] > self.end[i] {
                self.end[i] = other.end[i];
            }
        }
    }

    /// Extends this to include the provided extent.
    pub fn extend(&mut self, other: &GltfNodeExtent) {
        for i in 0..3 {
            if other.start[i] < self.start[i] {
                self.start[i] = other.start[i];
            }
            if other.end[i] > self.end[i] {
                self.end[i] = other.end[i];
            }
        }
    }

    /// Returns the centroid of this extent
    pub fn centroid(&self) -> Point3<f32> {
        (self.start + self.end.coords) / 2.
    }

    /// Returns the 3 dimensional distance between the start and end of this.
    pub fn distance(&self) -> Vector3<f32> {
        self.end - self.start
    }

    /// Determines if this extent is valid.
    pub fn valid(&self) -> bool {
        for i in 0..3 {
            if self.start[i] > self.end[i] {
                return false;
            }
        }
        true
    }
}

impl Into<BoundingSphere> for GltfNodeExtent {
    fn into(self) -> BoundingSphere {
        BoundingSphere {
            center: convert(self.centroid()),
            radius: convert(self.distance().magnitude() * 0.5),
        }
    }
}

impl From<Range<[f32; 3]>> for GltfNodeExtent {
    fn from(range: Range<[f32; 3]>) -> Self {
        GltfNodeExtent {
            start: Point3::from(range.start),
            end: Point3::from(range.end),
        }
    }
}
