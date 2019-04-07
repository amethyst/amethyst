use std::ops::Range;

use amethyst_error::Error;
use amethyst_rendy::{
    rendy::mesh::{MeshBuilder, PosNormTangTex},
    skinning::PosNormTangTexJoint,
};
use log::trace;

use super::Buffers;
use crate::{error, GltfSceneOptions};

pub fn load_mesh(
    mesh: &gltf::Mesh<'_>,
    buffers: &Buffers,
    options: &GltfSceneOptions,
) -> Result<Vec<(MeshBuilder<'static>, Option<usize>, Range<[f32; 3]>)>, Error> {
    trace!("Loading mesh");
    let mut primitives = vec![];

    for primitive in mesh.primitives() {
        trace!("Loading mesh primitive");
        let reader = primitive.reader(|buffer| buffers.buffer(&buffer));

        trace!("Loading faces");
        let faces = reader
            .read_indices()
            .map(|indices| indices.into_u32().collect::<Vec<_>>())
            .map(|mut indices| {
                // make sure that it's divisible by 3
                indices.truncate(indices.len() / 3 * 3);
                indices
            });

        trace!("Loading positions");
        let positions = reader
            .read_positions()
            .map(|positions| match faces {
                Some(ref faces) => {
                    let vertices = positions.collect::<Vec<_>>();
                    faces
                        .iter()
                        .map(|&i| vertices[i as usize])
                        .collect::<Vec<_>>()
                }
                None => positions.collect(),
            })
            .ok_or(error::Error::MissingPositions)?;

        trace!("Loading normals");
        let normals: Vec<[f32; 3]> = reader
            .read_normals()
            .map(|normals| match faces {
                Some(ref faces) => {
                    let normals = normals.collect::<Vec<_>>();
                    faces.iter().map(|&i| normals[i as usize]).collect()
                }
                None => normals.collect(),
            })
            .unwrap_or_else(|| {
                use amethyst_core::math::Point3;
                use std::iter::repeat;

                #[inline(always)]
                fn calc_normals(
                    a: [f32; 3],
                    b: [f32; 3],
                    c: [f32; 3],
                ) -> impl Iterator<Item = [f32; 3]> {
                    let a = Point3::from(a);
                    let ab = Point3::from(b) - a;
                    let ac = Point3::from(c) - a;
                    repeat(ab.cross(&ac).into()).take(3)
                }

                match faces {
                    Some(ref faces) => faces
                        .chunks(3)
                        .flat_map(|f| {
                            calc_normals(
                                positions[f[0] as usize],
                                positions[f[1] as usize],
                                positions[f[2] as usize],
                            )
                        })
                        .collect(),
                    None => positions
                        .chunks(3)
                        .flat_map(|p| calc_normals(p[0], p[1], p[2]))
                        .collect(),
                }
            });

        trace!("Loading texture coordinates");
        let uv: Vec<[f32; 2]> = reader
            .read_tex_coords(0)
            .map(|tex_coords| tex_coords.into_f32().collect::<Vec<[f32; 2]>>())
            .unwrap_or_else(|| {
                let u = options.generate_tex_coords.0;
                let v = if options.flip_v_coord {
                    1. - options.generate_tex_coords.1
                } else {
                    options.generate_tex_coords.1
                };
                vec![[u, v]; positions.len()]
            });

        let tex_coord: Vec<[f32; 2]> = match (&faces, options.flip_v_coord) {
            (Some(faces), true) => faces
                .iter()
                .map(|&i| [uv[i as usize][0], 1. - uv[i as usize][1]])
                .collect(),
            (Some(faces), false) => faces.iter().map(|&i| uv[i as usize]).collect(),
            (None, _) => uv,
        };

        trace!("Loading tangents");
        let tangents: Vec<[f32; 3]> = reader
            .read_tangents()
            .map(|tangents| match &faces {
                Some(faces) => {
                    let tangents = tangents.collect::<Vec<_>>();
                    faces
                        .iter()
                        .map(|&i| {
                            [
                                tangents[i as usize][0],
                                tangents[i as usize][1],
                                tangents[i as usize][2],
                            ]
                        })
                        .collect()
                }
                None => tangents.map(|t| [t[0], t[1], t[2]]).collect(),
            })
            .unwrap_or_else(|| {
                let f = faces
                    .as_ref()
                    .map(|f| f.clone())
                    .unwrap_or_else(|| (0..positions.len() as u32).collect::<Vec<_>>());
                let vertices_per_face = || 3;
                let face_count = || f.len() / 3;
                let p = |face, vert| &positions[f[face * 3 + vert] as usize];
                let n = |face, vert| &normals[f[face * 3 + vert] as usize];
                let tx = |face, vert| &tex_coord[f[face * 3 + vert] as usize];
                let mut tangents: Vec<(usize, [f32; 4])> = Vec::with_capacity(f.len());
                {
                    let mut set_tangent = |face, vert, tangent| {
                        let index = face * 3 + vert;
                        if let Err(pos) = tangents.binary_search_by(|probe| probe.0.cmp(&index)) {
                            tangents.insert(pos, (index, tangent));
                        }
                    };
                    mikktspace::generate_tangents(
                        &vertices_per_face,
                        &face_count,
                        &p,
                        &n,
                        &tx,
                        &mut set_tangent,
                    );
                }

                tangents.iter().map(|(_, t)| [t[0], t[1], t[2]]).collect()
            });

        trace!("Loading bounding box");
        let bounds = primitive.bounding_box();
        let bounds = bounds.min..bounds.max;

        // TODO: engine doesn't support vertex colors
        // trace!("Loading colors");
        // let colors = reader
        //     .read_colors(0)
        //     .map(|colors| colors.into_rgba_f32())
        //     .map(|colors| match &faces {
        //         Some(faces) => {
        //             let colors = colors.collect::<Vec<_>>();
        //             faces.iter().map(|i| colors[*i]).collect()
        //         }
        //         None => colors.collect(),
        //     });

        trace!("Loading joint ids");
        let joint_ids: Option<Vec<_>> =
            reader
                .read_joints(0)
                .map(|joints| joints.into_u16())
                .map(|joints| match faces {
                    Some(ref faces) => {
                        let joints = joints.collect::<Vec<_>>();
                        faces.iter().map(|&i| joints[i as usize]).collect()
                    }
                    None => joints.collect(),
                });
        trace!("Joint ids: {:?}", joint_ids);

        trace!("Loading joint weights");
        let joint_weights: Option<Vec<_>> = reader
            .read_weights(0)
            .map(|weights| weights.into_f32())
            .map(|weights| match faces {
                Some(ref faces) => {
                    let weights = weights.collect::<Vec<_>>();
                    faces.iter().map(|&i| weights[i as usize]).collect()
                }
                None => weights.collect(),
            });
        trace!("Joint weights: {:?}", joint_weights);

        let material = primitive.material().index();

        let mesh_builder =
            if let (Some(joint_ids), Some(joint_weights)) = (joint_ids, joint_weights) {
                let vertices: Vec<_> = positions
                    .into_iter()
                    .zip(normals.into_iter())
                    .zip(tangents.into_iter())
                    .zip(tex_coord.into_iter())
                    .zip(joint_ids.into_iter())
                    .zip(joint_weights.into_iter())
                    .map(|(((((pos, norm), tang), tex), joint_ids), joint_weights)| {
                        PosNormTangTexJoint {
                            position: pos.into(),
                            normal: norm.into(),
                            tangent: tang.into(),
                            tex_coord: tex.into(),
                            joint_ids: joint_ids.into(),
                            joint_weights: joint_weights.into(),
                        }
                    })
                    .collect();
                MeshBuilder::new().with_vertices(vertices)
            } else {
                let vertices: Vec<_> = positions
                    .into_iter()
                    .zip(normals.into_iter())
                    .zip(tangents.into_iter())
                    .zip(tex_coord.into_iter())
                    .map(|(((pos, norm), tang), tex)| PosNormTangTex {
                        position: pos.into(),
                        normal: norm.into(),
                        tangent: tang.into(),
                        tex_coord: tex.into(),
                    })
                    .collect();
                MeshBuilder::new().with_vertices(vertices)
            };

        primitives.push((mesh_builder, material, bounds));
    }
    trace!("Loaded mesh");
    Ok(primitives)
}
