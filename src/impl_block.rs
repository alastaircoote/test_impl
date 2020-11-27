use crate::args::TraitAndImpls;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Block;

pub fn create_impl_blocks(trait_and_impls: &TraitAndImpls, base_block: &Block) -> Vec<TokenStream> {
    let trait_ident = trait_and_impls.trait_ident;

    trait_and_impls
        .impl_idents
        .iter()
        .map(|impl_ident| {
            let stmts = &base_block.stmts;
            quote! {
                #[allow(non_snake_case)]
                fn #impl_ident() {
                    type #trait_ident = #impl_ident;
                    #(#stmts)*
                }
                #impl_ident();
            }
        })
        .collect()
}
