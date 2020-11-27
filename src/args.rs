use proc_macro2::TokenStream;
use proc_macro_error::{emit_call_site_error, emit_error};
use quote::{format_ident, quote_spanned};
use syn::{spanned::Spanned, Ident, Meta, MetaList, NestedMeta};

/// The actual parsed output of a test_impl macro invocation.
pub struct TraitAndImpls<'a> {
    pub trait_ident: &'a Ident,
    pub impl_idents: Vec<&'a Ident>,
}

impl<'a> TraitAndImpls<'a> {
    /// Output some code that will ensure the trait the user specified actually exists
    /// in the scope provided, and that it's not a typo, etc etc
    pub fn get_trait_check(&self, fn_ident: &Ident) -> TokenStream {
        let trait_ident = self.trait_ident;

        let ident = format_ident!("_TraitCheck{}{}", fn_ident, trait_ident);

        quote_spanned!(trait_ident.span() =>
            #[allow(non_camel_case_types)]
            type #ident = dyn #trait_ident;
        )
    }

    /// Output some code to ensure that the struct names we've been given actually do
    /// implement the trait we're testing. If they didn't we'd see errors all over the place,
    /// but adding this check in means we'll just get one error in the attribute field. Much nicer.
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
            // Not sure in what sitatuon this would never have an ident, but according to
            // the API it's possible, so let's cover our bases:
            emit_error!(
                arg.span(),
                "Could not parse this argument, it has no identifier"
            );
            return None;
        }
    };

    // Apply the same logic to the struct names provided. Maybe the user threw a literal in
    // there? If so, emit an error yet again.
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
    // The user might have invoked this macro without specifing a trait (i.e. #[test_impl]), which won't
    // work. So we'll mark an error for that and immediately return:
    let arg = match args.first() {
        Some(march) => march,
        _ => {
            emit_call_site_error!("You must specify a trait in the format of #[test_impl(ExampleTrait(ExampleStruct, AnotherExampleStruct))]");
            return None;
        }
    };

    // Similarly, it's a valid macro invocation to specify more than one argument. But we don't support
    // that (maybe one day? But also, maybe not?) so we'll emit and error and return:
    if args.len() > 1 {
        emit_call_site_error!("You can only specify one trait");
        return None;
    }

    // Not only that (argh), you can specify arguments in a different way than we want (e.g. this = "that"). So again,
    // if it's not formatted correctly, emit and error and return.
    match arg {
        NestedMeta::Meta(Meta::List(meta_list)) => {
            // But if it IS correct, we can finally actually parse the information we've been given:
            parse_impl_list(meta_list)
        }
        _ => {
            emit_error!(
                arg.span(),
                "This argument is not formatted correctly and can't be parsed"
            );
            None
        }
    }
}
