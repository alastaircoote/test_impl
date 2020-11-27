use crate::args::TraitAndImpls;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Block;

/// Convert an original trait-specific function body into multiple impl-specific bodies:
pub fn create_impl_blocks(trait_and_impls: &TraitAndImpls, base_block: &Block) -> Vec<TokenStream> {
    let trait_ident = trait_and_impls.trait_ident;

    trait_and_impls
        .impl_idents
        .iter()
        .map(|impl_ident| {
            let stmts = &base_block.stmts;
            quote! {
                // Create a function with the exact name of the impl. This ensures that any backtraces
                // in the test output will indicate exactly which impl failed.
                #[allow(non_snake_case)]
                fn #impl_ident() {
                    // this type alias is the key part here. We're redefining the trait ident
                    // to actually refer to the implentation instead:
                    type #trait_ident = #impl_ident;
                    // Then passing through the original function body untouched:
                    #(#stmts)*
                }
                #impl_ident();
            }
        })
        .collect()
}
