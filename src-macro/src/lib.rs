extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Select)]
pub fn derive_select(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = if let syn::Data::Struct(data_struct) = &input.data {
        data_struct
            .fields
            .iter()
            .map(|field| &field.ident)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let field_names = fields
        .iter()
        .map(|f| f.as_ref().unwrap().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let expanded = quote! {
        impl #name {
            pub fn select() -> &'static str {
                #field_names
            }
        }
    };

    TokenStream::from(expanded)
}
