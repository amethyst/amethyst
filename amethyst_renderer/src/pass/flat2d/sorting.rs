use super::Flat2DData;
use crate::{ActiveCamera, Camera};
use amethyst_core::{
    nalgebra::Point3,
    specs::{Join, Read, ReadStorage, System, Write},
    GlobalTransform,
};

enum SortData {
    TexOrder(usize, u32),
    DepthOrder(usize, f32),
}

impl SortData {
    #[inline]
    fn id(&self) -> usize {
        match self {
            &SortData::TexOrder(id, _) => id,
            &SortData::DepthOrder(id, _) => id,
        }
    }
    #[inline]
    fn set_id(&mut self, value: usize) {
        match self {
            SortData::TexOrder(id, _) => *id = value,
            SortData::DepthOrder(id, _) => *id = value,
        }
    }
}

fn cmp_sort_data(lhs: &SortData, rhs: &SortData) -> std::cmp::Ordering {
    use self::SortData::*;
    use std::cmp::Ordering;
    match (lhs, rhs) {
        (DepthOrder(_, a), DepthOrder(_, b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
        (TexOrder(_, a), TexOrder(_, b)) => a.cmp(b),
        (TexOrder(_, _), DepthOrder(_, _)) => Ordering::Less,
        (DepthOrder(_, _), TexOrder(_, _)) => Ordering::Greater,
    }
}

/// System used for sorting `DrawFlat2D` render buffer.
///
/// Uses centroid based depth sorting for transparent quads
/// and texture based sorting for opaque quads.
pub struct Flat2DDataSorter {
    scratch: Vec<SortData>,
}

impl Default for Flat2DDataSorter {
    fn default() -> Self {
        Self { scratch: vec![] }
    }
}

impl<'a> System<'a> for Flat2DDataSorter {
    type SystemData = (
        Write<'a, Vec<Flat2DData>>,
        ReadStorage<'a, GlobalTransform>,
        Option<Read<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
    );
    fn run(&mut self, (mut buffer, globals, active, cameras): Self::SystemData) {
        let camera: Option<&GlobalTransform> = active
            .and_then(|a| a.entity)
            .and_then(|e| globals.get(e))
            .or_else(|| (&cameras, &globals).join().map(|cg| cg.1).next());

        // no way to sort without a camera
        if let Some(camera) = camera {
            let camera_centroid = camera.0.transform_point(&Point3::origin());
            let camera_forward = camera.0.column(2).xyz();

            self.scratch.clear();
            self.scratch
                .extend(buffer.iter().enumerate().map(|(i, data)| {
                    if data.transparent {
                        SortData::DepthOrder(
                            i,
                            // calculate the camera z-depth of sprite centroids
                            camera_forward.dot(&(data.pos.xyz() - camera_centroid.coords)),
                        )
                    } else {
                        SortData::TexOrder(i, data.texture.id())
                    }
                }));
            self.scratch.sort_unstable_by(cmp_sort_data);

            // sort encoded buffer in place using computed ordering in scratch.
            // Note that the scratch buffer is modified in place to prevent
            // additional allocations, which destroys the original ordering data.
            // This is fine, because that data is later discarded.
            for i in 0..buffer.len() {
                let mut index = self.scratch[i].id();
                while (index as usize) < i {
                    index = self.scratch[index as usize].id();
                }
                self.scratch[i].set_id(index);
                buffer.swap(i, index as usize);
            }
        }
    }
}
