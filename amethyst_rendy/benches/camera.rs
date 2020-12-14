use amethyst_core::{
    math::{convert, Matrix4, Translation3, UnitQuaternion},
    transform::Transform,
};
use criterion::{criterion_group, criterion_main, Criterion};

// Our world-space is +Y Up, +X Right and -Z Away
// Current render target is +Y Down, +X Right and +Z Away
fn setup() -> Transform {
    // Setup common inputs for most of the tests.
    //
    // Sets up a test camera is positioned at (0,0,3) in world space.
    // A camera without rotation is pointing in the (0,0,-1) direction.
    Transform::new(
        Translation3::new(0.0, 0.0, 3.0),
        // Apply _no_ rotation
        UnitQuaternion::identity(),
        [1.0, 1.0, 1.0].into(),
    )
}

pub fn transform_global_view_matrix_1000(b: &mut Criterion) {
    let transform = setup();

    b.bench_function("transform_global_view_matrix_1000", move |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let _: [[f32; 4]; 4] =
                    convert::<_, Matrix4<f32>>(transform.global_view_matrix()).into();
            }
        });
    });
}

pub fn manual_inverse_global_matrix_1000(b: &mut Criterion) {
    let transform = setup();

    b.bench_function("manual_inverse_global_matrix_1000", move |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let _: [[f32; 4]; 4] = convert::<_, Matrix4<f32>>(
                    transform
                        .global_matrix()
                        .try_inverse()
                        .expect("Unable to get inverse of camera transform"),
                )
                .into();
            }
        });
    });
}

criterion_group!(
    cameras,
    transform_global_view_matrix_1000,
    manual_inverse_global_matrix_1000,
);
criterion_main!(cameras);
