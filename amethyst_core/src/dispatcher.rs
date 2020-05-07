use crate::{
    ecs::prelude::*,
    ArcThreadPool,
};
use std::collections::BTreeMap;

pub trait SystemBundle {
    fn build(
        self,
        world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder<'_>,
    ) -> Result<(), amethyst_error::Error>;
}

impl SystemBundle
    for Box<dyn FnMut(&mut World, &mut Resources, &mut DispatcherBuilder<'_>) -> Result<(), amethyst_error::Error>>
{
    fn build(
        mut self,
        world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder<'_>,
    ) -> Result<(), amethyst_error::Error> {
        (self)(world, resources, builder)
    }
}

pub struct DispatcherSystemBundle<B>(B);
impl<B: SystemBundle> ConsumeDesc for DispatcherSystemBundle<B> {
    fn consume(
        self: Box<Self>,
        world: &mut World,
        resources: &mut Resources,
        _: &mut DispatcherData,
        builder: &mut DispatcherBuilder<'_>,
    ) -> Result<(), amethyst_error::Error> {
        self.0.build(world, resources, builder)?;
        Ok(())
    }
}

pub struct DispatcherSystem<F>(RelativeStage, F);
impl<F> ConsumeDesc for DispatcherSystem<F>
where
    F: FnOnce(&mut World, &mut Resources) -> Box<dyn Schedulable>,
{
    fn consume(
        self: Box<Self>,
        world: &mut World,
        resources: &mut Resources,
        dispatcher: &mut DispatcherData,
        _: &mut DispatcherBuilder<'_>,
    ) -> Result<(), amethyst_error::Error> {
        let sys = (self.1)(world, resources);

        dispatcher
            .stages
            .entry(self.0)
            .or_insert_with(Vec::default)
            .push(sys);

        Ok(())
    }
}

pub struct DispatcherThreadLocalSystem<F>(F);
impl<F> ConsumeDesc for DispatcherThreadLocalSystem<F>
where
    F: FnOnce(&mut World, &mut Resources) -> Box<dyn Runnable>,
{
    fn consume(
        self: Box<Self>,
        world: &mut World,
        resources: &mut Resources,
        dispatcher: &mut DispatcherData,
        _: &mut DispatcherBuilder<'_>,
    ) -> Result<(), amethyst_error::Error> {
        let runnable = (self.0)(world, resources);

        // TODO: dispose?
        dispatcher.thread_locals.push(runnable.into());
        Ok(())
    }
}

pub struct DispatcherThreadLocal<F>(F);
impl<F> ConsumeDesc for DispatcherThreadLocal<F>
where
    F: FnOnce(&mut World, &mut Resources) -> Box<dyn ThreadLocal>,
{
    fn consume(
        self: Box<Self>,
        world: &mut World,
        resources: &mut Resources,
        dispatcher: &mut DispatcherData,
        _: &mut DispatcherBuilder<'_>,
    ) -> Result<(), amethyst_error::Error> {
        dispatcher.thread_locals.push((self.0)(world, resources));
        Ok(())
    }
}
pub trait ConsumeDesc {
    fn consume(
        self: Box<Self>,
        world: &mut World,
        resources: &mut Resources,
        stages: &mut DispatcherData,
        builder: &mut DispatcherBuilder<'_>,
    ) -> Result<(), amethyst_error::Error>;
}

pub trait ThreadLocal {
    fn run(&mut self, world: &mut World, resources: &mut Resources);
    fn dispose(self: Box<Self>, world: &mut World, resources: &mut Resources);
}

impl<F> ThreadLocal for F
where
    F: FnMut(&mut World, &mut Resources) + 'static,
{
    fn run(&mut self, world: &mut World, resources: &mut Resources) {
        (self)(world, resources)
    }
    fn dispose(self: Box<Self>, _world: &mut World, _resources: &mut Resources) {}
}

impl Into<Box<dyn ThreadLocal>> for Box<dyn Runnable> {
    fn into(mut self) -> Box<dyn ThreadLocal> {
        Box::new(move |world: &mut World, resources: &mut Resources| {
            self.run(world, resources);
        })
    }
}

pub struct ThreadLocalObject<S, F, D>(pub S, pub F, pub D);
impl<S, F, D> ThreadLocalObject<S, F, D>
where
    S: 'static,
    F: FnMut(&mut S, &mut World, &mut Resources) + 'static,
    D: FnOnce(S, &mut World, &mut Resources) + 'static,
{
    pub fn build(initial_state: S, run_fn: F, dispose_fn: D) -> Box<dyn ThreadLocal> {
        Box::new(Self(initial_state, run_fn, dispose_fn))
    }
}
impl<S, F, D> ThreadLocal for ThreadLocalObject<S, F, D>
where
    S: 'static,
    F: FnMut(&mut S, &mut World, &mut Resources) + 'static,
    D: FnOnce(S, &mut World, &mut Resources) + 'static,
{
    fn run(&mut self, world: &mut World, resources: &mut Resources) {
        (self.1)(&mut self.0, world, resources)
    }
    fn dispose(self: Box<Self>, world: &mut World, resources: &mut Resources) {
        (self.2)(self.0, world, resources)
    }
}

pub trait IntoRelativeStage: Copy {
    fn into_relative(self) -> RelativeStage;
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Stage {
    Begin,
    Logic,
    Render,
    ThreadLocal,
}
impl IntoRelativeStage for Stage {
    fn into_relative(self) -> RelativeStage {
        RelativeStage(self, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct RelativeStage(pub Stage, pub isize);
impl RelativeStage {
    pub fn stage(&self) -> Stage {
        self.0
    }

    pub fn offset(&self) -> isize {
        self.1
    }
}
impl IntoRelativeStage for RelativeStage {
    fn into_relative(self) -> RelativeStage {
        self
    }
}
impl From<Stage> for RelativeStage {
    fn from(other: Stage) -> Self {
        RelativeStage(other, 0)
    }
}
impl PartialOrd for RelativeStage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for RelativeStage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.stage()
            .cmp(&other.stage())
            .then(self.offset().cmp(&other.offset()))
    }
}

pub struct Dispatcher {
    executor: Executor,
    pub defrag_budget: Option<usize>,
    pub(crate) thread_locals: Vec<Box<dyn ThreadLocal>>,
}
impl Dispatcher {
    pub fn dispatch(&mut self, world: &mut World, resources: &mut Resources) {
        self.executor.execute(world, resources);

        self.thread_locals
            .iter_mut()
            .for_each(|local| local.run(world, resources));

        //world.defrag(self.defrag_budget);
    }
    pub fn dispose(mut self, world: &mut World, resources: &mut Resources) {
        self.thread_locals
            .drain(..)
            .for_each(|local| local.dispose(world, resources));

        // self.executor
        //     .into_vec()
        //     .into_iter()
        //     .for_each(|system| system.dispose(world, resources));
    }
}

#[derive(Default)]
pub struct DispatcherData {
    pub defrag_budget: Option<usize>,
    pub(crate) thread_locals: Vec<Box<dyn ThreadLocal>>,
    pub(crate) stages: BTreeMap<RelativeStage, Vec<Box<dyn Schedulable>>>,
}
impl DispatcherData {
    pub fn flatten(mut self) -> Dispatcher {
        let mut sorted_systems = Vec::with_capacity(128);
        self.stages
            .into_iter()
            .for_each(|(_, mut v)| v.drain(..).for_each(|sys| sorted_systems.push(sys)));

        log::trace!("Sorted {} systems", sorted_systems.len());
        if log::log_enabled!(log::Level::Trace) {
            sorted_systems.iter().for_each(|system| {
                log::trace!("System: {}", system.name());
            });
        }

        let executor = Executor::new(sorted_systems);

        Dispatcher {
            defrag_budget: self.defrag_budget,
            thread_locals: self.thread_locals,
            executor,
        }
    }

    pub fn merge(mut self, mut other: DispatcherData) -> Self {
        self.thread_locals.extend(other.thread_locals.drain(..));

        for (k, mut v) in other.stages.iter_mut() {
            self.stages
                .entry(*k)
                .or_insert_with(Vec::default)
                .extend(v.drain(..))
        }

        self
    }
}

pub struct DispatcherBuilder<'a> {
    pub(crate) defrag_budget: Option<usize>,
    pub(crate) systems: Vec<(RelativeStage, Box<dyn ConsumeDesc + 'a>)>,
    pub(crate) thread_locals: Vec<Box<dyn ConsumeDesc + 'a>>,
    pub(crate) bundles: Vec<Box<dyn ConsumeDesc + 'a>>,
    pub(crate) thread_pool: Option<ArcThreadPool>,
}
impl<'a> Default for DispatcherBuilder<'a> {
    // We preallocate 128 for these, as its just a random round number but they are just fat-pointers so whatever
    fn default() -> Self {
        Self {
            defrag_budget: None,
            systems: Vec::with_capacity(128),
            thread_locals: Vec::with_capacity(128),
            bundles: Vec::with_capacity(128),
            thread_pool: None,
        }
    }
}
impl<'a> DispatcherBuilder<'a> {
    pub fn add_thread_local<T: FnOnce(&mut World, &mut Resources) -> Box<dyn ThreadLocal> + 'a>(
        &mut self,
        desc: T,
    ) {
        self.thread_locals
            .push((Box::new(DispatcherThreadLocal(desc)) as Box<dyn ConsumeDesc>));
    }

    pub fn with_thread_local<T: FnOnce(&mut World, &mut Resources) -> Box<dyn ThreadLocal> + 'a>(
        mut self,
        desc: T,
    ) -> Self {
        self.add_thread_local(desc);

        self
    }

    pub fn add_thread_local_system<T: FnOnce(&mut World, &mut Resources) -> Box<dyn Runnable> + 'a>(
        &mut self,
        desc: T,
    ) {
        self.thread_locals
            .push((Box::new(DispatcherThreadLocalSystem(desc)) as Box<dyn ConsumeDesc>));
    }

    pub fn with_thread_local_system<T: FnOnce(&mut World, &mut Resources) -> Box<dyn Runnable> + 'a>(
        mut self,
        desc: T,
    ) -> Self {
        self.add_thread_local_system(desc);

        self
    }

    pub fn add_system<S: IntoRelativeStage, T: FnOnce(&mut World, &mut Resources) -> Box<dyn Schedulable> + 'a>(
        &mut self,
        stage: S,
        desc: T,
    ) {
        self.systems.push((
            stage.into_relative(),
            Box::new(DispatcherSystem(stage.into_relative(), desc)) as Box<dyn ConsumeDesc>,
        ));
    }

    pub fn with_system<S: IntoRelativeStage, T: FnOnce(&mut World, &mut Resources) -> Box<dyn Schedulable> + 'a>(
        mut self,
        stage: S,
        desc: T,
    ) -> Self {
        self.add_system(stage, desc);

        self
    }

    pub fn add_bundle<T: SystemBundle + 'a>(&mut self, bundle: T) {
        self.bundles
            .push(Box::new(DispatcherSystemBundle(bundle)) as Box<dyn ConsumeDesc>);
    }

    pub fn with_bundle<T: SystemBundle + 'a>(mut self, bundle: T) -> Self {
        self.add_bundle(bundle);

        self
    }

    pub fn with_defrag_budget(mut self, budget: Option<usize>) -> Self {
        self.defrag_budget = budget;

        self
    }

    pub fn with_pool(mut self, pool: Option<ArcThreadPool>) -> Self {
        self.thread_pool = pool;

        self
    }

    pub fn is_empty(&self) -> bool {
        self.systems.is_empty() && self.bundles.is_empty()
    }

    fn build_data(&mut self, world: &mut World, resources: &mut Resources) -> DispatcherData {
        let mut dispatcher_data = DispatcherData::default();

        for bundle in self.bundles.drain(..) {
            let mut recursive_builder = DispatcherBuilder::default();
            bundle
                .consume(world, resources, &mut dispatcher_data, &mut recursive_builder)
                .unwrap();
            dispatcher_data = dispatcher_data.merge(recursive_builder.build_data(world, resources));
        }

        for desc in self.systems.drain(..) {
            let mut recursive_builder = DispatcherBuilder::default();
            desc.1
                .consume(world, resources, &mut dispatcher_data, &mut recursive_builder)
                .unwrap();
            dispatcher_data = dispatcher_data.merge(recursive_builder.build_data(world, resources));
        }

        for desc in self.thread_locals.drain(..) {
            let mut recursive_builder = DispatcherBuilder::default();
            desc.consume(world, resources, &mut dispatcher_data, &mut recursive_builder)
                .unwrap();
            dispatcher_data = dispatcher_data.merge(recursive_builder.build_data(world, resources));
        }

        dispatcher_data
    }

    pub fn build(mut self, world: &mut World, resources: &mut Resources) -> Dispatcher {
        self.build_data(world, resources).flatten()
    }
}