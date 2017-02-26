use std::io::{Read, Write};

pub trait Import<T> {
    fn import<R: Read>(stream: R) -> Result<T, String>;
}

pub trait Export<T> {
    fn export<W: Write>(stream: W, data: T) -> Result<(), String>;
}
