
use std::marker::PhantomData;
use std::iter::once;
use std::ops::{Deref, DerefMut};

use gfx_hal::{Backend, Device};
use gfx_hal::pso::{
    DescriptorWrite,
    DescriptorSetWrite,
    DescriptorSetLayoutBinding,
    DescriptorType,
    ShaderStageFlags,
};

use smallvec::SmallVec;
use specs::{EntitiesRes, Join, MaskedStorage, Storage};

use descriptors::{DescriptorSet, DescriptorPool};
use graph::PassTag;
use relevant::Relevant;
use uniform::UniformCache;

/// Single binding.
/// Type and count are constant for particular type
/// binding index and stage flags are specified during creation.
pub trait Binding: Copy {
    /// Type of the binding.
    const TY: DescriptorType;

    /// Count of binding.
    const COUNT: usize;

    /// Binding index.
    fn binding(self) -> usize;

    /// Stage flags for binding.
    fn stage(self) -> ShaderStageFlags;
}

/// Uniform non-array binding type.
#[derive(Derivative)]
#[derivative(Clone, Copy, Debug)]
pub struct Uniform<T> {
    binding: usize,
    stage: ShaderStageFlags,
    pd: PhantomData<fn() -> T>,
}

impl<T> Uniform<T> {
    fn bind<'a, 'b, B, C>(self, set: &'a B::DescriptorSet, cache: &'b C) -> DescriptorSetWrite<'a, 'b, B>
    where
        B: Backend,
        C: UniformCache<B, T>,
    {
        let (buf, range) = cache.get_cached();
        DescriptorSetWrite {
            set,
            binding: self.binding(),
            array_offset: 0,
            write: DescriptorWrite::UniformBuffer(vec![(buf.raw(), range)]),
        }
    }
}

impl<T> Binding for Uniform<T> {
    const TY: DescriptorType = DescriptorType::UniformBuffer;
    const COUNT: usize = 1;

    #[inline(always)]
    fn binding(self) -> usize {
        self.binding
    }

    #[inline(always)]
    fn stage(self) -> ShaderStageFlags {
        self.stage
    }
}

/// List of bindings.
/// `()` is empty list, `(H, T)` is `BindingsLists` when `H: Binding` and `T: BindingsList`.
pub trait BindingsList: Copy {

    /// Fill bindings structures.
    fn fill<E>(self, extend: &mut E) where E: Extend<DescriptorSetLayoutBinding>;
}

impl BindingsList for () {
    fn fill<E>(self, extend: &mut E) {}
}

impl<H, T> BindingsList for (H, T)
where
    H: Binding,
    T: BindingsList,
{
    fn fill<E>(self, extend: &mut E) where E: Extend<DescriptorSetLayoutBinding> {
        extend.extend(once(DescriptorSetLayoutBinding {
            ty: H::TY,
            count: H::COUNT,
            stage_flags: self.0.stage(),
            binding: self.0.binding(),
        }));
        self.1.fill(extend);
    }
}

/// Pipeline layout type-level representation.
#[derive(Copy, Clone)]
pub struct Layout<L> {
    bindings: L,
}


impl Layout<()> {
    /// Crate empty layout.
    pub(crate) fn new() -> Self {
        Layout {
            bindings: (),
        }
    }
}

impl<L> Layout<L>
where
    L: BindingsList,
{
    /// Add uniform binding to the layout.
    /// binding - index of the binding.
    /// stage - stage or stage flags.
    pub fn uniform<T, S: Into<ShaderStageFlags>>(self, binding: usize, stage: S) -> Layout<(Uniform<T>, L)> {
        self.with(Uniform{ binding, stage: stage.into(), pd: PhantomData})
    }

    /// Get array of bindings.
    pub(crate) fn bindings(self) -> SmallVec<[DescriptorSetLayoutBinding; 64]> {
        let mut bindings = SmallVec::<[_; 64]>::new();
        self.bindings.fill(&mut bindings);
        bindings
    }

    /// Add element to the layout.
    fn with<B>(self, binding: B) -> Layout<(B, L)> {
        Layout {
            bindings: (binding, self.bindings),
        }
    }
}


/// Binder can be used to bind bindings. =^___^=
pub struct Binder<L> {
    bindings: L,
}

impl<L> Binder<L>
where
    L: Clone,
{
    pub fn set<'a, 'b, B>(&self, set: &'a B::DescriptorSet) -> SetBinder<'a, 'b, B, L>
    where
        B: Backend,
    {
        SetBinder {
            relevant: Relevant,
            bindings: self.bindings.clone(),
            set,
            writes: SmallVec::new(),
        }
    }

    pub fn entities<'a, B, P, T, J, D, F>(&self, tag: &Storage<PassTag<P>, T>, join: J, entities: &EntitiesRes, descriptors: &mut Storage<DescriptorSet<B, P>, D>, pool: &mut DescriptorPool<B>, f: F)
    where
        B: Backend,
        P: Send + Sync + 'static,
        T: Deref<Target = MaskedStorage<PassTag<P>>>,
        J: Join,
        D: DerefMut<Target = MaskedStorage<DescriptorSet<B, P>>>,
        F: for<'b> Fn(SetBinder<'b, 'a, B, L>, J::Type),
    {
        for (_, j, e) in (tag, join, entities).join() {
            if descriptors.get(e).is_none() {
                let set = pool.get::<P>();
                f(self.set(set.raw()), j);
                descriptors.insert(e, set);
            }
        }
    }
}

impl<L> From<Layout<L>> for Binder<L> {
    fn from(layout: Layout<L>) -> Self {
        Binder {
            bindings: layout.bindings,
        }
    }
}

pub struct SetBinder<'a, 'b, B: Backend, L> {
    relevant: Relevant,
    bindings: L,
    set: &'a B::DescriptorSet,
    writes: SmallVec<[DescriptorSetWrite<'a, 'b, B>; 64]>,
}

impl<'a, 'b, B> SetBinder<'a, 'b, B, ()>
where
    B: Backend,
{
    pub fn bind(self, device: &B::Device) {
        device.update_descriptor_sets(&self.writes);
    }
}

impl<'a, 'b, B, H, T> SetBinder<'a, 'b, B, (Uniform<H>, T)>
where
    B: Backend,
{
    pub fn uniform<C>(self, cache: &'b C) -> SetBinder<'a, 'b, B, T>
    where
        C: UniformCache<B, H>,
    {
        let SetBinder {
            relevant,
            bindings: (head, tail),
            set,
            mut writes,
        } = self;

        writes.push(head.bind(set, cache));
        SetBinder {
            relevant,
            bindings: tail,
            set,
            writes,
        }
    }
}

