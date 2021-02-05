//! This crate implements various derive macros for easing the use of various amethyst features.
//! At the moment, this consists of event readers and UI widget derives.

#![doc(
    html_logo_url = "https://amethyst.rs/brand/logo-standard.svg",
    html_root_url = "https://docs.amethyst.rs/stable"
)]
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
use proc_macro2::{Ident, Span};
use syn::{parse_macro_input, DeriveInput};

mod event_reader;
mod widget_id;

/// EventReader
#[proc_macro_derive(EventReader, attributes(reader))]
pub fn event_reader_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let gen = event_reader::impl_event_reader(&ast);
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

use std::path::Path;

use glob::glob;
use quote::quote;

/// creates test out of every *.md file in the repo
#[proc_macro]
pub fn make_doc_tests(_: TokenStream) -> TokenStream {
    let doc_paths: Vec<String> = glob("**/*.md")
        .expect("bad glob")
        .map(|entry| "../".to_string() + &entry.unwrap().to_string_lossy())
        .collect();

    let idents = doc_paths
        .iter()
        .cloned()
        .map(|s| {
            Path::new(&s)
                .to_string_lossy()
                .into_owned()
                .replace("-", "_")
                .replace("/", "_")
                .replace("\\", "_")
                .replace(".", "")
        })
        .map(|s| Ident::new(&s, Span::call_site()));

    let expanded = quote! {
        #(doctest!(#doc_paths, #idents);)*
    };

    expanded.into()
}
