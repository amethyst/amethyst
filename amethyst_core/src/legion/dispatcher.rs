use legion::prelude::*;
use std::collections::BTreeMap;

pub trait SystemBundle {
    fn build(
        self,
        world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder<'_>,
    ) -> Result<(), amethyst_error::Error>;
}

impl<T> ConsumeDesc for T
where
    T: SystemBundle,
{
    fn consume(
        self: Box<Self>,
        world: &mut World,
        resources: &mut Resources,
        _: &mut DispatcherData,
        builder: &mut DispatcherBuilder<'_>,
    ) -> Result<(), amethyst_error::Error> {
        self.build(world, resources, builder)
    }
}

pub struct SystemBundleFn<F>(F);
impl<F> ConsumeDesc for SystemBundleFn<F>
where
    F: FnOnce(
        &mut World,
        &mut Resources,
        &mut DispatcherBuilder<'_>,
    ) -> Result<(), amethyst_error::Error>,
{
    fn consume(
        self: Box<Self>,
        world: &mut World,
        resources: &mut Resources,
        _: &mut DispatcherData,
        builder: &mut DispatcherBuilder<'_>,
    ) -> Result<(), amethyst_error::Error> {
        (self.0)(world, resources, builder)
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
            .push(DispatcherEntry::System(sys));

        Ok(())
    }
}

pub struct DispatcherThreadLocalSystem<F>(RelativeStage, F);
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
        let sys = (self.1)(world, resources);

        dispatcher
            .stages
            .entry(self.0)
            .or_insert_with(Vec::default)
            .push(DispatcherEntry::ThreadLocalSystem(sys));

        Ok(())
    }
}

pub struct DispatcherThreadLocal<F>(RelativeStage, F);
impl<F> ConsumeDesc for DispatcherThreadLocal<F>
where
    F: FnOnce(&mut World, &mut Resources) -> Box<dyn FnMut(&mut World, &mut Resources)>,
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
            .push(DispatcherEntry::ThreadLocal(sys));

        Ok(())
    }
}

