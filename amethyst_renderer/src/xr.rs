use amethyst_core::cgmath::Matrix4;

pub struct XRRenderInfo {
    pub(crate) rendering_to_window: bool,
    pub(crate) target_id: usize,
    pub(crate) view_offset: Matrix4<f32>,
    pub(crate) projection: Matrix4<f32>,
}

impl XRRenderInfo {
    pub(crate) fn camera_reference(&self) -> Option<XRCameraReference> {
        if self.rendering_to_window {
            None
        } else {
            Some(XRCameraReference {
                view_offset: &self.view_offset,
                projection: &self.projection,
            })
        }
    }
}

pub struct XRCameraReference<'a> {
    pub(crate) view_offset: &'a Matrix4<f32>,
    pub(crate) projection: &'a Matrix4<f32>,
}
