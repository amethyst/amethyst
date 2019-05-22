macro_rules! set_layout {
    ($factory:expr, $([$times:expr] $ty:ident $flags:ident),*) => {
        $factory.create_descriptor_set_layout(
            crate::util::set_layout_bindings(
                std::iter::empty()
                    $(.chain(std::iter::once((
                        $times as u32,
                        rendy::hal::pso::DescriptorType::$ty,
                        rendy::hal::pso::ShaderStageFlags::$flags
                    ))))*
            )
        )?.into()
    }
}
