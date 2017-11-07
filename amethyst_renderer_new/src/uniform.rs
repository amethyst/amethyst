

use core::Transform;
use gfx_hal::memory::Pod;

use cam::Camera;

pub trait UniformFormat: Sized {
    type DataType: Pod + From<Self>;
}

impl<T> UniformFormat for T
where
    T: Pod,
{
    type DataType = Self;
}

/*impl UniformFormat for Transform {
    type DataType = [[f32; 4]; 4];
}*/

impl UniformFormat for Camera {
    type DataType = [[f32; 4]; 4];
}
