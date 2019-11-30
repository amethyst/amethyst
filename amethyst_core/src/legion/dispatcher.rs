use super::*;
use crate::{
    legion::{Executor, World},
    transform::Transform,
    ArcThreadPool, SystemBundle as SpecsSystemBundle, Time,
};
use amethyst_error::Error;
use legion::schedule::Schedulable;
use std::collections::BTreeMap;

pub trait ConsumeDesc {
    fn consume(
        self: Box<Self>,
        world: &mut legion::world::World,
        stages: &mut DispatcherData,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), amethyst_error::Error>;
}

pub trait ThreadLocal {
    fn run(&mut self, world: &mut World);
    fn dispose(self: Box<Self>, world: &mut World);
}

impl<F> ThreadLocal for F
where
    F: FnMut(&mut World) + 'static,
{
    fn run(&mut self, world: &mut World) {
        (self)(world)
    }
    fn dispose(self: Box<Self>, world: &mut World) {}
}

impl Into<Box<dyn ThreadLocal>> for Box<dyn Runnable> {
    fn into(self) -> Box<dyn ThreadLocal> {
        Box::new(move |world: &mut World| {
            self.run(world);
        })
    }
}

pub struct ThreadLocalObject<S, F, D>(pub S, pub F, pub D);
impl<S, F, D> ThreadLocalObject<S, F, D>
where
    S: 'static,
    F: FnMut(&mut S, &mut World) + 'static,
    D: FnOnce(S, &mut World) + 'static,
{
    pub fn build(initial_state: S, run_fn: F, dispose_fn: D) -> Box<dyn ThreadLocal> {
        Box::new(Self(initial_state, run_fn, dispose_fn))
    }
}
impl<S, F, D> ThreadLocal for ThreadLocalObject<S, F, D>
where
    S: 'static,
    F: FnMut(&mut S, &mut World) + 'static,
    D: FnOnce(S, &mut World) + 'static,
{
    fn run(&mut self, world: &mut World) {
        (self.1)(&mut self.0, world)
    }
    fn dispose(self: Box<Self>, world: &mut World) {
        (self.2)(self.0, world)
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
    pub fn run(&mut self, world: &mut World) {
        let thread_pool = world.resources.get::<ArcThreadPool>().unwrap().clone();

        self.executor.execute(world);

        self.thread_locals
            .iter_mut()
            .for_each(|local| local.run(world));

        //world.defrag(self.defrag_budget);
    }
    pub fn dispose(mut self, world: &mut World) {
        self.thread_locals
            .drain(..)
            .for_each(|local| local.dispose(world));

        self.executor
            .into_vec()
            .into_iter()
            .for_each(|system| system.dispose(world));
    }
}

#[derive(Default)]
pub struct DispatcherData {
    pub defrag_budget: Option<usize>,
    pub(crate) thread_locals: Vec<Box<dyn ThreadLocal>>,
    pub(crate) stages: BTreeMap<RelativeStage, Vec<Box<dyn legion::schedule::Schedulable>>>,
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
}
impl<'a> Default for DispatcherBuilder<'a> {
    // We preallocate 128 for these, as its just a random round number but they are just fat-pointers so whatever
    fn default() -> Self {
        Self {
            defrag_budget: None,
            systems: Vec::with_capacity(128),
            thread_locals: Vec::with_capacity(128),
            bundles: Vec::with_capacity(128),
        }
    }
}
impl<'a> DispatcherBuilder<'a> {
    pub fn add_thread_local<T: FnOnce(&mut World) -> Box<dyn ThreadLocal> + 'a>(
        &mut self,
        desc: T,
    ) {
        self.thread_locals
            .push((Box::new(DispatcherThreadLocal(desc)) as Box<dyn ConsumeDesc>));
    }

    pub fn with_thread_local<T: FnOnce(&mut World) -> Box<dyn ThreadLocal> + 'a>(
        mut self,
        desc: T,
    ) -> Self {
        self.add_thread_local(desc);

        self
    }

    pub fn add_thread_local_system<T: FnOnce(&mut World) -> Box<dyn Runnable> + 'a>(
        &mut self,
        desc: T,
    ) {
        self.thread_locals
            .push((Box::new(DispatcherThreadLocalSystem(desc)) as Box<dyn ConsumeDesc>));
    }

    pub fn with_thread_local_system<T: FnOnce(&mut World) -> Box<dyn Runnable> + 'a>(
        mut self,
        desc: T,
    ) -> Self {
        self.add_thread_local_system(desc);

        self
    }

    pub fn add_system<S: IntoRelativeStage, T: FnOnce(&mut World) -> Box<dyn Schedulable> + 'a>(
        &mut self,
        stage: S,
        desc: T,
    ) {
        self.systems.push((
            stage.into_relative(),
            Box::new(DispatcherSystem(stage.into_relative(), desc)) as Box<dyn ConsumeDesc>,
        ));
    }

    pub fn with_system<S: IntoRelativeStage, T: FnOnce(&mut World) -> Box<dyn Schedulable> + 'a>(
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

    pub fn is_empty(&self) -> bool {
        self.systems.is_empty() && self.bundles.is_empty()
    }

    fn build_data(&mut self, world: &mut legion::world::World) -> DispatcherData {
        let mut dispatcher_data = DispatcherData::default();

        for bundle in self.bundles.drain(..) {
            let mut recursive_builder = DispatcherBuilder::default();
            bundle
                .consume(world, &mut dispatcher_data, &mut recursive_builder)
                .unwrap();
            dispatcher_data = dispatcher_data.merge(recursive_builder.build_data(world));
        }

        for desc in self.thread_locals.drain(..) {
            let mut recursive_builder = DispatcherBuilder::default();
            desc.consume(world, &mut dispatcher_data, &mut recursive_builder)
                .unwrap();
            dispatcher_data = dispatcher_data.merge(recursive_builder.build_data(world));
        }

        for desc in self.systems.drain(..) {
            let mut recursive_builder = DispatcherBuilder::default();
            desc.1
                .consume(world, &mut dispatcher_data, &mut recursive_builder)
                .unwrap();
            dispatcher_data = dispatcher_data.merge(recursive_builder.build_data(world));
        }

        dispatcher_data
    }

    pub fn build(mut self, world: &mut legion::world::World) -> Dispatcher {
        self.build_data(world).flatten()
    }
}
