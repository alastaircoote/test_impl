use proc_macro2::TokenStream;
use proc_macro_error::{emit_call_site_error, emit_error};
use quote::{format_ident, quote_spanned};
use syn::{spanned::Spanned, Ident, Meta, MetaList, NestedMeta};

pub struct TraitAndImpls<'a> {
    pub trait_ident: &'a Ident,
    pub impl_idents: Vec<&'a Ident>,
}

impl<'a> TraitAndImpls<'a> {
    pub fn get_trait_check(&self, fn_ident: &Ident) -> TokenStream {
        let trait_ident = self.trait_ident;

        let ident = format_ident!("_TraitCheck{}{}", fn_ident, trait_ident);

        quote_spanned!(trait_ident.span() =>
            #[allow(non_camel_case_types)]
            type #ident = dyn #trait_ident;
        )
    }

    pub fn get_impl_checks(&self, fn_ident: &Ident) -> Vec<TokenStream> {
        let trait_ident = self.trait_ident;
        self.impl_idents
            .iter()
            .map(|impl_ident| {
                let ident = format_ident!("_ImplCheck{}{}", fn_ident, impl_ident);
                quote_spanned!(impl_ident.span() =>
                    #[allow(non_camel_case_types)]
                    struct #ident where #impl_ident: #trait_ident;
                )
            })
            .collect()
    }
}

fn parse_impl_list(arg: &MetaList) -> Option<TraitAndImpls> {
    let trait_ident = match arg.path.get_ident() {
        Some(ident) => ident,
        _ => {
            emit_error!(
                arg.span(),
                "Could not parse this argument, it has no identifier"
            );
            return None;
        }
    };

    let impl_idents: Vec<&Ident> = arg
        .nested
        .iter()
        .filter_map(|meta| match meta {
            NestedMeta::Meta(Meta::Path(path)) => match path.get_ident() {
                Some(ident) => Some(ident),
                _ => {
                    emit_error!(meta.span(), "This must be an identifier");
                    None
                }
            },
            _ => {
                emit_error!(meta.span(), "Could not parse this impl identifier");
                None
            }
        })
        .collect();

    Some(TraitAndImpls {
        trait_ident,
        impl_idents,
    })
}

pub fn parse_args<'a>(args: &'a Vec<NestedMeta>) -> Option<TraitAndImpls<'a>> {
    let arg = match args.first() {
        Some(march) => march,
        _ => {
            emit_call_site_error!("You must specify a trait in the format of #[test_impl(ExampleTrait(ExampleStruct, AnotherExampleStruct))]");
            return None;
        }
    };

    if args.len() > 1 {
        emit_call_site_error!("You can only specify one trait");
        return None;
    }

    match arg {
        NestedMeta::Meta(Meta::List(meta_list)) => parse_impl_list(meta_list),
        _ => {
            emit_error!(
                arg.span(),
                "This argument is not formatted correctly and can't be parsed"
            );
            None
        }
    }
}
