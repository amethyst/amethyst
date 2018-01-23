use gfx_hal::Backend;
use hibitset::BitSetLike;
use specs::{Component, DenseVecStorage, Index, UnprotectedStorage};

use descriptors::{DescriptorPool, DescriptorSet};
use relevant::RelevantStorage;

impl<B, P> Component for DescriptorSet<B, P>
where
    B: Backend,
    P: 'static,
{
    type Storage = RelevantStorage<DenseVecStorage<DescriptorSet<B, P>>, DescriptorSet<B, P>>;
}
