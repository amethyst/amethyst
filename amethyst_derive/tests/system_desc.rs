use std::marker::PhantomData;

use amethyst_core::{
    ecs::{System, SystemData, World, WorldExt},
    shrev::{EventChannel, ReaderId},
    SystemDesc,
};
use amethyst_error::Error;

use amethyst_derive::SystemDesc;

#[test]
fn simple_derive() -> Result<(), Error> {
    #[derive(Debug, SystemDesc)]
    struct SystemUnit;

    impl<'s> System<'s> for SystemUnit {
        type SystemData = ();
        fn run(&mut self, _: Self::SystemData) {}
    }

    let mut world = World::new();

    <SystemUnit as SystemDesc<'_, '_, _>>::build(SystemUnit, &mut world);

    Ok(())
}

#[test]
fn rename_struct() -> Result<(), Error> {
    #[derive(Debug, SystemDesc)]
    #[system_desc(name(SystemUnitDesc))]
    struct SystemUnit;

    impl<'s> System<'s> for SystemUnit {
        type SystemData = ();
        fn run(&mut self, _: Self::SystemData) {}
    }

    let mut world = World::new();

    SystemUnitDesc.build(&mut world);

    Ok(())
}

#[test]
fn struct_tuple_with_phantom() -> Result<(), Error> {
    // Expects `System` to `impl Default`
    #[derive(Debug, SystemDesc)]
    #[system_desc(name(SystemTuplePhantomDesc))]
    struct SystemTuplePhantom<T>(PhantomData<T>);
    impl<T> Default for SystemTuplePhantom<T> {
        fn default() -> Self {
            SystemTuplePhantom(PhantomData)
        }
    }

    impl<'s, T> System<'s> for SystemTuplePhantom<T> {
        type SystemData = ();
        fn run(&mut self, _: Self::SystemData) {}
    }

    let mut world = World::new();

    SystemTuplePhantomDesc::<()>::default().build(&mut world);

    Ok(())
}

