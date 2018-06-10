use std::ops::Range;

use gltf;
use renderer::{AnimatedComboMeshCreator, Attribute, MeshData, Separate};

use super::{Buffers, GltfError};
use GltfSceneOptions;

pub fn load_mesh(
    mesh: &gltf::Mesh,
    buffers: &Buffers,
    options: &GltfSceneOptions,
) -> Result<Vec<(MeshData, Option<usize>, Option<Range<[f32; 3]>>)>, GltfError> {
    // TODO: simplify loading here when we have support for indexed meshes
    // All attributes can then be mapped directly instead of using faces to unwind the indexing

    let mut primitives = vec![];

    for primitive in mesh.primitives() {
        let reader = primitive.reader(|buffer| buffers.buffer(&buffer));

        let faces = reader
            .read_indices()
            .map(|indices| indices.into_u32())
            .map(|mut indices| {
                let mut faces = vec![];
                while let (Some(a), Some(b), Some(c)) =
                    (indices.next(), indices.next(), indices.next())
                {
                    faces.push(a as usize);
                    faces.push(b as usize);
                    faces.push(c as usize);
                }
                faces
            });

        let mut positions = reader
            .read_positions()
            .map(|positions| match faces {
                Some(ref faces) => {
                    let vertices = positions.collect::<Vec<_>>();
                    faces.iter().map(|i| vertices[*i]).collect::<Vec<_>>()
                }
                None => positions.collect(),
            })
            .ok_or(GltfError::MissingPositions)?;

        let normals = reader
            .read_normals()
            .map(|normals| match faces {
                Some(ref faces) => {
                    let normals = normals.collect::<Vec<_>>();
                    faces.iter().map(|i| normals[*i]).collect()
                }
                None => normals.collect(),
            })
            .unwrap_or_else(|| {
                use core::cgmath::Point3;
                use std::iter::once;
                let f = faces
                    .as_ref()
                    .map(|f| f.clone())
                    .unwrap_or_else(|| (0..positions.len()).collect::<Vec<_>>());
                f.chunks(3)
                    .flat_map(|chunk| {
                        let a = Point3::from(positions[chunk[0]]);
                        let ab = Point3::from(positions[chunk[1]]) - a;
                        let ac = Point3::from(positions[chunk[2]]) - a;
                        let normal: [f32; 3] = ab.cross(ac).into();
                        once(normal.clone())
                            .chain(once(normal.clone()))
                            .chain(once(normal))
                    })
                    .collect::<Vec<_>>()
            });

        let bounds = primitive.bounding_box();
        let bounds = bounds.min..bounds.max;

        let colors = reader
            .read_colors(0)
            .map(|colors| colors.into_rgba_f32())
            .map(|colors| match faces {
                Some(ref faces) => {
                    let colors = colors.collect::<Vec<_>>();
                    faces.iter().map(|i| colors[*i]).collect()
                }
                None => colors.collect(),
            });

        let tex_coord = match reader.read_tex_coords(0) {
            Some(tex_coords) => Some(tex_coords.into_f32().collect::<Vec<[f32; 2]>>()),
            None => match options.generate_tex_coords {
                Some((u, v)) => Some((0..positions.len()).map(|_| [u, v]).collect()),
                None => None,
            },
        }.map(|texs| match faces {
            Some(ref faces) => faces
                .iter()
                .map(|i| flip_check(texs[*i], options.flip_v_coord))
                .collect(),
            None => texs.into_iter()
                .map(|t| flip_check(t, options.flip_v_coord))
                .collect(),
        });
        
        let tangents = reader.read_tangents().map(|tangents| match faces {
            Some(ref faces) => {
                let tangents = tangents.collect::<Vec<_>>();
                faces
                    .iter()
                    .map(|i| [tangents[*i][0], tangents[*i][1], tangents[*i][2]])
                    .collect()
            }
            None => tangents.map(|t| [t[0], t[1], t[2]]).collect(),
        });

        let joint_ids = reader
            .read_joints(0)
            .map(|joints| joints.into_u16())
            .map(|joints| match faces {
                Some(ref faces) => {
                    let joints = joints.collect::<Vec<_>>();
                    faces.iter().map(|i| joints[*i]).collect()
                }
                None => joints.collect(),
            });
        trace!("Joint ids: {:?}", joint_ids);

        let joint_weights = reader
            .read_weights(0)
            .map(|weights| weights.into_f32())
            .map(|weights| match faces {
                Some(ref faces) => {
                    let weights = weights.collect::<Vec<_>>();
                    faces.iter().map(|i| weights[*i]).collect()
                }
                None => weights.collect(),
            });
        trace!("Joint weights: {:?}", joint_weights);

        let material = primitive.material().index();

        let creator = AnimatedComboMeshCreator::new((
            cast_attribute(positions),
            colors.map(cast_attribute),
            tex_coord.map(cast_attribute),
            Some(cast_attribute(normals)),
            tangents.map(cast_attribute),
            joint_ids.map(cast_attribute),
            joint_weights.map(cast_attribute),
        ));

        primitives.push((creator.into(), material, Some(bounds)));
    }

    Ok(primitives)
}

fn cast_attribute<T>(mut old: Vec<T::Repr>) -> Vec<Separate<T>>
where
    T: Attribute,
{
    let new = unsafe {
        Vec::from_raw_parts(
            old.as_mut_ptr() as *mut Separate<T>,
            old.len(),
            old.capacity(),
        )
    };
    ::std::mem::forget(old);
    new
}

fn flip_check(uv: [f32; 2], flip_v: bool) -> [f32; 2] {
    if flip_v {
        [uv[0], 1. - uv[1]]
    } else {
        uv
    }
}
