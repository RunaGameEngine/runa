use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn system(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let sig = &input.sig;
    let block = &input.block;
    let name = &sig.ident;

    TokenStream::from(quote! {
        #sig #block

        ::runa_engine::runa_ecs::inventory::submit! {
            ::runa_engine::runa_ecs::SystemDescriptor {
                name: stringify!(#name),
                func: #name,
            }
        }
    })
}
