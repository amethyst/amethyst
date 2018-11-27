use proc_macro2::{Span, TokenStream};
use syn::{Data, DeriveInput, Fields, Ident};

/// Derive to implement the State trait.
pub fn impl_state(ast: &DeriveInput) -> TokenStream {
    let base = &ast.ident;
    let storage = Ident::new(&format!("{}StateStorage", base), Span::call_site());

    let en = match ast.data {
        Data::Enum(ref en) => en,
        _ => panic!("`State` attribute is only supported on enums"),
    };

    let callback = quote!(Box<dyn amethyst::StateCallback<#base, E>>);

    let mut fields = Vec::new();
    let mut get_mut_variants = Vec::new();
    let mut insert_variants = Vec::new();
    let mut names = Vec::new();

    let first = en
        .variants
        .iter()
        .next()
        .expect("enum must have at least one variant")
        .ident
        .clone();

    for (i, variant) in en.variants.iter().enumerate() {
        match variant.fields {
            Fields::Unit => {}
            _ => panic!("Only unit fields are supported in state enums"),
        }

        let field = Ident::new(&format!("f{}", i), Span::call_site());
        let var = &variant.ident;

        get_mut_variants.push(quote!(#base::#var => return self.#field.as_mut()));
        insert_variants.push(
            quote!(#base::#var => return ::std::mem::replace(&mut self.#field, Some(callback))),
        );
        fields.push(quote!(#field: Option<#callback>));
        names.push(field);
    }

    let names = &names;

    quote! {
        impl Default for #base {
            fn default() -> #base {
                #base::#first
            }
        }

        #[allow(non_camel_case_types)]
        pub struct #storage<E> {
            #(#fields,)*
        }

        impl<E> Default for #storage<E> {
            fn default() -> #storage<E> {
                #storage { #(#names: None,)* }
            }
        }

        impl<E> amethyst::StateStorage<#base, E> for #storage<E> {
            fn insert(
                &mut self,
                state: #base,
                callback: #callback,
            ) -> Option<#callback> {
                match state { #(#insert_variants,)* }
            }

            fn get_mut(&mut self, value: &#base) -> Option<&mut #callback> {
                match *value { #(#get_mut_variants,)* }
            }

            fn do_values<F>(&mut self, mut apply: F) where F: FnMut(&mut #callback) {
                #(
                if let Some(c) = self.#names.as_mut() {
                    apply(c);
                }
                )*
            }
        }

        impl<E> amethyst::State<E> for #base {
            type Storage = #storage<E>;
        }
    }
}
