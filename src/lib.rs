mod macro_args;

use macro_args::TraitAndImpls;
use proc_macro2::TokenStream;
use proc_macro_error::{emit_call_site_error, proc_macro_error};
use quote::format_ident;
use quote::quote;
use syn::ItemFn;

/// Takes the original function and repeats it for each of the implementations provided. Example:
/// ```
/// #[test]
/// fn do_test() {
///     ExampleTrait::do_thing()
/// }
/// ```
/// becomes:
/// ```
/// #[test]
/// fn do_test() {
///     fn impl_ExampleStruct() {
///         type ExampleTrait = ExampleStruct;
///         ExampleTrait::do_thing();
///     }
///     impl_ExampleStruct();
///
///     fn impl_ExampleStruct2() {
///         type ExampleTrait = ExampleStruct2;
///         ExampleTrait::do_thing();
///     }
///     impl_ExampleStruct2();
/// }
///
fn repeat_function_with_mappings(func: &ItemFn, trait_and_impls: TraitAndImpls) -> TokenStream {
    let impl_blocks: Vec<TokenStream> = trait_and_impls
        .structs
        .iter()
        .map(|struc| {
            let fn_ident = format_ident!("impl_{}", struc.ident);
            let trait_ident = &trait_and_impls.base_trait.ident;
            let trait_generics = &trait_and_impls.base_trait.generics;

            let struct_ident = &struc.ident;
            let struct_generics = &struc.generics;
            let stmts = &func.block.stmts;

            quote! {
                fn #fn_ident() {
                    type #trait_ident#trait_generics = #struct_ident#struct_generics;
                    #(#stmts)*
                }

                #fn_ident();
            }
        })
        .collect();

    let attrs = &func.attrs;
    let vis = &func.vis;
    let sig = &func.sig;

    quote! {
        #(#attrs)*
        #[allow(non_snake_case)]
        #vis #sig
        {
            #(#impl_blocks)*
        }
    }
}

/// Run this test multiple times, replacing all references to the trait specified with a specific implementation.
/// Use it like this:
///
/// `#[test_impl(ExampleTrait = ExampleStruct, ExampleStruct2)]`
#[proc_macro_attribute]
#[proc_macro_error]
pub fn test_impl(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = match syn::parse::<TraitAndImpls>(args) {
        Ok(a) => a,
        Err(e) => {
            emit_call_site_error!("Could not parse macro arguments: {}", e);
            return proc_macro::TokenStream::new();
        }
    };

    let fn_def = match syn::parse::<ItemFn>(input) {
        Ok(f) => f,
        Err(e) => {
            emit_call_site_error!("You must use this macro with a function: {}", e);
            return proc_macro::TokenStream::new();
        }
    };

    let impl_checks = args.output_impl_checks(&fn_def.sig.ident);
    let mapped = repeat_function_with_mappings(&fn_def, args);
    (quote! {
        #impl_checks
        #mapped
    })
    .into()
}
