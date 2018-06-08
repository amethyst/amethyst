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
    use gltf_utils::PrimitiveIterators;

    let mut primitives = vec![];

    for primitive in mesh.primitives() {
        let faces = primitive.indices_u32(buffers).map(|mut iter| {
            let mut faces = vec![];
            while let (Some(a), Some(b), Some(c)) = (iter.next(), iter.next(), iter.next()) {
                faces.push(a as usize);
                faces.push(b as usize);
                faces.push(c as usize);
            }
            faces
        });

        let positions = primitive
            .positions(buffers)
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

        let bounds = primitive
            .position_bounds()
            .map(|bound| bound.min..bound.max);

        let colors = primitive
            .colors_rgba_f32(0, 1., buffers)
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

        let tex_coord = match primitive.tex_coords_f32(0, buffers) {
            Some(tex_coords) => Some(tex_coords.collect::<Vec<[f32; 2]>>()),
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

        let normals = primitive.normals(buffers).map(|normals| match faces {
            Some(ref faces) => {
                let normals = normals.collect::<Vec<_>>();
                faces
                    .iter()
                    .map(|i| Separate::<Normal>::new(normals[*i]))
                    .collect()
            }
            None => normals.map(|n| Separate::<Normal>::new(n)).collect(),
        });

        let tangents = primitive.tangents(buffers).map(|tangents| match faces {
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

        let joint_ids = primitive.joints_u16(0, buffers).map(|joints| match faces {
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

        let joint_weights = primitive
            .weights_f32(0, buffers)
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

        primitives.push((creator.into(), material, bounds));
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
