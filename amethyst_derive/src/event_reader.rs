use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, GenericParam, Ident, Lifetime, LifetimeDef, Meta, NestedMeta, Type};

pub fn impl_event_reader(ast: &DeriveInput) -> TokenStream {
    let event_name = &ast.ident;

    let mut reader_name: Option<Ident> = None;
    for meta in ast
        .attrs
        .iter()
        .filter(|attr| attr.path.segments[0].ident == "reader")
        .map(|attr| {
            attr.interpret_meta()
                .expect("reader attribute incorrectly defined")
        })
    {
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
    let mut generics = ast.generics.clone();

    let reader_lifetime = Lifetime::new("'r", Span::call_site());
    // Need to wrap the lifetime in an iterator because the event channel resources are tokenized in
    // a cycle.
    let reader_lifetime_iter = std::iter::repeat(reader_lifetime.clone()).take(tys.len());
    generics
        .params
        .push(GenericParam::Lifetime(LifetimeDef::new(
            reader_lifetime.clone(),
        )));

    let (impl_generics, _, _) = generics.split_for_impl();

    let reads: Vec<_> = (0..tys.len())
        .map(|n| {
            let variant = &names[n];
            let tuple_index = Literal::usize_unsuffixed(n);
            quote! {
                events.extend(
                    data.#tuple_index.read(
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
                self.#tuple_index = Some(res.fetch_mut::<EventChannel<#ty>>().register_reader());
            }
        })
        .collect();
    quote! {
        #[allow(missing_docs)]
        #[derive(Default)]
        pub struct #reader_name <#(#type_params_with_defaults),*> (
            #(Option<ReaderId<#tys>>, )*
        ) #where_clause;

        impl #impl_generics EventReader<#reader_lifetime> for #reader_name #type_generics
        #where_clause
        {
            type SystemData = (
                #(Read<#reader_lifetime_iter, EventChannel<#tys>>),*
            );
            type Event = #event_name #type_generics;

            fn read(&mut self, data: Self::SystemData, events: &mut Vec<#event_name #type_generics>) {
                #(#reads)*
            }

            fn setup(&mut self, res: &mut Resources) {
                <Self::SystemData as SystemData<#reader_lifetime>>::setup(res);
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