#[test]
fn struct_tuple_with_skip() -> Result<(), Error> {
    // Expects `System` to `impl Default`
    #[derive(Debug, Default, SystemDesc)]
    #[system_desc(name(SystemTupleSkipDesc))]
    struct SystemTupleSkip(#[system_desc(skip)] u32);

    impl<'s> System<'s> for SystemTupleSkip {
        type SystemData = ();
        fn run(&mut self, _: Self::SystemData) {}
    }

    let mut world = World::new();

    SystemTupleSkipDesc::default().build(&mut world);

    Ok(())
}

#[test]
fn struct_tuple_with_passthrough() -> Result<(), Error> {
    #[derive(Debug, Default, SystemDesc)]
    #[system_desc(name(SystemTuplePassthroughDesc))]
    struct SystemTuplePassthrough(u32);

    impl<'s> System<'s> for SystemTuplePassthrough {
        type SystemData = ();
        fn run(&mut self, _: Self::SystemData) {}
    }

    let mut world = World::new();

    SystemTuplePassthroughDesc::new(123).build(&mut world);

    Ok(())
}

#[test]
fn struct_tuple_with_event_channel_reader() -> Result<(), Error> {
    // Expects `System` to have a `new` constructor.
    #[derive(Debug, SystemDesc)]
    #[system_desc(name(SystemTupleEventChannelDesc))]
    struct SystemTupleEventChannel(#[system_desc(event_channel_reader)] ReaderId<u32>);
    impl SystemTupleEventChannel {
        fn new(reader_id: ReaderId<u32>) -> Self {
            Self(reader_id)
        }
    }

    impl<'s> System<'s> for SystemTupleEventChannel {
        type SystemData = ();
        fn run(&mut self, _: Self::SystemData) {}
    }

    let mut world = World::new();
    world.insert(EventChannel::<u32>::new());

    SystemTupleEventChannelDesc::default().build(&mut world);

    Ok(())
}

#[test]
fn struct_tuple_complex() -> Result<(), Error> {
    // Expects `System` to have a `new` constructor.
    pub trait Magic {}
    impl Magic for i8 {}
    impl Magic for i16 {}

    #[derive(Debug, SystemDesc)]
    #[system_desc(name(SystemTupleComplexDesc))]
    struct SystemTupleComplex<'x, T1, T2>(
        u8,
        PhantomData<&'x T1>,
        #[system_desc(event_channel_reader)] ReaderId<u32>,
        #[system_desc(skip)] i32,
        PhantomData<T2>,
        u32,
    )
    where
        T1: Magic,
        T2: Magic;

    impl<'x, T1, T2> SystemTupleComplex<'x, T1, T2>
    where
        T1: Magic,
        T2: Magic,
    {
        fn new(a: u8, reader_id: ReaderId<u32>, d: u32) -> Self {
            Self(a, PhantomData, reader_id, 999, PhantomData, d)
        }
    }

    impl<'s, 'x, T1, T2> System<'s> for SystemTupleComplex<'x, T1, T2>
    where
        T1: Magic,
        T2: Magic,
    {
        type SystemData = ();
        fn run(&mut self, _: Self::SystemData) {}
    }

    let mut world = World::new();
    world.insert(EventChannel::<u32>::new());

    SystemTupleComplexDesc::<'_, i8, i16>::new(1u8, 4u32).build(&mut world);

    Ok(())
}

#[test]
fn struct_named_with_phantom() -> Result<(), Error> {
    // Expects `System` to `impl Default`
    #[derive(Debug, SystemDesc)]
    #[system_desc(name(SystemNamedPhantomDesc))]
    struct SystemNamedPhantom<T> {
        marker: PhantomData<T>,
    }
    impl<T> Default for SystemNamedPhantom<T> {
        fn default() -> Self {
            SystemNamedPhantom {
                marker: PhantomData,
            }
        }
    }

    impl<'s, T> System<'s> for SystemNamedPhantom<T> {
        type SystemData = ();
        fn run(&mut self, _: Self::SystemData) {}
    }

    let mut world = World::new();

    SystemNamedPhantomDesc::<()>::default().build(&mut world);

    Ok(())
}

#[test]
fn struct_named_with_skip() -> Result<(), Error> {
    // Expects `System` to `impl Default`
    #[derive(Debug, Default, SystemDesc)]
    #[system_desc(name(SystemNamedSkipDesc))]
    struct SystemNamedSkip {
        #[system_desc(skip)]
        a: u32,
    }

    impl<'s> System<'s> for SystemNamedSkip {
        type SystemData = ();
        fn run(&mut self, _: Self::SystemData) {}
    }

    let mut world = World::new();

    SystemNamedSkipDesc::default().build(&mut world);

    Ok(())
}

#[test]
fn struct_named_with_passthrough() -> Result<(), Error> {
    #[derive(Debug, Default, SystemDesc)]
    #[system_desc(name(SystemNamedPassthroughDesc))]
    struct SystemNamedPassthrough {
        a: u32,
    }

    impl<'s> System<'s> for SystemNamedPassthrough {
        type SystemData = ();
        fn run(&mut self, _: Self::SystemData) {}
    }

    let mut world = World::new();

    SystemNamedPassthroughDesc::new(123).build(&mut world);

    Ok(())
}

#[test]
fn struct_named_with_event_channel_reader() -> Result<(), Error> {
    // Expects `System` to have a `new` constructor.
    #[derive(Debug, SystemDesc)]
    #[system_desc(name(SystemNamedEventChannelDesc))]
    struct SystemNamedEventChannel {
        #[system_desc(event_channel_reader)]
        u32_reader: ReaderId<u32>,
    }
    impl SystemNamedEventChannel {
        fn new(u32_reader: ReaderId<u32>) -> Self {
            Self { u32_reader }
        }
    }

    impl<'s> System<'s> for SystemNamedEventChannel {
        type SystemData = ();
        fn run(&mut self, _: Self::SystemData) {}
    }

    let mut world = World::new();
    world.insert(EventChannel::<u32>::new());

    SystemNamedEventChannelDesc::default().build(&mut world);

    Ok(())
}

#[test]
fn struct_named_complex() -> Result<(), Error> {
    // Expects `System` to have a `new` constructor.
    pub trait Magic {}
    impl Magic for i8 {}
    impl Magic for i16 {}

    #[derive(Debug, SystemDesc)]
    #[system_desc(name(SystemNamedComplexDesc))]
    struct SystemNamedComplex<'x, T1, T2>
    where
        T1: Magic,
        T2: Magic,
    {
        a: u8,
        marker1: PhantomData<&'x T1>,
        #[system_desc(event_channel_reader)]
        u32_reader: ReaderId<u32>,
        #[system_desc(skip)]
        c: i32,
        marker2: PhantomData<T2>,
        d: u32,
    }

    impl<'x, T1, T2> SystemNamedComplex<'x, T1, T2>
    where
        T1: Magic,
        T2: Magic,
    {
        fn new(a: u8, u32_reader: ReaderId<u32>, d: u32) -> Self {
            Self {
                a,
                marker1: PhantomData,
                u32_reader,
                c: 999,
                marker2: PhantomData,
                d,
            }
        }
    }

    impl<'s, 'x, T1, T2> System<'s> for SystemNamedComplex<'x, T1, T2>
    where
        T1: Magic,
        T2: Magic,
    {
        type SystemData = ();
        fn run(&mut self, _: Self::SystemData) {}
    }

    let mut world = World::new();
    world.insert(EventChannel::<u32>::new());

    SystemNamedComplexDesc::<'_, i8, i16>::new(1u8, 4u32).build(&mut world);

    Ok(())
}

#[test]
fn system_with_flagged_storage_reader() -> Result<(), Error> {
    use amethyst_core::{ecs::storage::ComponentEvent, transform::Transform};

    // Expects `System` to have a `new` constructor.
    #[derive(Debug, SystemDesc)]
    #[system_desc(name(SystemWithFlaggedStorageReaderDesc))]
    struct SystemWithFlaggedStorageReader {
        #[system_desc(flagged_storage_reader(Transform))]
        transform_events: ReaderId<ComponentEvent>,
    }
    impl SystemWithFlaggedStorageReader {
        fn new(transform_events: ReaderId<ComponentEvent>) -> Self {
            Self { transform_events }
        }
    }

    impl<'s> System<'s> for SystemWithFlaggedStorageReader {
        type SystemData = ();
        fn run(&mut self, _: Self::SystemData) {}
    }

    let mut world = World::new();
    world.register::<Transform>();

    SystemWithFlaggedStorageReaderDesc::default().build(&mut world);

    Ok(())
}
