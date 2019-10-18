pub use purple as arena;

pub struct Allocators {
    pub global_frame_arena: purple::Arena,
}
impl Allocators {
    pub(crate) fn frame(&mut self) {
        unsafe {
            self.global_frame_arena.reset();
        }
    }
}
