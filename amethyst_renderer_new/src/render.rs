use gfx_hal::Gpu;

use back::*;


struct Renderer {
    device: Device,
    pipeline: Pipeline,
    memory_types: Vec<MemoryType>,
    memory_heaps: Vec<u64>,
}