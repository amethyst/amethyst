use crossbeam_channel::{unbounded, Receiver, Sender, TryIter};

#[derive(Debug, Clone)]
pub struct Escape<T> {
    sender: Sender<T>,
}

impl<T> Escape<T> {
    pub fn escape(&mut self, value: T) {
        self.sender
            .send(value)
            .expect("Terminal dropped before escapes");
    }
}

#[derive(Debug)]
pub struct Terminal<T> {
    receiver: Receiver<T>,
    sender: Sender<T>,
}

impl<T> Terminal<T> {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Terminal { sender, receiver }
    }

    pub fn escape(&self) -> Escape<T> {
        Escape {
            sender: self.sender.clone(),
        }
    }

    pub fn drain(&mut self) -> TryIter<T> {
        self.receiver.try_iter()
    }
}
