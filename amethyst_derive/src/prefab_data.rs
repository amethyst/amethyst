use proc_macro2::TokenStream;
use syn::{Data, DeriveInput, Generics, Ident, Meta, NestedMeta, Type};

pub fn impl_prefab_data(ast: &DeriveInput) -> TokenStream {
    let mut component = false;
    for meta in ast
        .attrs
        .iter()
        .filter(|attr| attr.path.segments[0].ident == "prefab")
        .map(|attr| {
            attr.interpret_meta()
                .expect("reader attribute incorrectly defined")
        }) {
        match meta {
            Meta::List(l) => {
                for nested_meta in l.nested.iter() {
                    match *nested_meta {
                        NestedMeta::Meta(Meta::Word(ref word)) => {
                            if word == "Component" {
                                component = true;
                            }
                        }
                        _ => panic!("reader attribute does not contain a single name"),
                    }
                }
            }
            _ => (),
        };
    }

    if component {
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
                             _: &[Entity]) -> Result<(), PrefabError> {
                system_data.insert(entity, self.clone()).map(|_| ())
            }
        }
    }
}

fn impl_prefab_data_aggregate(ast: &DeriveInput) -> TokenStream {
    let base = &ast.ident;
    let tys = collect_field_types(&ast.data);
    let tys = &tys;
    let names = collect_field_names(&ast.data);
    let names = &names;

    let system_datas: Vec<_> = (0..tys.len())
        .map(|n| {
            let ty = &tys[n];
            quote! {
                <#ty as PrefabData<'pfd>>::SystemData
            }
        }).collect();
    let adds: Vec<_> = (0..tys.len())
        .map(|n| {
            let name = &names[n];
            quote! {
                self.#name.add_to_entity(entity, &mut system_data.#n, entities)?;
            }
        }).collect();
    let subs: Vec<_> = (0..tys.len())
        .map(|n| {
            let name = &names[n];
            quote! {
                if self.#name.load_sub_assets(progress, &mut system_data.#n)? {
                    ret = true;
                }
            }
        }).collect();

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
                             entities: &[Entity]) -> Result<(), PrefabError> {
                #(#adds)*
                Ok(())
            }

            fn load_sub_assets(&mut self,
                               progress: &mut ProgressCounter,
                               system_data: &mut Self::SystemData) -> Result<bool, PrefabError> {
                let mut ret = false;
                #(#subs)*
                Ok(ret)
            }
        }
    }
}

fn collect_field_types(ast: &Data) -> Vec<Type> {
    match *ast {
        Data::Struct(ref s) => s.fields.iter().map(|f| f.ty.clone()).collect(),
        _ => panic!("PrefabData derive only support structs"),
    }
}

fn collect_field_names(ast: &Data) -> Vec<Ident> {
    match *ast {
        Data::Struct(ref s) => s
            .fields
            .iter()
            .map(|f| {
                f.ident
                    .as_ref()
                    .expect("PrefabData derive only support named fieldds")
                    .clone()
            }).collect(),
        _ => panic!("PrefabData derive only support structs"),
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
        }).collect();

    quote! { #( #lts ),* }
}

fn gen_def_ty_params(generics: &Generics) -> TokenStream {
    let ty_params: Vec<_> = generics
        .type_params()
        .map(|x| {
            let ref ty = x.ident;
            let ref bounds = x.bounds;

            quote! { #ty: #( #bounds )+* }
        }).collect();

    quote! { #( #ty_params ),* }
}
