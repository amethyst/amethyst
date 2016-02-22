use frame::Frame;

pub struct Frontend;

impl Frontend {
    pub fn new() -> Frontend {
        Frontend
    }

    pub fn draw(&mut self, _frame: &Frame) {
        unimplemented!();
    }
}
