use amethyst_core::{
    shrev::ReaderId,
    specs::prelude::{
        BitSet, Component, ComponentEvent, FlaggedStorage, Join, ReadExpect, Resources, System,
        WriteStorage,
    },
};
use amethyst_renderer::ScreenDimensions;

use super::*;

/// Whenever the window is resized the function in this component will be called on this
/// entity's UiTransform, along with the new width and height of the window.
///
/// The function in this component is also guaranteed to be called at least once by the
/// `ResizeSystem` when either the component is attached, or the function is changed.
pub struct UiResize {
    /// The core function of this component
    pub function: Box<dyn FnMut(&mut UiTransform, (f32, f32)) + Send + Sync>,
}

impl UiResize {
    /// Creates a new component with the given function.
    pub fn new<F>(function: F) -> Self
    where
        F: FnMut(&mut UiTransform, (f32, f32)) + Send + Sync + 'static,
    {
        UiResize {
            function: Box::new(function),
        }
    }
}

impl Component for UiResize {
    type Storage = FlaggedStorage<Self>;
}

/// This system rearranges UI elements whenever the screen is resized using their `UiResize`
/// component.
#[derive(Default)]
pub struct ResizeSystem {
    screen_size: (f32, f32),
    resize_events_id: Option<ReaderId<ComponentEvent>>,
    local_modified: BitSet,
}

impl ResizeSystem {
    /// Creates a new ResizeSystem that listens with the given reader Id.
    pub fn new() -> ResizeSystem {
        ResizeSystem::default()
    }
}

impl<'a> System<'a> for ResizeSystem {
    type SystemData = (
        WriteStorage<'a, UiTransform>,
        WriteStorage<'a, UiResize>,
        ReadExpect<'a, ScreenDimensions>,
    );

    fn run(&mut self, (mut transform, mut resize, dimensions): Self::SystemData) {
        self.local_modified.clear();

        let self_local_modified = &mut self.local_modified;

        let self_resize_events_id = self
            .resize_events_id
            .as_mut()
            .expect("`ResizeSystem::setup` was not called before `ResizeSystem::run`");
        resize
            .channel()
            .read(self_resize_events_id)
            .for_each(|event| match event {
                ComponentEvent::Inserted(id) | ComponentEvent::Modified(id) => {
                    self_local_modified.add(*id);
                }
                ComponentEvent::Removed(_id) => {}
            });

        let screen_size = (dimensions.width() as f32, dimensions.height() as f32);
        if self.screen_size != screen_size {
            self.screen_size = screen_size;
            for (transform, resize) in (&mut transform, &mut resize).join() {
                (resize.function)(transform, screen_size);
            }
        } else {
            // Immutable borrow
            let self_local_modified = &*self_local_modified;
            for (transform, resize, _) in (&mut transform, &mut resize, self_local_modified).join()
            {
                (resize.function)(transform, screen_size);
            }
        }

        // We need to treat any changes done inside the system as non-modifications, so we read out
        // any events that were generated during the system run
        resize
            .channel()
            .read(self_resize_events_id)
            .for_each(|event| match event {
                ComponentEvent::Inserted(id) | ComponentEvent::Modified(id) => {
                    self_local_modified.add(*id);
                }
                ComponentEvent::Removed(_id) => {}
            });
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::specs::prelude::SystemData;
        Self::SystemData::setup(res);
        self.screen_size = (0.0, 0.0);
        let mut resize = WriteStorage::<UiResize>::fetch(res);
        self.resize_events_id = Some(resize.register_reader());
    }
}
