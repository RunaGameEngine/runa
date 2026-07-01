use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Simplified code‑first `Component` derive.
///
/// Generates the internal `Component` trait impl with `as_any` / `as_any_mut`.
/// This is the only macro you need for plain data components.
///
/// # Example
/// ```ignore
/// use runa_engine::prelude::*;
///
/// #[derive(Component)]
/// struct Health { max: f32, current: f32 }
/// ```
#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;

    TokenStream::from(quote! {
        impl ::runa_engine::runa_core::components::Component for #ident {
            fn as_any(&self) -> &dyn ::std::any::Any { self }
            fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any { self }
        }
    })
}
