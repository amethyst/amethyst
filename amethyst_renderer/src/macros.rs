macro_rules! build_mesh_with_some {
    ($builder:expr, $factory:expr, $h:expr $(,$t:expr)*) => {
        match $h {
            Some(vertices) => build_mesh_with_some!($builder.with_buffer(vertices),
                                                    $factory $(,$t)*),
            None => build_mesh_with_some!($builder, $factory $(,$t)*),
        }
    };

    ($builder:expr, $factory:expr ) => {
        $factory.create_mesh($builder)
    };
}
