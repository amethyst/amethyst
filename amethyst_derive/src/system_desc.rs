//! PrefabData Implementation

use heck::SnakeCase;
use proc_macro2::{Literal, Span, TokenStream};
use proc_macro_roids::{DeriveInputStructExt, FieldExt};
use quote::quote;
use syn::{
    parse_quote, Attribute, DeriveInput, Expr, Field, Fields, FieldsNamed, FieldsUnnamed,
    GenericParam, Ident, ImplGenerics, LifetimeDef, Lit, Meta, MetaList, NestedMeta, TypeGenerics,
    WhereClause,
};

pub fn impl_system_desc(ast: &DeriveInput) -> TokenStream {
    let system_name = &ast.ident;
    let system_desc_name = system_desc_name(&ast);

    // Whether the `SystemDesc` implementation is on the `System` type itself.
    let is_self = system_desc_name.is_none();
    let system_desc_name = system_desc_name.unwrap_or_else(|| system_name.clone());

    let (system_desc_fields, is_default) = if is_self {
        (None, false)
    } else {
        let system_desc_fields = system_desc_fields(&ast);
        let is_default = system_desc_fields.iter().all(FieldExt::is_phantom_data);

        (Some(system_desc_fields), is_default)
    };
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let context = Context {
        system_name,
        system_desc_name,
        system_desc_fields,
        impl_generics,
        ty_generics,
        where_clause,
        is_default,
        is_self,
    };

    let (system_desc_struct, constructor, call_system_constructor) = if is_self {
        (TokenStream::new(), TokenStream::new(), quote!(self))
    } else {
        (
            system_desc_struct(&context),
            impl_constructor(&context),
            call_system_constructor(&context),
        )
    };
    let resource_insertion_expressions = resource_insertion_expressions(&ast);

    let Context {
        system_name,
        system_desc_name,
        ty_generics,
        where_clause,
        ..
    } = context;

    let mut generics = ast.generics.clone();
    let system_desc_life_a: LifetimeDef = parse_quote!('system_desc_a);
    let system_desc_life_b: LifetimeDef = parse_quote!('system_desc_b);
    generics
        .params
        .push(GenericParam::from(system_desc_life_a.clone()));
    generics
        .params
        .push(GenericParam::from(system_desc_life_b.clone()));
    let (impl_generics_with_lifetimes, _, _) = generics.split_for_impl();

    quote! {
        #system_desc_struct

        #constructor

        impl #impl_generics_with_lifetimes
        SystemDesc<
            #system_desc_life_a,
            #system_desc_life_b,
            #system_name #ty_generics
        >
            for #system_desc_name #ty_generics
        #where_clause
        {
            fn build(self, world: &mut World) -> #system_name #ty_generics {
                <#system_name #ty_generics as System<'_>>::SystemData::setup(world);

                #resource_insertion_expressions

                #call_system_constructor
            }
        }
    }
}

fn system_desc_struct(context: &Context<'_>) -> TokenStream {
    let Context {
        ref system_name,
        ref system_desc_name,
        ref system_desc_fields,
        ref ty_generics,
        ref where_clause,
        ..
    } = context;

    let system_desc_fields = system_desc_fields
        .as_ref()
        .expect("Expected `system_desc_fields` to exist.");
    let struct_declaration = match system_desc_fields {
        Fields::Unit => quote!(struct #system_desc_name;),
        Fields::Unnamed(..) => quote! {
            struct #system_desc_name #ty_generics #system_desc_fields #where_clause;
        },
        Fields::Named(..) => quote! {
            struct #system_desc_name #ty_generics #where_clause #system_desc_fields
        },
    };

    let doc_string = format!("Builds a `{}`.", system_name);
    quote! {
        #[doc = #doc_string]
        #[derive(Debug)]
        pub #struct_declaration
    }
}

fn system_desc_fields(ast: &DeriveInput) -> Fields {
    // This includes any `PhantomData` fields to avoid unused type parameters.
    let fields_to_copy = ast
        .fields()
        .iter()
        .filter(|field| !field.contains_tag("system_desc", "skip"))
        .collect::<Vec<&Field>>();

    if fields_to_copy.is_empty() {
        Fields::Unit
    } else if ast.is_named() {
        let fields_named: FieldsNamed = parse_quote!({ #(#fields_to_copy,)* });
        Fields::from(fields_named)
    } else {
        // Tuple struct
        let fields_unnamed: FieldsUnnamed = parse_quote!((#(#fields_to_copy,)*));
        Fields::from(fields_unnamed)
    }
}

fn impl_constructor(context: &Context<'_>) -> TokenStream {
    let Context {
        ref system_desc_name,
        ref impl_generics,
        ref ty_generics,
        ref where_clause,
        ref is_default,
        ..
    } = context;

    let constructor_parameters = impl_constructor_parameters(context);
    let constructor_body = impl_constructor_body(context);

    if *is_default {
        quote! {
            impl #impl_generics std::default::Default for #system_desc_name #ty_generics
            #where_clause
            {
                fn default() -> Self {
                    #constructor_body
                }
            }
        }
    } else {
        let doc_constructor = format!("Returns a new {}", system_desc_name);
        quote! {
            impl #impl_generics #system_desc_name #ty_generics
            #where_clause
            {
                #[doc = #doc_constructor]
                pub fn new(#constructor_parameters) -> Self {
                    #constructor_body
                }
            }
        }
    }
}

fn impl_constructor_body(context: &Context<'_>) -> TokenStream {
    let Context {
        ref system_desc_name,
        ref system_desc_fields,
        ..
    } = context;

    let system_desc_fields = system_desc_fields
        .as_ref()
        .expect("Expected `system_desc_fields` to exist.");
    match system_desc_fields {
        Fields::Unit => quote!(#system_desc_name),
        Fields::Unnamed(fields_unnamed) => {
            let field_initializers = fields_unnamed
                .unnamed
                .iter()
                .map(|field| {
                    if field.is_phantom_data() {
                        quote!(std::marker::PhantomData::default())
                    } else {
                        let type_name_snake = field.type_name().to_string().to_snake_case();
                        let type_name_snake = Ident::new(&type_name_snake, Span::call_site());
                        quote!(#type_name_snake)
                    }
                })
                .collect::<Vec<TokenStream>>();

            quote! {
                #system_desc_name(#(#field_initializers,)*)
            }
        }
        Fields::Named(fields_named) => {
            let field_initializers = fields_named
                .named
                .iter()
                .map(|field| {
                    let field_name = field
                        .ident
                        .as_ref()
                        .expect("Expected named field to have an ident.");

                    if field.is_phantom_data() {
                        quote!(#field_name: std::marker::PhantomData::default())
                    } else {
                        quote!(#field_name)
                    }
                })
                .collect::<Vec<TokenStream>>();

            quote! {
                #system_desc_name {
                    #(#field_initializers,)*
                }
            }
        }
    }
}

fn impl_constructor_parameters(context: &Context<'_>) -> TokenStream {
    let Context {
        ref system_desc_fields,
        ..
    } = context;

    let system_desc_fields = system_desc_fields
        .as_ref()
        .expect("Expected `system_desc_fields` to exist.");
    match system_desc_fields {
        Fields::Unit => quote!(),
        Fields::Unnamed(fields_unnamed) => {
            let constructor_parameters = fields_unnamed
                .unnamed
                .iter()
                .filter(|field| !field.is_phantom_data())
                .map(|field| {
                    let type_name_snake = field.type_name().to_string().to_snake_case();
                    let type_name_snake = Ident::new(&type_name_snake, Span::call_site());
                    let field_type = &field.ty;
                    quote!(#type_name_snake: #field_type)
                })
                .collect::<Vec<TokenStream>>();

            quote! {
                #(#constructor_parameters,)*
            }
        }
        Fields::Named(fields_named) => {
            let constructor_parameters = fields_named
                .named
                .iter()
                .filter(|field| !field.is_phantom_data())
                .map(|field| {
                    let field_name = field
                        .ident
                        .as_ref()
                        .expect("Expected named field to have an ident.");
                    let field_type = &field.ty;
                    quote!(#field_name: #field_type)
                })
                .collect::<Vec<TokenStream>>();

            quote! {
                #(#constructor_parameters,)*
            }
        }
    }
}

fn call_system_constructor(context: &Context<'_>) -> TokenStream {
    let Context {
        ref system_name,
        ref system_desc_fields,
        ..
    } = context;

    let system_desc_fields = system_desc_fields
        .as_ref()
        .expect("Expected `system_desc_fields` to exist.");
    match system_desc_fields {
        Fields::Unit => quote!(#system_name::default()),
        Fields::Unnamed(fields_unnamed) => {
            let field_initializers = fields_unnamed
                .unnamed
                .iter()
                .enumerate()
                // Only pass through non-`PhantomData` fields.
                .filter(|(_, field)| !field.is_phantom_data())
                .map(|(index, _)| {
                    let index = Literal::usize_unsuffixed(index);
                    quote!(self.#index)
                })
                .collect::<Vec<TokenStream>>();

            quote! {
                #system_name::new(#(#field_initializers,)*)
            }
        }
        Fields::Named(fields_named) => {
            let field_initializers = fields_named
                .named
                .iter()
                // Only pass through non-`PhantomData` fields.
                .filter(|field| !field.is_phantom_data())
                .map(|field| {
                    let field_name = field
                        .ident
                        .as_ref()
                        .expect("Expected named field to have an ident.");

                    quote!(self.#field_name)
                })
                .collect::<Vec<TokenStream>>();

            quote! {
                #system_name::new(#(#field_initializers,)*)
            }
        }
    }
}

/// Extracts the name from the `#[system_desc(name(..))]` attribute.
#[allow(clippy::let_and_return)] // Needed due to bug in clippy.
fn system_desc_name(ast: &DeriveInput) -> Option<Ident> {
    let meta_lists = ast
        .attrs
        .iter()
        .map(Attribute::parse_meta)
        .filter_map(Result::ok)
        .filter(|meta| meta.name() == "system_desc")
        .filter_map(|meta| {
            if let Meta::List(meta_list) = meta {
                Some(meta_list)
            } else {
                None
            }
        })
        .collect::<Vec<MetaList>>();

    // Each `meta_list` is the `system_desc(..)` item.
    let name = meta_lists
        .iter()
        .flat_map(|meta_list| {
            meta_list
                .nested
                .iter()
                .filter_map(|nested_meta| {
                    if let NestedMeta::Meta(meta) = nested_meta {
                        Some(meta)
                    } else {
                        None
                    }
                })
                .filter(|meta| meta.name() == "name")
        })
        // `meta` is the `name(..)` item.
        .filter_map(|meta| {
            if let Meta::List(meta_list) = meta {
                Some(meta_list)
            } else {
                None
            }
        })
        // We want to insert a resource for each item in the list.
        .map(|meta_list| {
            if meta_list.nested.len() != 1 {
                panic!(
                    "Expected exactly one identifier for `#[system_desc(name(..))]`. `{:?}`.",
                    &meta_list.nested
                );
            }

            meta_list
                .nested
                .first()
                .map(|pair| {
                    let nested_meta = pair.value();
                    if let NestedMeta::Meta(Meta::Word(ident)) = nested_meta {
                        ident.clone()
                    } else {
                        panic!(
                            "`{:?}` is an invalid value in this position.\n\
                             Expected a single identifier.",
                            nested_meta,
                        );
                    }
                })
                .expect("Expected one meta item to exist.")
        })
        .next();

    name
}

/// Inserts resources specified inside the `#[system_desc(insert(..))]` attribute.
fn resource_insertion_expressions(ast: &DeriveInput) -> TokenStream {
    let meta_lists = ast
        .attrs
        .iter()
        .map(Attribute::parse_meta)
        .filter_map(Result::ok)
        .filter(|meta| meta.name() == "system_desc")
        .filter_map(|meta| {
            if let Meta::List(meta_list) = meta {
                Some(meta_list)
            } else {
                None
            }
        })
        .collect::<Vec<MetaList>>();

    // Each `meta_list` is the `system_desc(..)` item.
    meta_lists
        .iter()
        .flat_map(|meta_list| {
            meta_list
                .nested
                .iter()
                .filter_map(|nested_meta| {
                    if let NestedMeta::Meta(meta) = nested_meta {
                        Some(meta)
                    } else {
                        None
                    }
                })
                .filter(|meta| meta.name() == "insert")
        })
        // `meta` is the `insert(..)` item.
        .filter_map(|meta| {
            if let Meta::List(meta_list) = meta {
                Some(meta_list)
            } else {
                None
            }
        })
        // We want to insert a resource for each item in the list.
        .flat_map(|meta_list| {
            meta_list
                .nested
                .iter()
                .map(|nested_meta| match nested_meta {
                    NestedMeta::Meta(meta) => {
                        if let Meta::Word(ident) = meta {
                            quote!(#ident)
                        } else {
                            panic!(
                                "`{:?}` is an invalid value in this position.\n\
                                 Expected a literal string or single word.",
                                meta
                            )
                        }
                    }
                    NestedMeta::Literal(lit) => {
                        if let Lit::Str(lit_str) = lit {
                            // Turn the literal into tokens.
                            // The literal must be a valid expression
                            let expr = lit_str.parse::<Expr>().unwrap_or_else(|e| {
                                panic!(
                                    "Failed to parse `{:?}` as an expression. Error: {}",
                                    lit_str, e,
                                )
                            });
                            quote!(#expr)
                        } else {
                            panic!(
                                "`{:?}` is an invalid value in this position.\n\
                                 Expected a literal string or single word.",
                                lit
                            )
                        }
                    }
                })
        })
        .fold(TokenStream::new(), |mut accumulated_tokens, expr_tokens| {
            accumulated_tokens.extend(quote! {
                world.insert(#expr_tokens);
            });
            accumulated_tokens
        })
}

#[derive(Debug)]
struct Context<'c> {
    system_name: &'c Ident,
    system_desc_name: Ident,
    system_desc_fields: Option<Fields>,
    impl_generics: ImplGenerics<'c>,
    ty_generics: TypeGenerics<'c>,
    where_clause: Option<&'c WhereClause>,
    is_default: bool,
    is_self: bool,
}
