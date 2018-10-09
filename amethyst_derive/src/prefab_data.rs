use proc_macro2::TokenStream;
use syn::{Data, DeriveInput, Ident, Type, Meta, NestedMeta};

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
    quote! {
        impl<'a> PrefabData<'a> for #base {
            type SystemData = Write<'a, #base>;
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
                <#ty as PrefabData<'a>>::SystemData
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
    quote! {
        impl<'a> PrefabData<'a> for #base {
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
