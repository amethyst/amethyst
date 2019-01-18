use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Generics, Ident, Meta, NestedMeta, Type};

pub fn impl_prefab_data(ast: &DeriveInput) -> TokenStream {
    if have_component_attribute(&ast.attrs[..]) {
        impl_prefab_data_component(ast)
    } else {
        impl_prefab_data_aggregate(ast)
    }
}

fn impl_prefab_data_component(ast: &DeriveInput) -> TokenStream {
    let base = &ast.ident;
    let (_, ty_generics, where_clause) = ast.generics.split_for_impl();
    let lf_tokens = gen_def_lt_tokens(&ast.generics);
    let ty_tokens = gen_def_ty_params(&ast.generics);

    quote! {
        impl<'pfd, #lf_tokens #ty_tokens> PrefabData<'pfd> for #base #ty_generics #where_clause {
            type SystemData = WriteStorage<'pfd, #base #ty_generics>;
            type Result = ();

            fn add_to_entity(&self,
                             entity: Entity,
                             system_data: &mut Self::SystemData,
                             _: &[Entity]) -> ::std::result::Result<(), PrefabError> {
                system_data.insert(entity, self.clone()).map(|_| ())
            }
        }
    }
}

fn impl_prefab_data_aggregate(ast: &DeriveInput) -> TokenStream {
    let base = &ast.ident;
    let data = collect_field_data(&ast.data);

    let system_datas = data.iter().map(|(ty, _, is_component)| {
        if *is_component {
            quote! {
                WriteStorage<'pfd, #ty>
            }
        } else {
            quote! {
                <#ty as PrefabData<'pfd>>::SystemData
            }
        }
    });
    let adds = (0..data.len()).map(|n| {
        let (_, name, is_component) = &data[n];
        if *is_component {
            quote! {
                system_data.#n.insert(entity, self.#name.clone())?;
            }
        } else {
            quote! {
                self.#name.add_to_entity(entity, &mut system_data.#n, entities)?;
            }
        }
    });
    let subs = (0..data.len()).filter_map(|n| {
        let (_, name, is_component) = &data[n];
        if *is_component {
            None
        } else {
            Some(quote! {
                if self.#name.load_sub_assets(progress, &mut system_data.#n)? {
                    ret = true;
                }
            })
        }
    });

    let (_, ty_generics, where_clause) = ast.generics.split_for_impl();
    let lf_tokens = gen_def_lt_tokens(&ast.generics);
    let ty_tokens = gen_def_ty_params(&ast.generics);

    quote! {
        impl<'pfd, #lf_tokens #ty_tokens> PrefabData<'pfd> for #base #ty_generics #where_clause {
            type SystemData = (
                #(#system_datas,)*
            );
            type Result = ();

            fn add_to_entity(&self,
                             entity: Entity,
                             system_data: &mut Self::SystemData,
                             entities: &[Entity]) -> ::std::result::Result<(), PrefabError> {
                #(#adds)*
                Ok(())
            }

            fn load_sub_assets(&mut self,
                               progress: &mut ProgressCounter,
                               system_data: &mut Self::SystemData) -> ::std::result::Result<bool, PrefabError> {
                let mut ret = false;
                #(#subs)*
                Ok(ret)
            }
        }
    }
}

fn collect_field_data(ast: &Data) -> Vec<(Type, Ident, bool)> {
    match *ast {
        Data::Struct(ref s) => s
            .fields
            .iter()
            .map(|f| {
                (
                    f.ty.clone(),
                    f.ident
                        .as_ref()
                        .expect("PrefabData derive only support named fields")
                        .clone(),
                    have_component_attribute(&f.attrs[..]),
                )
            })
            .collect(),
        _ => panic!("PrefabData aggregate derive only support structs"),
    }
}

fn gen_def_lt_tokens(generics: &Generics) -> TokenStream {
    let lts: Vec<_> = generics
        .lifetimes()
        .map(|x| {
            let ref lt = x.lifetime;
            let ref bounds = x.bounds;

            if bounds.is_empty() {
                quote! { #lt }
            } else {
                quote! { #lt: #( #bounds )+* }
            }
        })
        .collect();

    quote! { #( #lts ),* }
}

fn gen_def_ty_params(generics: &Generics) -> TokenStream {
    let ty_params: Vec<_> = generics
        .type_params()
        .map(|x| {
            let ref ty = x.ident;
            let ref bounds = x.bounds;

            quote! { #ty: #( #bounds )+* }
        })
        .collect();

    quote! { #( #ty_params ),* }
}

fn have_component_attribute(attrs: &[Attribute]) -> bool {
    for meta in attrs
        .iter()
        .filter(|attr| attr.path.segments[0].ident == "prefab")
        .map(|attr| {
            attr.interpret_meta()
                .expect("prefab attribute incorrectly defined")
        })
    {
        match meta {
            Meta::List(l) => {
                for nested_meta in l.nested.iter() {
                    match *nested_meta {
                        NestedMeta::Meta(Meta::Word(ref word)) => {
                            if word == "Component" {
                                return true;
                            }
                        }
                        _ => panic!("prefab attribute does not contain a single word value"),
                    }
                }
            }
            _ => (),
        };
    }
    false
}
