use super::*;
use crate::{legion::World, transform::Transform, SystemBundle as SpecsSystemBundle, Time};
use amethyst_error::Error;
use legion::system::Schedulable;
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
    pub thread_locals: Vec<Box<dyn ThreadLocalSystem>>,
    pub stages: HashMap<Stage, Vec<Box<dyn legion::system::Schedulable>>>,
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
                legion::system::StageExecutor::new(&mut self.stages.get_mut(&stage).unwrap())
                    .execute(world);
            }
        }
    }
}

#[derive(Default)]
pub struct DispatcherBuilder {
    systems: Vec<(Stage, Box<dyn ConsumeDesc>)>,
    bundles: Vec<Box<dyn ConsumeDesc>>,
    thread_locals: Vec<Box<dyn ThreadLocalSystem>>,
}
impl DispatcherBuilder {
    pub fn add_thread_local<D: ThreadLocalSystem + 'static>(&mut self, system: D) {
        self.thread_locals.push(Box::new(system));
    }

    pub fn add_system_desc<D: SystemDesc + 'static>(&mut self, stage: Stage, desc: D) {
        self.systems.push((
            stage,
            Box::new(DispatcherSystemDesc(stage, desc)) as Box<dyn ConsumeDesc>,
        ));
    }

    pub fn add_bundle<D: SystemBundle + 'static>(&mut self, bundle: D) {
        self.bundles
            .push(Box::new(DispatcherSystemBundle(bundle)) as Box<dyn ConsumeDesc>);
    }

    pub fn with_thread_local<D: ThreadLocalSystem + 'static>(mut self, system: D) -> Self {
        self.thread_locals.push(Box::new(system));

        self
    }

    pub fn with_system_desc<D: SystemDesc + 'static>(mut self, stage: Stage, desc: D) -> Self {
        self.systems.push((
            stage,
            Box::new(DispatcherSystemDesc(stage, desc)) as Box<dyn ConsumeDesc>,
        ));

        self
    }

    pub fn with_bundle<D: SystemBundle + 'static>(mut self, bundle: D) -> Self {
        self.bundles
            .push(Box::new(DispatcherSystemBundle(bundle)) as Box<dyn ConsumeDesc>);

        self
    }

    pub fn build(mut self, world: &mut legion::world::World) -> Dispatcher {
        let mut dispatcher = Dispatcher::default();

        let mut recursive_builder = DispatcherBuilder::default();

        for desc in self.systems.drain(..) {
            desc.1
                .consume(world, &mut dispatcher, &mut recursive_builder)
                .unwrap()
        }

        for bundle in self.bundles.drain(..) {
            bundle
                .consume(world, &mut dispatcher, &mut recursive_builder)
                .unwrap()
        }

        dispatcher
            .thread_locals
            .extend(self.thread_locals.drain(..));

        dispatcher
        //recursive_builder.build(world)
    }
}
