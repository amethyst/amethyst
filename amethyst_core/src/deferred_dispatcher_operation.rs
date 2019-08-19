//! Provides the ability to store `Systems`, `Bundles`, `Barriers`, in a normal vector for deferred dispatcher construction.

use std::marker::PhantomData;

use derivative::Derivative;

use amethyst_error::Error;

use crate::{
    ecs::prelude::{DispatcherBuilder, RunNow, System, World},
    RunNowDesc, SystemBundle, SystemDesc,
};

/// Trait to capture deferred dispatcher builder operations.
pub trait DispatcherOperation<'a, 'b> {
    /// Executes the dispatcher builder instruction.
    fn exec(
        self: Box<Self>,
        world: &mut World,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error>;
}

/// Deferred operation Add Barrier
#[derive(Debug)]
pub struct AddBarrier;

impl<'a, 'b> DispatcherOperation<'a, 'b> for AddBarrier {
    fn exec(
        self: Box<Self>,
        _world: &mut World,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        dispatcher_builder.add_barrier();
        Ok(())
    }
}

/// Deferred operation Add System
#[derive(Derivative)]
#[derivative(Debug)]
pub struct AddSystem<S> {
    /// System object
    #[derivative(Debug = "ignore")]
    pub system: S,
    /// System name
    pub name: &'static str,
    /// System dependencies list
    pub dependencies: &'static [&'static str],
}

impl<'a, 'b, S> DispatcherOperation<'a, 'b> for AddSystem<S>
where
    S: for<'s> System<'s> + Send + 'a,
{
    fn exec(
        self: Box<Self>,
        _world: &mut World,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        dispatcher_builder.add(self.system, self.name, self.dependencies);
        Ok(())
    }
}

/// Deferred operation Add System Desc
#[derive(Derivative)]
#[derivative(Debug)]
pub struct AddSystemDesc<SD, S> {
    /// System description
    #[derivative(Debug = "ignore")]
    pub system_desc: SD,
    /// System name
    pub name: &'static str,
    /// System dependencies
    pub dependencies: &'static [&'static str],
    /// Generic type holder
    pub marker: PhantomData<S>,
}

impl<'a, 'b, SD, S> DispatcherOperation<'a, 'b> for AddSystemDesc<SD, S>
where
    SD: SystemDesc<'a, 'b, S>,
    S: for<'s> System<'s> + Send + 'a,
{
    fn exec(
        self: Box<Self>,
        world: &mut World,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        let system = self.system_desc.build(world);
        dispatcher_builder.add(system, self.name, self.dependencies);
        Ok(())
    }
}

/// Deferred operation Add Thread Local
#[derive(Derivative)]
#[derivative(Debug)]
pub struct AddThreadLocal<S> {
    /// System object
    #[derivative(Debug = "ignore")]
    pub system: S,
}

impl<'a, 'b, S> DispatcherOperation<'a, 'b> for AddThreadLocal<S>
where
    S: for<'c> RunNow<'c> + 'b,
{
    fn exec(
        self: Box<Self>,
        _world: &mut World,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        dispatcher_builder.add_thread_local(self.system);
        Ok(())
    }
}

/// Deferred operation Add Thread Local Desc
#[derive(Derivative)]
#[derivative(Debug)]
pub struct AddThreadLocalDesc<SD, S> {
    /// System description
    #[derivative(Debug = "ignore")]
    pub system_desc: SD,
    /// Generic type holder
    pub marker: PhantomData<S>,
}

impl<'a, 'b, SD, S> DispatcherOperation<'a, 'b> for AddThreadLocalDesc<SD, S>
where
    SD: RunNowDesc<'a, 'b, S>,
    S: for<'c> RunNow<'c> + 'b,
{
    fn exec(
        self: Box<Self>,
        world: &mut World,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        let system = self.system_desc.build(world);
        dispatcher_builder.add_thread_local(system);
        Ok(())
    }
}

/// Deferred operation Add Bundle
#[derive(Derivative)]
#[derivative(Debug)]
pub struct AddBundle<B> {
    /// Bundle object
    #[derivative(Debug = "ignore")]
    pub bundle: B,
}

impl<'a, 'b, B> DispatcherOperation<'a, 'b> for AddBundle<B>
where
    B: SystemBundle<'a, 'b>,
{
    fn exec(
        self: Box<Self>,
        world: &mut World,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        self.bundle.build(world, dispatcher_builder)?;
        Ok(())
    }
}
