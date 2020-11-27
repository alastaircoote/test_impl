mod args;
mod impl_block;

use args::parse_args;
use impl_block::create_impl_blocks;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn};

/// Run this test multiple times, replacing all references to the trait specified with a specific implementation.
/// Use it like this:
///
/// `#[test_impl(ExampleTrait(ExampleStruct, ExampleStruct2))]`
#[proc_macro_attribute]
#[proc_macro_error]
pub fn test_impl(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse the arguments parsed to our macro and extract the trait and impl info:
    let args_parsed = parse_macro_input!(args as AttributeArgs);
    let trait_and_impls = match parse_args(&args_parsed) {
        // If we get None back it's because parse_args has failed and used emit_error.
        // In this instance we just throw the input stream straight back out again
        // as there's no further work to do here.
        None => return input,
        Some(t) => t,
    };

    // Now that we've successfully parsed the arguments, parse the actual function we've
    // been passed.
    let input_parsed = syn::parse::<ItemFn>(input).unwrap();

    // Grab the ident of the function. We use this in the next two calls to ensure that
    // the type checks we write have unique identifiers, by prepending the fn name to them.
    let fn_ident = &input_parsed.sig.ident;

    // Emit some code to check that the trait the user entered actually exists in this scope
    let trait_check = trait_and_impls.get_trait_check(&fn_ident);

    // Then emit some code to check that the structs we've been given really are implementations
    // of the trait in question
    let impl_checks = trait_and_impls.get_impl_checks(&fn_ident);

    // Grab some info about the function because I'm not sure how to access it inside quote!
    // otherwise (e.g. #input_parsed.sig doesn't work)
    let attrs = &input_parsed.attrs;
    let vis = &input_parsed.vis;
    let sig = &input_parsed.sig;

    // Take the original function body and copy it multiple times, making the necessary changes
    // for it to test the implementation rather than the trait itself.
    let impl_blocks = create_impl_blocks(&trait_and_impls, &input_parsed.block);

    // Finally, reconstruct the whole thing, along with the type check code:
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
