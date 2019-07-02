//! WidgetID Implementation

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Ident};

pub fn impl_widget_id(ast: &DeriveInput) -> TokenStream {
    // For now, we can only derive this on enums
    let default_variant = match &ast.data {
        Data::Enum(ref data_enum) => {
            let maybe_marked_default = data_enum
                .variants
                .iter()
                .filter(|v| {
                    v.attrs
                        .iter()
                        .any(|attr| attr.path.segments[0].ident == "widget_id_default")
                })
                .map(|v| v.ident.clone())
                .collect::<Vec<Ident>>();

            if maybe_marked_default.len() > 1 {
                panic!("Only 1 variant can be marked as default widget id")
            }

            if !maybe_marked_default.is_empty() {
                maybe_marked_default[0].clone()
            } else {
                data_enum.variants[0].ident.clone()
            }
        }
        _ => panic!("WidgetId derive only supports enums"),
    };

    let name = ast.ident.clone();

    quote! {
        impl WidgetId for #name {
            fn generate(last: &Option<Self>) -> Self {
                Self::#default_variant
            }
        }
    }
}
