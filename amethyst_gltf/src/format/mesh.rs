use std::ops::Range;

use gltf;
use renderer::{AnimatedComboMeshCreator, Color, JointIds, JointWeights, MeshData, Normal,
               Position, Separate, Tangent, TexCoord};

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

        let positions = reader
            .read_positions()
            .map(|positions| match faces {
                Some(ref faces) => {
                    let vertices = positions.collect::<Vec<_>>();
                    faces
                        .iter()
                        .map(|i| Separate::<Position>::new(vertices[*i]))
                        .collect::<Vec<_>>()
                }
                None => positions
                    .map(|pos| Separate::<Position>::new(pos))
                    .collect(),
            })
            .ok_or(GltfError::MissingPositions)?;

        let bounds = primitive.bounding_box();
        let bounds = bounds.min..bounds.max;

        let colors = reader
            .read_colors(0)
            .map(|colors| colors.into_rgba_f32())
            .map(|colors| match faces {
                Some(ref faces) => {
                    let colors = colors.collect::<Vec<_>>();
                    faces
                        .iter()
                        .map(|i| Separate::<Color>::new(colors[*i]))
                        .collect()
                }
                None => colors.map(|color| Separate::<Color>::new(color)).collect(),
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
                .map(|i| Separate::<TexCoord>::new(flip_check(texs[*i], options.flip_v_coord)))
                .collect(),
            None => texs.into_iter()
                .map(|t| Separate::<TexCoord>::new(flip_check(t, options.flip_v_coord)))
                .collect(),
        });

        let normals = reader.read_normals().map(|normals| match faces {
            Some(ref faces) => {
                let normals = normals.collect::<Vec<_>>();
                faces
                    .iter()
                    .map(|i| Separate::<Normal>::new(normals[*i]))
                    .collect()
            }
            None => normals.map(|n| Separate::<Normal>::new(n)).collect(),
        });

        let tangents = reader.read_tangents().map(|tangents| match faces {
            Some(ref faces) => {
                let tangents = tangents.collect::<Vec<_>>();
                faces
                    .iter()
                    .map(|i| {
                        Separate::<Tangent>::new([
                            tangents[*i][0],
                            tangents[*i][1],
                            tangents[*i][2],
                        ])
                    })
                    .collect()
            }
            None => tangents
                .map(|t| Separate::<Tangent>::new([t[0], t[1], t[2]]))
                .collect(),
        });

        let joint_ids = reader
            .read_joints(0)
            .map(|joints| joints.into_u16())
            .map(|joints| match faces {
                Some(ref faces) => {
                    let joints = joints.collect::<Vec<_>>();
                    faces
                        .iter()
                        .map(|i| Separate::<JointIds>::new(joints[*i]))
                        .collect()
                }
                None => joints.map(|j| Separate::<JointIds>::new(j)).collect(),
            });
        trace!("Joint ids: {:?}", joint_ids);

        let joint_weights = reader
            .read_weights(0)
            .map(|weights| weights.into_f32())
            .map(|weights| match faces {
                Some(ref faces) => {
                    let weights = weights.collect::<Vec<_>>();
                    faces
                        .iter()
                        .map(|i| Separate::<JointWeights>::new(weights[*i]))
                        .collect()
                }
                None => weights.map(|w| Separate::<JointWeights>::new(w)).collect(),
            });
        trace!("Joint weights: {:?}", joint_weights);

        let material = primitive.material().index();

        let creator = AnimatedComboMeshCreator::new((
            positions,
            colors,
            tex_coord,
            normals,
            tangents,
            joint_ids,
            joint_weights,
        ));

        primitives.push((creator.into(), material, Some(bounds)));
    }

    Ok(primitives)
}

fn flip_check(uv: [f32; 2], flip_v: bool) -> [f32; 2] {
    if flip_v {
        [uv[0], 1. - uv[1]]
    } else {
        uv
    }
}
