use super::*;
use crate::{
    legion::{StageExecutor, World},
    transform::Transform,
    ArcThreadPool, SystemBundle as SpecsSystemBundle, Time,
};
use amethyst_error::Error;
use legion::schedule::Schedulable;
use std::collections::HashMap;

pub trait ConsumeDesc {
    fn consume(
        self: Box<Self>,
        world: &mut legion::world::World,
        stages: &mut Dispatcher,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), amethyst_error::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Stage {
    Begin,
    Logic,
    Render,
    ThreadLocal,
}

pub struct Dispatcher {
    pub thread_locals: Vec<Box<dyn ThreadLocal>>,
    pub stages: HashMap<Stage, Vec<Box<dyn legion::schedule::Schedulable>>>,
}
impl Default for Dispatcher {
    fn default() -> Self {
        use std::iter::FromIterator;

        Self {
            thread_locals: Vec::default(),
            stages: vec![
                (Stage::Begin, Vec::default()),
                (Stage::Logic, Vec::default()),
                (Stage::Render, Vec::default()),
            ]
            .into_iter()
            .collect(),
        }
    }
}
impl Dispatcher {
    pub fn run(&mut self, stage: Stage, world: &mut World) {
        match stage {
            Stage::ThreadLocal => {
                self.thread_locals
                    .iter_mut()
                    .for_each(|local| local.run(world));
            }
            _ => {
                StageExecutor::new(
                    &mut self.stages.get_mut(&stage).unwrap(),
                    &world.resources.get::<ArcThreadPool>().unwrap(),
                )
                .execute(world);
            }
        }
    }

    pub fn merge(mut self, mut other: Dispatcher) -> Self {
        self.thread_locals.extend(other.thread_locals.drain(..));
        for (k, v) in self.stages.iter_mut() {
            v.extend(other.stages.get_mut(k).unwrap().drain(..));
        }

        self
    }
}

#[derive(Default)]
pub struct DispatcherBuilder<'a> {
    systems: Vec<(Stage, Box<dyn ConsumeDesc + 'a>)>,
    thread_locals: Vec<Box<dyn ConsumeDesc + 'static>>,
    bundles: Vec<Box<dyn ConsumeDesc + 'a>>,
}
impl<'a> DispatcherBuilder<'a> {
    pub fn add_thread_local<D: ThreadLocal + 'static>(&mut self, system: D) {
        self.thread_locals
            .push(Box::new(DispatcherThreadLocal(system)));
    }

    pub fn add_system<D: FnOnce(&mut World) -> Box<dyn Schedulable> + 'a>(
        &mut self,
        stage: Stage,
        desc: D,
    ) {
        self.systems.push((
            stage,
            Box::new(DispatcherSystem(stage, desc)) as Box<dyn ConsumeDesc>,
        ));
    }

    pub fn add_bundle<D: SystemBundle + 'a>(&mut self, bundle: D) {
        self.bundles
            .push(Box::new(DispatcherSystemBundle(bundle)) as Box<dyn ConsumeDesc>);
    }

    pub fn with_thread_local<D: ThreadLocal + 'static>(mut self, system: D) -> Self {
        self.add_thread_local(system);

        self
    }

    pub fn with_system<D: FnOnce(&mut World) -> Box<dyn Schedulable> + 'a>(
        mut self,
        stage: Stage,
        desc: D,
    ) -> Self {
        self.add_system(stage, desc);

        self
    }

    pub fn with_bundle<D: SystemBundle + 'a>(mut self, bundle: D) -> Self {
        self.add_bundle(bundle);

        self
    }

    pub fn is_empty(&self) -> bool {
        self.systems.is_empty() && self.bundles.is_empty() && self.thread_locals.is_empty()
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

        // TODO: We need to recursively iterate any newly added bundles
        if !recursive_builder.is_empty() {
            dispatcher.merge(recursive_builder.build(world))
        } else {
            dispatcher
        }
    }
}
