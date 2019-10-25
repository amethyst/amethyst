use super::*;
use crate::{
    legion::{StageExecutor, World},
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
        stages: &mut Dispatcher,
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

pub struct ThreadLocalObject<S, F, D>(S, F, D);
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

#[derive(Default)]
pub struct Dispatcher {
    pub defrag_budget: Option<usize>,
    pub(crate) thread_local_systems: Vec<Box<dyn legion::schedule::Runnable>>,
    pub(crate) thread_locals: Vec<Box<dyn ThreadLocal>>,
    pub(crate) stages: BTreeMap<RelativeStage, Vec<Box<dyn legion::schedule::Schedulable>>>,
    sorted_systems: Vec<Box<dyn legion::schedule::Schedulable>>,
}
impl Dispatcher {
    pub fn finalize(mut self) -> Self {
        let mut sorted_systems = self.sorted_systems;
        self.stages
            .into_iter()
            .for_each(|(_, mut v)| v.drain(..).for_each(|sys| sorted_systems.push(sys)));

        log::trace!("Sorted {} systems", sorted_systems.len());
        if log::log_enabled!(log::Level::Trace) {
            sorted_systems.iter().for_each(|system| {
                log::trace!("System: {}", system.name());
            });
        }

        Self {
            defrag_budget: self.defrag_budget,
            thread_local_systems: self.thread_local_systems,
            thread_locals: self.thread_locals,
            stages: BTreeMap::default(),
            sorted_systems,
        }
    }
    pub fn run(&mut self, world: &mut World) {
        if let Some(thread_pool) = world.resources.get::<ArcThreadPool>() {
            let thread_pool = thread_pool.clone();
            StageExecutor::new(self.sorted_systems.as_mut_slice(), &thread_pool).execute(world);

            self.thread_local_systems
                .iter_mut()
                .for_each(|local| local.run(world));

            self.thread_locals
                .iter_mut()
                .for_each(|local| local.run(world));

            world.defrag(self.defrag_budget);
        }
    }

    pub fn merge(mut self, mut other: Dispatcher) -> Self {
        self.thread_local_systems
            .extend(other.thread_local_systems.drain(..));
        self.thread_locals.extend(other.thread_locals.drain(..));

        for (k, mut v) in other.stages.iter_mut() {
            self.stages
                .entry(*k)
                .or_insert_with(Vec::default)
                .extend(v.drain(..))
        }

        self
    }

    pub fn dispose(mut self, world: &mut World) {
        self.thread_local_systems
            .drain(..)
            .for_each(|local| local.dispose(world));

        self.thread_locals
            .drain(..)
            .for_each(|local| local.dispose(world));

        self.sorted_systems
            .into_iter()
            .for_each(|system| system.dispose(world));
    }
}

pub struct DispatcherBuilder<'a> {
    pub(crate) defrag_budget: Option<usize>,
    pub(crate) systems: Vec<(RelativeStage, Box<dyn ConsumeDesc + 'a>)>,
    pub(crate) thread_local_systems: Vec<Box<dyn ConsumeDesc + 'a>>,
    pub(crate) thread_locals: Vec<Box<dyn ConsumeDesc + 'a>>,
    pub(crate) bundles: Vec<Box<dyn ConsumeDesc + 'a>>,
}
impl<'a> Default for DispatcherBuilder<'a> {
    // We preallocate 128 for these, as its just a random round number but they are just fat-pointers so whatever
    fn default() -> Self {
        Self {
            defrag_budget: None,
            systems: Vec::with_capacity(128),
            thread_local_systems: Vec::with_capacity(128),
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
        self.thread_local_systems
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
        self.systems.is_empty() && self.bundles.is_empty() && self.thread_local_systems.is_empty()
    }

    pub fn build(mut self, world: &mut legion::world::World) -> Dispatcher {
        let mut dispatcher = Dispatcher::default();

        let mut recursive_builder = DispatcherBuilder::default();
        for desc in self.systems.drain(..) {
            desc.1
                .consume(world, &mut dispatcher, &mut recursive_builder)
                .unwrap();
        }

        for bundle in self.bundles.drain(..) {
            bundle
                .consume(world, &mut dispatcher, &mut recursive_builder)
                .unwrap();
        }

        for desc in self.thread_locals.drain(..) {
            desc.consume(world, &mut dispatcher, &mut recursive_builder)
                .unwrap();
        }

        for desc in self.thread_local_systems.drain(..) {
            desc.consume(world, &mut dispatcher, &mut recursive_builder)
                .unwrap();
        }

        // TODO: We need to recursively iterate any newly added bundles
        dispatcher.defrag_budget = self.defrag_budget;
        if !recursive_builder.is_empty() {
            dispatcher.merge(recursive_builder.build(world))
        } else {
            dispatcher
        }
    }
}
