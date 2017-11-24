
/// Type that can't be silently dropped.
trait Relevant {}

impl<T> Drop for T
where
    T: Relevant
{
    fn drop(&mut self) {
        panic!("This type can't be dropped")
    }
}