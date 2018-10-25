#![recursion_limit = "256"]

extern crate amethyst_core;
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{Data, DeriveInput, Ident, Meta, NestedMeta, Type};

#[proc_macro_derive(EventReader, attributes(reader))]
pub fn event_reader_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let gen = impl_event_reader(&ast);
    gen.into()
}

fn impl_event_reader(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let event_name = &ast.ident;

    let mut reader_name: Option<Ident> = None;
    for meta in ast
        .attrs
        .iter()
        .filter(|attr| attr.path.segments[0].ident == "reader")
        .map(|attr| {
            attr.interpret_meta()
                .expect("reader attribute incorrectly defined")
        }) {
        match meta {
            Meta::List(l) => {
                for nested_meta in l.nested.iter() {
                    match *nested_meta {
                        NestedMeta::Meta(Meta::Word(ref word)) => {
                            reader_name = Some(word.clone());
                        }
                        _ => panic!("reader attribute does not contain a single name"),
                    }
                }
            }
            _ => (),
        };
    }

    let reader_name = reader_name.expect(&format!(
        r#"
#[derive(EventReader)] requested for {}, but #[reader(SomeEventReader)] attribute is missing

Example usage:
#[derive(EventReader)]
#[reader(SomeEventReader)]
pub enum SomeEvent {{
    One(Event1),
    Two(Event2),
}}
"#,
        event_name
    ));

    let tys = collect_field_types(&ast.data);
    let tys = &tys;
    let names = collect_variant_names(&ast.data);
    let names = &names;

    let reads : Vec<_> = (0..tys.len()).map(|n| {
        let variant = &names[n];
        quote! {
            events.extend(data.#n.read(self.#n.as_mut().expect("ReaderId undefined, has setup been run?")).cloned().map(|e| #event_name::#variant(e)));
        }
    }).collect();
    let setups: Vec<_> = (0..tys.len())
        .map(|n| {
            let ty = &tys[n];
            quote! {
                self.#n = Some(res.fetch_mut::<EventChannel<#ty>>().register_reader());
            }
        }).collect();
    quote! {
        #[allow(missing_docs)]
        #[derive(Default)]
        pub struct #reader_name(
            #(Option<ReaderId<#tys>>, )*
        );

        impl<'a> EventReader<'a> for #reader_name {
            type SystemData = (
                #(Read<'a, EventChannel<#tys>>),*
            );
            type Event = #event_name;

            fn read(&mut self, data: Self::SystemData, events: &mut Vec<#event_name>) {
                #(#reads)*
            }

            fn setup(&mut self, res: &mut Resources) {
                <Self::SystemData as SystemData<'a>>::setup(res);
                #(#setups)*
            }
        }
    }
}

fn collect_field_types(ast: &Data) -> Vec<Type> {
    let variants = match *ast {
        Data::Enum(ref variants) => &variants.variants,
        _ => panic!("EventReader derive only support enums"),
    };
    variants
        .iter()
        .map(|v| {
            v.fields
                .iter()
                .next()
                .expect("Event enum variant does not contain an inner event type")
                .ty
                .clone()
        }).collect()
}

fn collect_variant_names(ast: &Data) -> Vec<Ident> {
    let variants = match *ast {
        Data::Enum(ref variants) => &variants.variants,
        _ => panic!("EventReader derive only support enums"),
    };
    variants.iter().map(|v| v.ident.clone()).collect()
}
