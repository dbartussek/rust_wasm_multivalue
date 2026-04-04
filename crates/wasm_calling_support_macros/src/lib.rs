use proc_macro::TokenStream;
use proc_macro2::{Ident, Punct, Spacing};
use quote::{format_ident, quote};
use std::ops::Deref;
use syn::{FnArg, ItemFn, Pat, ReturnType, Type, Visibility, parse_macro_input, DeriveInput};

#[proc_macro_derive(MagicArg)]
pub fn t(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    
    quote! {}.into()
}

#[proc_macro_attribute]
pub fn wrap_wasm(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let hash = Punct::new('#', Spacing::Alone);

    let input = parse_macro_input!(item as ItemFn);

    let function_ident = &input.sig.ident;

    let mut static_data = quote! {};

    let mut read_args = quote! {};
    let mut arguments = quote! {};

    fn create_static(ident: &Ident, ty: &Type) -> proc_macro2::TokenStream {
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

    for (index, input) in input.sig.inputs.iter().enumerate() {
        let (ident, ty) = match input {
            FnArg::Receiver(_) => {
                panic!("Can't have self")
            }
            FnArg::Typed(typed) => (&typed.pat, &typed.ty),
        };
        let ident = match ident.deref() {
            Pat::Ident(ident) => &ident.ident,
            other => panic!("Bad identifier {other:?}"),
        };

        let static_ident = format_ident!("SUPPORT_{function_ident}_INPUT_{index}_{ident}");
        let static_instance = create_static(&static_ident, ty);
        static_data = quote! {
            #static_data
            #static_instance
        };

        let read = quote! {
            let #ident = unsafe {
                std::ptr::read_volatile(&raw mut #static_ident).assume_init()
            };
        };
        read_args = quote! { #read_args #read };

        arguments = quote! { #arguments #ident, };
    }

    let output = match &input.sig.output {
        ReturnType::Default => {
            quote! {let _ = ret;}
        }
        ReturnType::Type(_, ty) => {
            let static_ident = format_ident!("SUPPORT_{function_ident}_OUTPUT");

            let static_instance = create_static(&static_ident, ty);
            static_data = quote! {
                #static_data
                #static_instance
            };

            quote! {
                unsafe {
                    std::ptr::write_volatile(
                        &raw mut #static_ident,
                        std::mem::MaybeUninit::new(ret)
                    );
                }
            }
        }
    };

    let mut wrapped_function = input.clone();
    wrapped_function.vis = Visibility::Inherited;

    let function_ident_inner = format_ident!("{}_inner", function_ident);
    wrapped_function.sig.ident = function_ident_inner.clone();

    let function_ident_wrapper = format_ident!("WRAPPED_{function_ident}");

    quote! {
        #static_data

        #hash [unsafe(no_mangle)]
        pub extern "C" fn #function_ident_wrapper() {
            #hash [inline(always)]
            #wrapped_function

            #read_args

            let ret = #function_ident_inner ( #arguments );

            #output
        }
    }
    .into()
}
