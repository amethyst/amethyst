use amethyst_core::nalgebra::Matrix4;

#[derive(Debug, Clone)]
pub enum XRRenderInfo {
    Window,
    XR(XRTargetInfo),
}

#[derive(Debug, Clone)]
pub struct XRTargetInfo {
    pub(crate) render_target: usize,
    pub(crate) view_offset: Matrix4<f32>,
    pub(crate) projection: Matrix4<f32>,
}

impl XRRenderInfo {
    pub(crate) fn camera_reference(&self) -> Option<XRCameraReference> {
        match self {
            XRRenderInfo::Window => None,
            XRRenderInfo::XR(info) => Some(XRCameraReference {
                view_offset: &info.view_offset,
                projection: &info.projection,
            })
        }
    }
}

pub struct XRCameraReference<'a> {
    pub(crate) view_offset: &'a Matrix4<f32>,
    pub(crate) projection: &'a Matrix4<f32>,
}
