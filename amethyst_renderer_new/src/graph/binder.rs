
struct Binder<'a, B: Backend> {
    cbuf: &'a mut B::CommandBuffer,
    device: &'a mut Device,
    world: &World,
}

struct EntityBinder<'a, B: Backend> {
    entity: Entity,
    binder: Binder<'a, B>,
}

impl<'a, B> EntityBinder<'a, B>
where
    B: Backend,
{
    fn bind_uniform<F, T, U, C>(self, binding: usize, f: F) -> Self
    where
        F: Fn() -> T,
        T: IntoUniform<Uniform=U, Cache=C>,
        C: Component,
    {
        unimplemented!()
    }
}


#[cfg(test)]
fn test() {
    #[derive(Compoent)]
    #[component(NullStorage)]
    struct Pbm;

    type Components = (Mesh, Material, Transform, Camera, Light);

    let draw_pbm = PassBuider::new(Pbm)
        .vertex(include_bytes!(".vs"))
        .fragment(include_bytes!(".fs"))
        .with_color(Rgba8::SELF)
        .with_depth(DepthTest::On {
            fun: Comparison::GreaterEqual,
            write: true,
        })
        .binding::<Components, ActiveCamera>(|mut binder, entities, tag, components, active| {
            let (mesh, mtl, tr, cam, light) = components;

            let plights = (light, tr).join().filter_map(|(light, tr)| match *light {
                PointLight(ref light) => Some((ligth, tr)),
                _ => None,
            });
            let dlights = (light, tr).join().filter_map(|(light, tr)| match *light {
                DirectionalLight(ref light) => Some((ligth, tr)),
                _ => None,
            });

            binder
                .bind_uniform_array(0, || plights.map(|(l, t)| l))
                .bind_uniform_array(1, || plights.map(|(l, t)| t))
                .bind_uniform_array(2, || dlights.map(|(l, t)| l))
                .bind_uniform_array(3, || dlights.map(|(l, t)| t))
                .entity(active.entity, |binder| {
                    binder
                        .bind_uniform(4, || cam.get(active.entity).unwrap_or_default())
                        .bind_uniform(5, || tr.get(active.entity).unwrap_or_default().invert())
                });

            (entities, mesh, mtl, tr, tag).join().map(|(ent,
              mesh,
              mtl,
              tr,
              _)| {
                binder.next().entity(ent, |binder| {
                    binder
                        .bind_vertices(0..4, || {
                            mesh.bind(
                                &[
                                    Position::VERTEX_FORMAT,
                                    Normal::VERTEX_FORMAT,
                                    Tangent::VERTEX_FORMAT,
                                    TexCoord::VERTEX_FORMAT,
                                ],
                            )?
                        })
                        .bind_textures(0..7, || mtl)
                        .bind_uniform(6, || tr)
                })
            })
        })
        .build(device);
}
