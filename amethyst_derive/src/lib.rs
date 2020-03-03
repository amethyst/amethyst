//! This crate implements various derive macros for easing the use of various amethyst features.
//! At the moment, this consists of event readers, prefab and UI widget derives.

#![recursion_limit = "256"]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility,
    clippy::all
)]
// Needed because `nightly` warns on `extern crate proc_macro;`, but `stable` still requires it.
#![allow(unused_extern_crates)]

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod event_reader;
mod prefab_data;
mod system_desc;
mod widget_id;

/// EventReader
#[proc_macro_derive(EventReader, attributes(reader))]
pub fn event_reader_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let gen = event_reader::impl_event_reader(&ast);
    gen.into()
}

/// Deriving a `Prefab` requires that `amethyst::ecs::Entity`,
/// `amethyst:assets::{PrefabData, ProgressCounter}` and
/// `amethyst::error::Error` are imported and visible in the current scope. This
/// is due to how Rust macros work.
#[proc_macro_derive(PrefabData, attributes(prefab))]
pub fn prefab_data_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let gen = prefab_data::impl_prefab_data(&ast);
    gen.into()
}

/// This allows the use of an enum as an ID for the `Widgets` resource. One
/// variant has to be marked as the default variant with `#[widget_id_default]
/// and will be used when a `Widget` is added to the resource without an
/// explicit ID. Note that when using `Widgets::add`, this will overwrite
/// an existing widget with the same default id!
#[proc_macro_derive(WidgetId, attributes(widget_id))]
pub fn widget_id_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let gen = widget_id::impl_widget_id(&ast);
    gen.into()
}

/// Derive a `SystemDesc` implementation.
///
/// The `SystemDesc` is passed to the `GameData` to instantiate the `System` when building the
/// dispatcher.
///
/// This derive may be used for `System`s that do not require special code for `System::setup`.
#[proc_macro_derive(SystemDesc, attributes(system_desc))]
pub fn system_desc_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let gen = system_desc::impl_system_desc(&ast);
    gen.into()
}
