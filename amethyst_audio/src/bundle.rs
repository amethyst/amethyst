//! ECS audio bundles

use amethyst_assets::Processor;
use amethyst_core::{
    alga::general::SubsetOf, bundle::SystemBundle, ecs::prelude::DispatcherBuilder, math::RealField,
};
use amethyst_error::Error;
use std::marker::PhantomData;

use crate::{output::Output, source::*, systems::AudioSystem};

/// Audio bundle
///
/// This will only add the audio system and the asset processor for `Source`.
///
/// `DjSystem` must be added separately if you want to use our background music system.
///
/// The generic N type should be the same as the one in `Transform<N>`.
#[derive(Default)]
pub struct AudioBundle<N>(Output, PhantomData<N>);

impl<'a, 'b, N: RealField> SystemBundle<'a, 'b> for AudioBundle<N>
where
    N: RealField + SubsetOf<f32>,
{
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(AudioSystem::<N>::new(self.0), "audio_system", &[]);
        builder.add(Processor::<Source>::new(), "source_processor", &[]);
        Ok(())
    }
}
