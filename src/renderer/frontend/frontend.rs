use super::Renderable;
use renderer::backend::Backend;
use renderer::ir::CommandQueue;

/// A collection of renderable elements to be drawn by the Frontend.
pub struct Frame {
    elements: Vec<Box<Renderable>>,
}

impl Frame {
    pub fn new() -> Frame {
        Frame { elements: Vec::new() }
    }

    pub fn push<T: 'static>(&mut self, element: T)
        where T: Renderable
    {
        self.elements.push(Box::new(element));
    }

    pub fn peer_into(&self) -> &Vec<Box<Renderable>> {
        &self.elements
    }
}

/// Simple renderer frontend.
pub struct Frontend {
    backend: Box<Backend>,
    queue: CommandQueue,
}

impl Frontend {
    pub fn new<T: 'static>(backend: T) -> Frontend
        where T: Backend
    {
        Frontend {
            backend: Box::new(backend),
            queue: CommandQueue::new(),
        }
    }

    pub fn load_render_path(&mut self) {
        unimplemented!();
    }

    /// Draws a frame with the currently set render path. TODO: Build actual
    /// modular, parallelized Object translators.
    pub fn draw(&mut self, frame: Frame) {
        for element in frame.peer_into() {
            self.queue.submit(element.to_cmdbuf());
        }

        let commands = self.queue.sort_and_flush();
        self.backend.process(commands);
    }
}
