mod attributes;
mod derives;

use crate::{attributes::wrap_wasm_impl, derives::magic_arg_impl};
use proc_macro::TokenStream;
use syn::{DeriveInput, ItemFn, parse_macro_input};

#[proc_macro_derive(MagicArg)]
pub fn magic_arg(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    magic_arg_impl(input).into()
}

#[proc_macro_attribute]
pub fn wrap_wasm(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    wrap_wasm_impl(input).into()
}
