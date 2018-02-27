#[derive(Debug)]
struct ReclamationNode<I> {
    items: Vec<I>,
}

#[derive(Debug)]
pub struct ReclamationQueue<I> {
    offset: u64,
    queue: Vec<ReclamationNode<I>>,
    cache: Vec<ReclamationNode<I>>,
}

impl<I> ReclamationQueue<I> {
    pub fn new() -> Self {
        ReclamationQueue {
            offset: 0,
            queue: Vec::new(),
            cache: Vec::new(),
        }
    }

    fn grow(&mut self, current: u64) {
        if self.queue.is_empty() {
            self.offset = current;
        }

        for _ in self.queue.len()..(current - self.offset + 1) as usize {
            self.queue.push(
                self.cache
                    .pop()
                    .unwrap_or_else(|| ReclamationNode { items: Vec::new() }),
            )
        }
    }

    pub fn push(&mut self, current: u64, item: I) {
        self.grow(current);
        self.queue[(current - self.offset) as usize]
            .items
            .push(item);
    }

    pub fn clear<F>(&mut self, ongoing: u64, mut f: F)
    where
        F: FnMut(I),
    {
        for mut node in self.queue.drain(..(ongoing - self.offset) as usize) {
            for item in node.items.drain(..) {
                f(item);
            }
            self.cache.push(node);
        }
        self.offset = ongoing;
    }
}
