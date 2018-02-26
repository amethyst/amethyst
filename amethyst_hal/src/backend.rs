use hal::{Backend, Instance};
use winit::Window;

/// Extend backend trait with initialization method.
pub trait BackendEx: Backend {
    type Instance: Instance<Backend = Self> + Send + Sync;
    fn init() -> Self::Instance;
    fn create_surface(instance: &Self::Instance, window: &Window) -> Self::Surface;
}

#[cfg(feature = "gfx-vulkan")]
impl BackendEx for ::vulkan::Backend {
    type Instance = ::vulkan::Instance;
    fn init() -> Self::Instance {
        ::vulkan::Instance::create("amethyst", 1)
    }
    fn create_surface(instance: &Self::Instance, window: &Window) -> Self::Surface {
        instance.create_surface(window)
    }
}

#[cfg(feature = "gfx-metal")]
impl BackendEx for ::metal::Backend {
    type Instance = ::metal::Instance;
    fn init() -> Self::Instance {
        ::metal::Instance::create("amethyst", 1)
    }
    fn create_surface(instance: &Self::Instance, window: &Window) -> Self::Surface {
        instance.create_surface(window)
    }
}

#[cfg(feature = "gfx-dx12")]
impl BackendEx for ::dx12::Backend {
    type Instance = ::dx12::Instance;
    fn init() -> Self::Instance {
        ::dx12::Instance::create("amethyst", 1)
    }
    fn create_surface(instance: &Self::Instance, window: &Window) -> Self::Surface {
        instance.create_surface(window)
    }
}

#[cfg(not(any(feature = "gfx-vulkan", feature = "gfx-metal", feature = "gfx-dx12")))]
impl BackendEx for ::empty::Backend {
    type Instance = ::empty::Instance;
    fn init() -> Self::Instance {
        ::empty::Instance
    }
    fn create_surface(_: &Self::Instance, _: &Window) -> Self::Surface {
        ::empty::Surface
    }
}