#![recursion_limit = "256"]

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

#[proc_macro_derive(EventReader, attributes(reader))]
pub fn event_reader_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let gen = event_reader::impl_event_reader(&ast);
    gen.into()
}

#[proc_macro_derive(PrefabData, attributes(prefab))]
pub fn prefab_data_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let gen = prefab_data::impl_prefab_data(&ast);
    gen.into()
}
