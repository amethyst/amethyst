
use gfx_hal::command::BufferCopy;
use gfx_hal::RawCommandBuffer;


pub struct Staging<B: Backend> {
    src: Eh<Buffer<B>>,
    dst: Ec<Buffer<B>>,
}

impl<B> Staging<B: Backend> {
    fn new(src: Eh<Buffer<B>>, dst: Eh<Buffer<B>>, ec: &EpochCounter) {
        Staging {
            src,
            dst: Eh::borrow_for(dst, ec.now() + 2),
        }
    }

    fn commit(self, cbuf: &mut B::CommandBuffer, ec: &EpochCounter) {
        let ref dst = *st.get(ec).unwrap();
        assert_eq!(dst.size, src.size);
        cbuf.copy_buffer(&*src, dst, &[BufferCopy { src: 0, dst: 0, size: dst.size }]);
    }
}