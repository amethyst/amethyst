#![allow(
    clippy::unneeded_field_pattern,
    clippy::block_in_if_condition_stmt,
    clippy::unneeded_field_pattern
)]
use amethyst_assets::ProgressCounter;
use amethyst_core::{
    ecs::{Read, World, WriteStorage},
    shrev::{EventChannel, ReaderId},
    EventReader,
};
use amethyst_derive::EventReader;
use amethyst_error::Error;

#[derive(Clone)]
pub struct TestEvent1;

#[derive(Clone)]
pub struct TestEvent2;

#[derive(Clone)]
pub struct TestEvent3<T>(T);

#[derive(Clone, EventReader)]
#[reader(TestEventReader)]
pub enum TestEvent {
    One(TestEvent1),
    Two(TestEvent2),
}

#[derive(Clone, EventReader)]
#[reader(TestEventWithTypeParameterReader)]
pub enum TestEventWithTypeParameter<T1, T2>
where
    T1: Clone + Send + Sync + 'static,
    T2: Clone + Send + Sync + 'static,
{
    One(TestEvent1),
    Two(TestEvent2),
    Three(TestEvent3<T1>),
    Four(TestEvent3<T2>),
}
