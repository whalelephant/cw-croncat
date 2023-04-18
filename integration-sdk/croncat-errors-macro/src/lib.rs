extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput, Error, Ident, Variant};

#[proc_macro_attribute]
pub fn croncat_error(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let enum_name = input.ident.clone();

    // Ensure they've placed the attribute macro above the derive
    if !input.attrs.iter().any(|attr| attr.path.is_ident("derive")) {
        let msg = "😻 Please move the `#[croncat_error]` macro above the `#[derive]` attribute with the `Error` trait. 😻";
        let error = Error::new_spanned(enum_name, msg).to_compile_error();
        return TokenStream::from(quote! {
            #error
        });
    }

    let mut is_croncat_error_present = false;
    if let Data::Enum(DataEnum {
        ref mut variants, ..
    }) = input.data
    {
        // See if they already have CronCatError variant
        for variant in variants.iter() {
            if variant.ident == Ident::new("CronCatError", variant.ident.span()) {
                is_croncat_error_present = true;
            }
        }

        if !is_croncat_error_present {
            // Add the CronCat variant, which looks like:
            // #[error("CronCat error: {err:?}")]
            // CronCatError {
            //   err: CronCatContractError
            // }
            let croncat_error_variant: Variant = syn::parse_quote! {
                #[error("CronCat error: {err:?}")]
                CronCatError {
                    err: croncat_integration_utils::error::CronCatContractError,
                }
            };
            variants.push(croncat_error_variant);
        }
    }

    // Add an impl for error propagation. Looks like:
    // impl From<CronCatContractError> for ContractError {
    //   fn from(error: CronCatContractError) -> Self {
    //     ContractError::CronCatError {
    //       err: error,
    //     }
    //   }
    // }
    let expanded = quote! {
        #input

        impl From<croncat_integration_utils::error::CronCatContractError> for #enum_name {
            fn from(error: croncat_integration_utils::error::CronCatContractError) -> Self {
                #enum_name::CronCatError {
                    err: error,
                }
            }
        }
    };

    TokenStream::from(expanded)
}
