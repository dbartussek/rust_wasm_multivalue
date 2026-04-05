use proc_macro2::{Punct, Spacing, TokenStream};
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields};

pub fn magic_arg_impl(input: DeriveInput) -> TokenStream {
    let hash = Punct::new('#', Spacing::Alone);
    let magic_arg = quote! { wasm_calling_support::MagicArg };

    let ident = &input.ident;

    let mut read_fields = quote! {};
    let mut construct_read = quote! {};

    let mut write_fields = quote! {};

    let mut number_of_args = quote! { 0 };

    match &input.data {
        Data::Struct(data) => {
            let mut construct_read_fields = quote! {};

            for (index, f) in data.fields.iter().enumerate() {
                let temp_ident = format_ident!("arg{index}");
                let ty = &f.ty;

                read_fields = quote! {
                    #read_fields
                    let #temp_ident: #ty = #magic_arg::read();
                };

                number_of_args = quote! { #number_of_args + <#ty as #magic_arg>::NUMBER_OF_ARGS };

                if let Some(ident) = &f.ident {
                    construct_read_fields = quote! {
                        #construct_read_fields
                        #ident: #temp_ident,
                    };

                    write_fields = quote! {
                        #write_fields
                        #magic_arg::write(value.#ident);
                    };
                } else {
                    let index = syn::Index::from(index);

                    construct_read_fields = quote! {
                        #construct_read_fields
                        #temp_ident,
                    };

                    write_fields = quote! {
                        #write_fields
                        #magic_arg::write(value.#index);
                    };
                }
            }

            construct_read = match &data.fields {
                Fields::Named(_) => {
                    quote! { #ident { #construct_read_fields } }
                },
                Fields::Unnamed(_) => {
                    quote! { #ident ( #construct_read_fields ) }
                },
                Fields::Unit => {
                    quote! { #ident }
                },
            };
        },
        _ => panic!("A MagicArg must be a struct, not a union or enum"),
    }

    quote! {
        unsafe impl #magic_arg for #ident {
            const NUMBER_OF_ARGS: usize = #number_of_args;

            #hash [inline(always)]
            unsafe fn read() -> Self {
                unsafe {
                    #read_fields
                    #construct_read
                }
            }

            #hash [inline(always)]
            unsafe fn write(value: Self) {
                #write_fields
            }
        }
    }
}
