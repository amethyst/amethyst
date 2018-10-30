use std::{iter::Iterator, mem, time::Duration};

use rodio::{Sample, Source};

// Wraps a source and calls the given closure when the source ends.
pub struct EndSignalSource<I: Source, F: FnOnce()>
where
    <I as Iterator>::Item: Sample,
{
    input: I,
    f: Option<F>,
}

impl<I: Source, F: FnOnce()> EndSignalSource<I, F>
where
    <I as Iterator>::Item: Sample,
{
    pub fn new(input: I, f: F) -> EndSignalSource<I, F> {
        EndSignalSource { input, f: Some(f) }
    }
}

impl<I: Source, F: FnOnce()> Iterator for EndSignalSource<I, F>
where
    <I as Iterator>::Item: Sample,
{
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.input.next();
        if next.is_none() {
            let f = mem::replace(&mut self.f, None);
            if let Some(f) = f {
                f()
            }
        }
        next
    }
}

impl<I: Source, F: FnOnce()> Source for EndSignalSource<I, F>
where
    <I as Iterator>::Item: Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.input.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }
}
