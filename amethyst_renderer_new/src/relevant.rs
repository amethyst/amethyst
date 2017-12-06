/// Values of this type can't be automatically dropped.
/// If struct or enum has field with type `Relevant`,
/// it can't be automatically dropped either. And so considered relevant too.
/// User has to deconstruct such values and call `Relevant::dispose`.
/// If relevant filed is private it means that user has to move value into some public method.
/// For example `memory::Block` should be returned to the `MemoryAllocator` it came from.
///
/// User of the engine won't usually deal with real relevant types.
/// More often user will face wrappers that has backdoor - some technique
/// to dispose internal relevant fields with runtime cost.
/// In debug mode such wrappers can put warnings in log.
/// So that user will know they should be disposed manually.
///
/// # Panics
///
/// Panics when dropped.
///
#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Relevant;

impl Relevant {
    /// Dispose this value.
    pub fn dispose(self) {
        ::std::mem::forget(self)
    }
}

impl Drop for Relevant {
    fn drop(&mut self) {
        println!("This type can't be dropped!")
    }
}
