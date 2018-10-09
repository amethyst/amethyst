#[macro_use]
extern crate amethyst_derive;
extern crate amethyst_core;
use amethyst_core::{
    shrev::{EventChannel, ReaderId},
    specs::{Read, Resources, SystemData},
    EventReader,
};

#[derive(Clone)]
pub struct TestEvent1;

#[derive(Clone)]
pub struct TestEvent2;

#[derive(Clone, EventReader)]
#[reader(TestEventReader)]
pub enum TestEvent {
    One(TestEvent1),
    Two(TestEvent2),
}
