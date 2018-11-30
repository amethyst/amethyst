#![recursion_limit = "256"]
#![warn(rust_2018_idioms, rust_2018_compatibility)]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::DeriveInput;

mod event_reader;
mod prefab_data;
mod state;

#[proc_macro_derive(EventReader, attributes(reader))]
pub fn event_reader_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let gen = event_reader::impl_event_reader(&ast);
    gen.into()
}

/// Deriving a `Prefab` requires that `amethyst::ecs::Entity` and
/// `amethyst:assets::{PrefabData, PrefabError, ProgressCounter}` are imported
/// and visible in the current scope. This is due to how Rust macros work.
#[proc_macro_derive(PrefabData, attributes(prefab))]
pub fn prefab_data_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let gen = prefab_data::impl_prefab_data(&ast);
    gen.into()
}

/// Derive to implement the State trait.
///
/// Requires that `amethyst` is in scope.
#[proc_macro_derive(State, attributes(state))]
pub fn state_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let gen = state::impl_state(&ast);
    gen.into()
}
