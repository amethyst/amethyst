//! ECS rendering bundle

use std::marker::PhantomData;

use amethyst_core::{
    ecs::*,
};
use amethyst_error::Error;
use derive_new::new;
use crate::{WidgetId, UiTransformSystem};

/// UI bundle
///
/// Will register all necessary components and systems needed for UI, along with any resources.
/// The generic type T represent the T generic parameter of the InputHandler<T>.
///
/// Will fail with error 'No resource with the given id' if either the InputBundle or TransformBundle are not added.
#[derive(new, Debug)]
pub struct UiBundle</*C = NoCustomUi, */W = u32, G = ()> {
    #[new(default)]
    _marker: PhantomData<(/*C,*/ W, G)>,
}

impl</*C,*/ W, G> SystemBundle for UiBundle</*C,*/ W, G>
where
    //C: ToNativeWidget,
    W: WidgetId,
    G: Send + Sync + PartialEq + 'static,
{
    fn load(&mut self, world: &mut World, resources: &mut Resources, builder: &mut DispatcherBuilder) -> Result<(), Error> {
        /*
                builder.add_system(UiTransformSystem::new().build());





                builder.add_system(
                    UiLoaderSystemDesc::<<C as ToNativeWidget>::PrefabData, W>::default().build(world),
                );


                builder.add_system(
                    UiTransformSystemDesc::default().build(world),
                );
                builder.add_system(
                    UiMouseSystem::<T>::new(),
                );
                builder.add_system(
                    Processor::<FontAsset>::new(),
                );
                builder.add_system(
                    CacheSelectionOrderSystem::<G>::new(),
                );
                builder.add_system(
                    SelectionMouseSystemDesc::<G, T>::default().build(world),
                );
                builder.add_system(
                    SelectionKeyboardSystemDesc::<G>::default().build(world),
                );
                builder.add_system(
                    TextEditingMouseSystemDesc::default().build(world),
                );
                builder.add_system(
                    TextEditingInputSystemDesc::default().build(world),
                );
                builder.add_system(
                    ResizeSystemDesc::default().build(world),
                );
                builder.add_system(
                    UiButtonSystemDesc::default().build(world),
                );
                builder.add_system(
                    DragWidgetSystemDesc::<T>::default().build(world),
                );

                builder.add_system(
                    UiButtonActionRetriggerSystemDesc::default().build(world),
                );
                builder.add_system(
                    UiSoundSystemDesc::default().build(world),
                );
                builder.add_system(
                    UiSoundRetriggerSystemDesc::default().build(world),
                );

                // Required for text editing. You want the cursor image to blink.
                builder.add_system(BlinkSystem);
        */
        Ok(())
    }

    fn unload(&mut self, _world: &mut World, _resources: &mut Resources) -> Result<(), Error> {
        unimplemented!()
    }
}
