pub struct ScreenDimensions {
    pub w: u32,
    pub h: u32,
    pub aspect_ratio: f32,
}

impl ScreenDimensions {
    pub fn new(w: u32, h: u32) -> ScreenDimensions {
        ScreenDimensions {
            w: w,
            h: h,
            aspect_ratio: w as f32 / h as f32,
        }
    }
}
