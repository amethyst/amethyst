/// Convenience type for storing and retrieving the result of an effect.
///
/// # Type Parameters
///
/// * `T`: Type of the object to store in the world.
#[derive(Debug)]
pub struct EffectReturn<T>(pub T);
