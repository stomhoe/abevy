//! Derive ActionBuilder on a given struct
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Lit, LitStr, Meta};

/// Derive ActionBuilder on a struct that implements Component + Clone


fn get_label(input: &DeriveInput) -> Option<LitStr> {
    let mut label: Option<LitStr> = None;
    let attrs = &input.attrs;
    for option in attrs {
        let option = option.parse_meta().unwrap();
        if let Meta::NameValue(meta_name_value) = option {
            let path = meta_name_value.path;
            let lit = meta_name_value.lit;
            if let Some(ident) = path.get_ident() {
                if ident == "action_label" {
                    if let Lit::Str(lit_str) = lit {
                        label = Some(lit_str);
                    } else {
                        panic!("Must specify a string for the `action_label` attribute")
                    }
                }
            }
        }
    }
    label
}

fn build_method(component_name: &Ident, ty_generics: &syn::TypeGenerics) -> TokenStream {
    let turbofish = ty_generics.as_turbofish();

    quote! {
        fn build(&self, cmd: &mut ::bevy::prelude::Commands, action: ::bevy::prelude::Entity, _actor: ::bevy::prelude::Entity) {
            cmd.entity(action).insert(#component_name #turbofish::clone(self));
        }
    }
}

fn label_method(label: LitStr) -> TokenStream {
    quote! {
        fn label(&self) -> ::std::option::Option<&str> {
            ::std::option::Option::Some(#label)
        }
    }
}
