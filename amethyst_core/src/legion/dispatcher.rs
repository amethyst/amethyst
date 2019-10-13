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

    pub fn merge(mut self, mut other: Dispatcher) -> Self {
        println!(
            "thread-local merging {} vs. {}",
            self.thread_locals.len(),
            other.thread_locals.len()
        );

        self.thread_locals.extend(other.thread_locals.drain(..));
        for (k, v) in self.stages.iter_mut() {
            v.extend(other.stages.get_mut(k).unwrap().drain(..));
        }

        self
    }
}

#[derive(Default)]
pub struct DispatcherBuilder {
    systems: Vec<(Stage, Box<dyn ConsumeDesc>)>,
    bundles: Vec<Box<dyn ConsumeDesc>>,
    pub thread_locals: Vec<Box<dyn ConsumeDesc>>,
}
impl DispatcherBuilder {
    pub fn add_thread_local<D: ThreadLocalDesc + 'static>(&mut self, system: D) {
        self.thread_locals
            .push(Box::new(DispatcherThreadLocalDesc(system)));
        println!("thread locals = {}", self.thread_locals.len());
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

    pub fn with_thread_local<D: ThreadLocalDesc + 'static>(mut self, system: D) -> Self {
        self.add_thread_local(system);

        self
    }

    pub fn with_system_desc<D: SystemDesc + 'static>(mut self, stage: Stage, desc: D) -> Self {
        self.add_system_desc(stage, desc);

        self
    }

    pub fn with_bundle<D: SystemBundle + 'static>(mut self, bundle: D) -> Self {
        self.add_bundle(bundle);

        self
    }

    pub fn is_empty(&self) -> bool {
        self.systems.is_empty() && self.bundles.is_empty() && self.thread_locals.is_empty()
    }

    pub fn build(mut self, world: &mut legion::world::World) -> Dispatcher {
        let mut dispatcher = Dispatcher::default();

        println!("BUILD systems loals  = {}", self.systems.len(),);
        println!("BUILD thread loals  = {}", self.thread_locals.len(),);

        let mut recursive_builder = DispatcherBuilder::default();
        println!("WTF WTF START BUILD");
        for desc in self.systems.drain(..) {
            desc.1
                .consume(world, &mut dispatcher, &mut recursive_builder)
                .unwrap();
        }

        for bundle in self.bundles.drain(..) {
            let mut test = DispatcherBuilder::default();
            println!("Consuming bundle...");
            bundle.consume(world, &mut dispatcher, &mut test).unwrap();
            println!(
                "BUNDLE thread loals  = {}, recursive={}",
                self.thread_locals.len(),
                test.thread_locals.len()
            );
        }

        for desc in self.thread_locals.drain(..) {
            desc.consume(world, &mut dispatcher, &mut recursive_builder)
                .unwrap();
        }

        println!(
            "BUILD thread recursive={}",
            recursive_builder.thread_locals.len()
        );
        println!(
            "BUILD systems  recursive={}",
            recursive_builder.systems.len()
        );

        // TODO: We need to recursively iterate any newly added bundles
        if !recursive_builder.is_empty() {
            println!("recurse");
            dispatcher.merge(recursive_builder.build(world))
        } else {
            dispatcher
        }
    }
}
