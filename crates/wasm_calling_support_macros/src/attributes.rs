use proc_macro2::{Ident, Punct, Spacing, TokenStream};
use quote::{format_ident, quote};
use std::ops::Deref;
use syn::{FnArg, ItemFn, Pat, ReturnType, Type, Visibility};

pub fn wrap_wasm_impl(input: ItemFn) -> TokenStream {
    let hash = Punct::new('#', Spacing::Alone);

    let function_ident = &input.sig.ident;

    let mut read_args = quote! {};
    let mut arguments = quote! {};

    fn create_static(ident: &Ident, ty: &Type) -> TokenStream {
        let hash = Punct::new('#', Spacing::Alone);
        let size_ident = format_ident!("{ident}_SIZE");

        quote! {
            #hash [allow(non_upper_case_globals)]
            #hash [unsafe(no_mangle)]
            static mut #ident: std::mem::MaybeUninit< #ty >
                = std::mem::MaybeUninit::uninit();

            #hash [unsafe(no_mangle)]
            pub extern "C" fn #size_ident() -> usize {
                std::mem::size_of::<#ty>()
            }
        }
    }

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
                wasm_calling_support::MagicArg::read()
            };
        };
        read_args = quote! { #read_args #read };

        arguments = quote! { #arguments #ident, };
    }

    let mut wrapped_function = input.clone();
    wrapped_function.vis = Visibility::Inherited;

    let function_ident_inner = format_ident!("{}_inner", function_ident);
    wrapped_function.sig.ident = function_ident_inner.clone();

    let function_ident_wrapper = format_ident!("WRAPPED_{function_ident}");

    let output = match &input.sig.output {
        ReturnType::Default => {
            quote! { #function_ident_inner ( #arguments ) }
        },
        ReturnType::Type(_, ty) => {
            quote! {
                unsafe {
                    let ret: #ty = #function_ident_inner ( #arguments );
                    wasm_calling_support::MagicArg::write(ret);
                }
            }
        },
    };

    quote! {
        #hash [unsafe(no_mangle)]
        pub extern "C" fn #function_ident_wrapper() {
            #hash [inline(always)]
            #wrapped_function

            #read_args

            #output
        }
    }
}
