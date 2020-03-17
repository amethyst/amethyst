macro_rules! set_layout {
    ($factory:expr, $(($times:expr, $descriptor_type:expr, $flags:expr)),*) => {
        $factory.create_descriptor_set_layout(
            crate::util::set_layout_bindings(
                std::iter::empty()
                    $(.chain(std::iter::once((
                        $times as u32,
                        $descriptor_type,
                        $flags
                    ))))*
            )
        )?.into()
    }
}