pub struct DispatcherFlush(RelativeStage);
impl ConsumeDesc for DispatcherFlush {
    fn consume(
        self: Box<Self>,
        _: &mut World,
        _: &mut Resources,
        dispatcher: &mut DispatcherData,
        _: &mut DispatcherBuilder<'_>,
    ) -> Result<(), amethyst_error::Error> {
        dispatcher
            .stages
            .entry(self.0)
            .or_insert_with(Vec::default)
            .push(DispatcherEntry::Flush);

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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Stage {
    Begin,
    AI,
    Logic,
    Render,
    ThreadLocal,
    End,
}
impl Into<RelativeStage> for Stage {
    fn into(self) -> RelativeStage {
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
    schedule: Schedule,
    pub defrag_budget: Option<usize>,
}
impl Dispatcher {
    pub fn run(&mut self, world: &mut World, resources: &mut Resources) {
        self.schedule.execute(world, resources);

        world.defrag(self.defrag_budget);
    }
}

pub enum DispatcherEntry {
    System(Box<dyn Schedulable>),
    ThreadLocal(Box<dyn FnMut(&mut World, &mut Resources)>),
    ThreadLocalSystem(Box<dyn Runnable>),
    Flush,
}

#[derive(Default)]
pub struct DispatcherData {
    pub defrag_budget: Option<usize>,
    pub(crate) stages: BTreeMap<RelativeStage, Vec<DispatcherEntry>>,
}
impl DispatcherData {
    pub fn flatten(self) -> Result<Dispatcher, amethyst_error::Error> {
        let mut builder = Schedule::builder();
        for (_, mut v) in self.stages {
            for entry in v.drain(..) {
                match entry {
                    DispatcherEntry::System(sys) => builder = builder.add_system(sys),
                    DispatcherEntry::ThreadLocal(sys) => builder = builder.add_thread_local_fn(sys),
                    DispatcherEntry::ThreadLocalSystem(sys) => {
                        builder = builder.add_thread_local(sys)
                    }
                    DispatcherEntry::Flush => builder = builder.flush(),
                }
            }
        }
        Ok(Dispatcher {
            defrag_budget: self.defrag_budget,
            schedule: builder.build(),
        })
    }

    pub fn merge(mut self, mut other: Self) -> Self {
        for (k, v) in &mut other.stages {
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
    pub(crate) bundles: Vec<Box<dyn ConsumeDesc + 'a>>,
}
impl<'a> Default for DispatcherBuilder<'a> {
    // We preallocate 128 for these, as its just a random round number but they are just fat-pointers so whatever
    fn default() -> Self {
        Self {
            defrag_budget: None,
            systems: Vec::with_capacity(128),
            bundles: Vec::with_capacity(128),
        }
    }
}
impl<'a> DispatcherBuilder<'a> {
    pub fn add_flush<S: Copy + Into<RelativeStage>>(&mut self, stage: S) {
        self.systems.push((
            stage.into(),
            Box::new(DispatcherFlush(stage.into())) as Box<dyn ConsumeDesc>,
        ));
    }

    pub fn with_flush<S: Copy + Into<RelativeStage>>(mut self, stage: S) -> Self {
        self.add_flush(stage);

        self
    }

    pub fn add_thread_local_fn<
        S: Copy + Into<RelativeStage>,
        T: FnOnce(&mut World, &mut Resources) -> Box<dyn FnMut(&mut World, &mut Resources)> + 'a,
    >(
        &mut self,
        stage: S,
        desc: T,
    ) {
        self.systems.push((
            stage.into(),
            Box::new(DispatcherThreadLocal(stage.into(), desc)) as Box<dyn ConsumeDesc>,
        ));
    }

    pub fn with_thread_local_fn<
        S: Copy + Into<RelativeStage>,
        T: FnOnce(&mut World, &mut Resources) -> Box<dyn FnMut(&mut World, &mut Resources)> + 'a,
    >(
        mut self,
        stage: S,
        desc: T,
    ) -> Self {
        self.add_thread_local_fn(stage, desc);

        self
    }

    pub fn add_thread_local_system<
        S: Copy + Into<RelativeStage>,
        T: FnOnce(&mut World, &mut Resources) -> Box<dyn Runnable> + 'a,
    >(
        &mut self,
        stage: S,
        desc: T,
    ) {
        self.systems.push((
            stage.into(),
            Box::new(DispatcherThreadLocalSystem(stage.into(), desc)) as Box<dyn ConsumeDesc>,
        ));
    }

    pub fn with_thread_local_system<
        S: Copy + Into<RelativeStage>,
        T: FnOnce(&mut World, &mut Resources) -> Box<dyn Runnable> + 'a,
    >(
        mut self,
        stage: S,
        desc: T,
    ) -> Self {
        self.add_thread_local_system(stage, desc);

        self
    }

    pub fn add_system<
        S: Copy + Into<RelativeStage>,
        T: FnOnce(&mut World, &mut Resources) -> Box<dyn Schedulable> + 'a,
    >(
        &mut self,
        stage: S,
        desc: T,
    ) {
        self.systems.push((
            stage.into(),
            Box::new(DispatcherSystem(stage.into(), desc)) as Box<dyn ConsumeDesc>,
        ));
    }

    pub fn with_system<
        S: Copy + Into<RelativeStage>,
        T: FnOnce(&mut World, &mut Resources) -> Box<dyn Schedulable> + 'a,
    >(
        mut self,
        stage: S,
        desc: T,
    ) -> Self {
        self.add_system(stage, desc);

        self
    }

    pub fn add_bundle<
        F: FnOnce(
                &mut World,
                &mut Resources,
                &mut DispatcherBuilder<'_>,
            ) -> Result<(), amethyst_error::Error>
            + 'a,
    >(
        &mut self,
        bundle: F,
    ) {
        self.bundles
            .push(Box::new(SystemBundleFn(bundle)) as Box<dyn ConsumeDesc>);
    }

    pub fn with_bundle<
        F: FnOnce(
                &mut World,
                &mut Resources,
                &mut DispatcherBuilder<'_>,
            ) -> Result<(), amethyst_error::Error>
            + 'a,
    >(
        mut self,
        bundle: F,
    ) -> Self {
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

    fn build_data(&mut self, world: &mut World, resources: &mut Resources) -> DispatcherData {
        let mut dispatcher_data = DispatcherData::default();

        for bundle in self.bundles.drain(..) {
            let mut recursive_builder = DispatcherBuilder::default();
            bundle
                .consume(
                    world,
                    resources,
                    &mut dispatcher_data,
                    &mut recursive_builder,
                )
                .unwrap();
            dispatcher_data = dispatcher_data.merge(recursive_builder.build_data(world, resources));
        }

        for desc in self.systems.drain(..) {
            let mut recursive_builder = DispatcherBuilder::default();
            desc.1
                .consume(
                    world,
                    resources,
                    &mut dispatcher_data,
                    &mut recursive_builder,
                )
                .unwrap();
            dispatcher_data = dispatcher_data.merge(recursive_builder.build_data(world, resources));
        }

        dispatcher_data
    }

    pub fn build(
        mut self,
        world: &mut World,
        resources: &mut Resources,
    ) -> Result<Dispatcher, amethyst_error::Error> {
        self.build_data(world, resources).flatten()
    }
}
