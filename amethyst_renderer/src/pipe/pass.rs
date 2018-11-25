//! Types for constructing render passes.

use amethyst_core::specs::prelude::SystemData;

use crate::{
    error::Result,
    pipe::{Effect, NewEffect, Target},
    types::{Encoder, Factory},
};

/// Used to fetch data from the game world for rendering in the pass.
pub trait PassData<'a> {
    /// The data itself.
    type Data: SystemData<'a> + Send;
}

/// Structures implementing this provide a renderer pass.
pub trait Pass: for<'a> PassData<'a> {
    /// The pass is given an opportunity to compile shaders and store them in an `Effect`
    /// which is then passed to the pass in `apply`.
    fn compile(&mut self, effect: NewEffect<'_>) -> Result<Effect>;

    /// Called whenever the renderer is ready to apply the pass.  Feed commands into the
    /// encoder here.
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        factory: Factory,
        data: <Self as PassData<'b>>::Data,
    );
}

/// A compiled pass.  These are created and managed by the `Renderer`.  This should not be
/// used directly outside of the renderer.
#[derive(Clone, Debug)]
pub struct CompiledPass<P> {
    effect: Effect,
    inner: P,
}

impl<P> CompiledPass<P>
where
    P: Pass,
{
    pub(super) fn compile(
        mut pass: P,
        fac: &mut Factory,
        out: &Target,
        multisampling: u16,
    ) -> Result<Self> {
        let effect = pass.compile(NewEffect::new(fac, out, multisampling))?;
        Ok(CompiledPass {
            effect,
            inner: pass,
        })
    }
}

impl<P> CompiledPass<P> {
    /// Applies the inner pass.
    pub fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        factory: Factory,
        data: <P as PassData<'b>>::Data,
    ) where
        P: Pass,
    {
        self.inner.apply(encoder, &mut self.effect, factory, data)
    }

    /// Distributes new target data to the pass.
    pub fn new_target(&mut self, target: &Target) {
        // Distribute new targets that don't blend.
        self.effect.data.out_colors.clear();
        self.effect
            .data
            .out_colors
            .extend(target.color_bufs().iter().map(|cb| &cb.as_output).cloned());

        // Distribute new blend targets
        self.effect.data.out_blends.clear();
        self.effect
            .data
            .out_blends
            .extend(target.color_bufs().iter().map(|cb| &cb.as_output).cloned());

        // Distribute new depth buffer
        self.effect.data.out_depth = target.depth_buf().map(|db| (db.as_output.clone(), (0, 0)));
    }
}
