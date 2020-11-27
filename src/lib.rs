mod args;
mod impl_block;

use args::parse_args;
use impl_block::create_impl_blocks;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn};

#[proc_macro_attribute]
#[proc_macro_error]
pub fn test_impl(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args_parsed = parse_macro_input!(args as AttributeArgs);
    let trait_and_impls = match parse_args(&args_parsed) {
        None => return input,
        Some(t) => t,
    };
    let input_parsed = syn::parse::<ItemFn>(input).unwrap();

    let fn_ident = &input_parsed.sig.ident;

    let trait_check = trait_and_impls.get_trait_check(&fn_ident);

    let impl_checks = trait_and_impls.get_impl_checks(&fn_ident);

    let attrs = &input_parsed.attrs;
    let vis = &input_parsed.vis;
    let sig = &input_parsed.sig;
    let block = &input_parsed.block;
    let impl_blocks = create_impl_blocks(&trait_and_impls, &block);

    let output = quote! {
        #trait_check
        #(#impl_checks)*

        #(#attrs)*
        #vis #sig
        {
            #(#impl_blocks)*
        }

    };

    output.into()
}
