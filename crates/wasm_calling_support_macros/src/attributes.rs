use proc_macro2::{Punct, Spacing, TokenStream};
use quote::{format_ident, quote};
use std::ops::Deref;
use syn::{FnArg, ItemFn, Pat, ReturnType, Visibility};

pub fn wrap_wasm_impl(input: ItemFn) -> TokenStream {
    let hash = Punct::new('#', Spacing::Alone);
    let magic_arg = quote! { wasm_calling_support::MagicArg };

    let function_ident = &input.sig.ident;

    let mut read_args = quote! {};
    let mut arguments = quote! {};
    let mut number_of_args = quote! { 0 };

    for input in input.sig.inputs.iter() {
        let (ident, ty) = match input {
            FnArg::Receiver(_) => {
                panic!("Can't have self")
            },
            FnArg::Typed(typed) => (&typed.pat, &typed.ty),
        };
        let ident = match ident.deref() {
            Pat::Ident(ident) => &ident.ident,
            other => panic!("Bad identifier {other:?}"),
        };

        let read = quote! {
            let #ident: #ty = unsafe {
                #magic_arg::read()
            };
        };
        read_args = quote! { #read_args #read };

        arguments = quote! { #arguments #ident, };

        number_of_args = quote! { #number_of_args + <#ty as #magic_arg>::NUMBER_OF_ARGS };
    }

    let mut wrapped_function = input.clone();
    wrapped_function.vis = Visibility::Inherited;

    let function_ident_inner = format_ident!("{}_inner", function_ident);
    wrapped_function.sig.ident = function_ident_inner.clone();

    let function_ident_wrapper = format_ident!("WRAPPED__{function_ident}");
    let function_ident_nr_arguments = format_ident!("WRAPMETA_SIG_ARG_COUNT__{function_ident}");

    let function_ident_nr_returns = format_ident!("WRAPMETA_SIG_RETURN_COUNT__{function_ident}");
    let function_ident_signature = format_ident!("WRAPMETA_SIG__{function_ident}");

    let output = match &input.sig.output {
        ReturnType::Default => {
            quote! {
                #hash [unsafe(no_mangle)]
                pub extern "C" fn #function_ident_nr_returns() -> usize {
                    0
                }

                #hash [unsafe(no_mangle)]
                #hash [allow(unused)]
                pub extern "C" fn #function_ident_signature() {
                    #read_args
                }

                #function_ident_inner ( #arguments );
            }
        },
        ReturnType::Type(_, ty) => {
            quote! {
                #hash [unsafe(no_mangle)]
                pub extern "C" fn #function_ident_nr_returns() -> usize {
                    <#ty as #magic_arg>::NUMBER_OF_ARGS
                }

                #hash [unsafe(no_mangle)]
                #hash [allow(unused)]
                pub extern "C" fn #function_ident_signature() {
                    #read_args
                    unsafe { <#ty as #magic_arg>::read(); }
                }

                unsafe {
                    let ret: #ty = #function_ident_inner ( #arguments );
                    #magic_arg::write(ret);
                }
            }
        },
    };

    quote! {
        #hash [unsafe(no_mangle)]
        pub extern "C" fn #function_ident_wrapper() {
            #hash [inline(always)]
            #wrapped_function

            #hash [unsafe(no_mangle)]
            pub extern "C" fn #function_ident_nr_arguments() -> usize {
                const { #number_of_args }
            }

            #read_args

            #output
        }
    }
}
