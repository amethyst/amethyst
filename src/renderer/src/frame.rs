pub struct Frame;

impl Frame {
    pub fn new() -> Frame {
        Frame
    }

    pub fn build() -> FrameBuilder {
        FrameBuilder::new()
    }
}

pub struct FrameBuilder {
    frame: Frame,
}

impl FrameBuilder {
    pub fn new() -> FrameBuilder {
        FrameBuilder { frame: Frame::new() }
    }

    pub fn done(self) -> Frame {
        self.frame
    }
}
