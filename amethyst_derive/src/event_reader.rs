//! EventReader Implementation

use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Ident, Meta, NestedMeta, Type};

pub fn impl_event_reader(ast: &DeriveInput) -> TokenStream {
    let event_name = &ast.ident;

    let mut reader_name: Option<Ident> = None;
    for meta in ast
        .attrs
        .iter()
        .filter(|attr| attr.path.segments[0].ident == "reader")
        .map(|attr| {
            attr.parse_meta()
                .expect("reader attribute incorrectly defined")
        })
    {
        if let Meta::List(l) = meta {
            for nested_meta in l.nested.iter() {
                match nested_meta {
                    NestedMeta::Meta(Meta::Path(path)) => {
                        if let Some(ident) = path.get_ident() {
                            reader_name = Some(ident.clone());
                        } else {
                            panic!("reader attribute does not contain a single name");
                        }
                    }
                    _ => panic!("reader attribute does not contain a single name"),
                }
            }
        };
    }

    let reader_name = reader_name.unwrap_or_else(|| {
        panic!(
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
        )
    });

    let tys = collect_field_types(&ast.data);
    let tys = &tys;
    let names = collect_variant_names(&ast.data);
    let names = &names;
    let type_params_with_defaults = ast.generics.type_params();
    let (_, type_generics, where_clause) = ast.generics.split_for_impl();

    let (impl_generics, _, _) = ast.generics.split_for_impl();

    let reads: Vec<_> = (0..tys.len())
        .map(|n| {
            let ty = &tys[n];
            let variant = &names[n];
            let tuple_index = Literal::usize_unsuffixed(n);
            quote! {
                events.extend(
                    resources.get::<EventChannel<#ty>>().unwrap().read(
                        self.#tuple_index
                            .as_mut()
                            .expect("ReaderId undefined, has setup been run?")
                        )
                    .cloned()
                    .map(#event_name::#variant)
                );
            }
        })
        .collect();
    let setups: Vec<_> = (0..tys.len())
        .map(|n| {
            let ty = &tys[n];
            let tuple_index = Literal::usize_unsuffixed(n);
            quote! {
                self.#tuple_index = Some(resources.get_mut_or_default::<EventChannel<#ty>>().register_reader());
            }
        })
        .collect();
    quote! {
        #[allow(missing_docs)]
        #[derive(Default)]
        pub struct #reader_name <#(#type_params_with_defaults),*> (
            #(Option<ReaderId<#tys>>, )*
        ) #where_clause;

        impl #impl_generics EventReader for #reader_name #type_generics
        #where_clause
        {
            type Event = #event_name #type_generics;

            fn read(&mut self, resources: &mut Resources, events: &mut Vec<#event_name #type_generics>) {
                #(#reads)*
            }

            fn setup(&mut self, resources: &mut Resources) {
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
        })
        .collect()
}

fn collect_variant_names(ast: &Data) -> Vec<Ident> {
    let variants = match *ast {
        Data::Enum(ref variants) => &variants.variants,
        _ => panic!("EventReader derive only support enums"),
    };
    variants.iter().map(|v| v.ident.clone()).collect()
}
