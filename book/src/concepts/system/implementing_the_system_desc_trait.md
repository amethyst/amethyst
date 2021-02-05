# Implementing the `SystemDesc` Trait

If the `SystemDesc` derive is unable to generate a `SystemDesc` trait
implementation for system initialization, the `SystemDesc` trait can be
implemented manually:

```rust
use amethyst::{
    audio::output::Output,
    ecs::{System, World},
};

# /// Syncs 3D transform data with the audio engine to provide 3D audio.
# #[derive(Debug, Default)]
# pub struct AudioSystem(Output);
# impl System for AudioSystem {
#   fn build(mut self) -> Box<dyn ParallelRunnable> {}
# }
# 
/// Builds an `AudioSystem`.
#[derive(Default, Debug)]
pub struct AudioSystemDesc {
    /// Audio `Output`.
    pub output: Output,
}

impl SystemDesc<'a, 'b, AudioSystem> for AudioSystemDesc {
    fn build(self, world: &mut World) -> AudioSystem {
        world.insert(self.output.clone());

        AudioSystem(self.output)
    }
}

// in `main.rs`:
// let game_data = DispatcherBuilder::default()
//     .with_system_desc(AudioSystemDesc::default(), "", &[]);
```

## Templates

```rust
/// Builds a `SystemName`.
#[derive(Default, Debug)]
pub struct SystemNameDesc;

impl SystemDesc<'a, 'b, SystemName> for SystemNameDesc {
    fn build(self, world: &mut World) -> SystemName {
        let arg = unimplemented!("Replace code here");

        SystemName::new(arg)
    }
}
```

With type parameters:

```rust
use std::marker::PhantomData;

use derivative::Derivative;

/// Builds a `SystemName`.
#[derive(Derivative, Debug)]
#[derivative(Default(bound = ""))]
pub struct SystemNameDesc<T> {
    marker: PhantomData<T>,
}

impl SystemDesc<'a, 'b, SystemName<T>>
    for SystemNameDesc<T>
where
    T: unimplemented!("Replace me."),
{
    fn build(self, world: &mut World) -> SystemName<T> {
        let arg = unimplemented!("Replace code here");

        SystemName::new(arg)
    }
}
```
