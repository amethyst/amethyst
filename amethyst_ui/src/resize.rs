use amethyst_core::ecs::*;
use amethyst_window::ScreenDimensions;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use super::*;

/// Whenever the window is resized the function in this component will be called on this
/// entity's UiTransform, along with the new width and height of the window.
///
/// The function in this component is also guaranteed to be called at least once by the
/// `ResizeSystem` when either the component is attached, or the function is changed.
#[allow(missing_debug_implementations)]
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

#[derive(Debug)]
pub struct ResizeSystemResource {
    screen_size: (f32, f32),
}

impl ResizeSystemResource {
    /// Creates a new ResizeSystem that listens with the given reader Id.
    pub fn new() -> ResizeSystemResource {
        let screen_size = (0.0, 0.0);

        ResizeSystemResource { screen_size }
    }
}

/// This system rearranges UI elements whenever the screen is resized using their `UiResize`
/// component.
pub fn build_resize_system(resources: &mut Resources) -> impl Runnable {
    resources.insert(ResizeSystemResource::new());
    SystemBuilder::new("ResizeSystem")
        .write_resource::<ResizeSystemResource>()
        .read_resource::<ScreenDimensions>()
        .with_query(
            <(&mut UiTransform, &mut UiResize)>::query().filter(maybe_changed::<UiResize>()),
        )
        .with_query(<(&mut UiTransform, &mut UiResize)>::query())
        .build(
            move |_commands, world, (resource, screen_dimensions), (resized, all_with_resize)| {
                #[cfg(feature = "profiler")]
                profile_scope!("resize_system");
                let screen_size = (
                    screen_dimensions.width() as f32,
                    screen_dimensions.height() as f32,
                );

                // If the screen_size changed, we resize everything with UiResize
                if resource.screen_size != screen_size {
                    resource.screen_size = screen_size;
                    all_with_resize.for_each_mut(world, |(transform, resize)| {
                        (resize.function)(transform, screen_size);
                    });
                }
                // Else, we only try to resize the modified ones
                else {
                    resized.for_each_mut(world, |(transform, resize)| {
                        (resize.function)(transform, screen_size);
                    });
                }
            },
        )
}
