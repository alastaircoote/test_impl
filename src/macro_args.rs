use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse::Parse, punctuated::Punctuated, GenericParam, Generics, Ident, Token, TypeParam};

pub struct IdentAndGenerics {
    pub ident: Ident,
    pub generics: Generics,
}

impl Parse for IdentAndGenerics {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let generics = input.parse::<Generics>()?;
        Ok(IdentAndGenerics { ident, generics })
    }
}

/// The actual parsed output of a test_impl macro invocation.
pub struct TraitAndImpls {
    pub base_trait: IdentAndGenerics,
    pub structs: Vec<IdentAndGenerics>,
}

impl TraitAndImpls {
    /// Generate a type signature for our trait. i.e. if we have:
    /// ```
    /// ExampleTrait<'a, 'b, T>
    /// ```
    /// this will generate:
    /// ```
    /// TraitChecker: ExampleTrait<'a, 'b, T>
    /// ```
    fn generate_type_param_for_trait(
        base_trait: &IdentAndGenerics,
    ) -> Result<TypeParam, syn::Error> {
        let trait_ident = &base_trait.ident;
        let generics = &base_trait.generics;
        let extra_generic = quote! {
            TraitChecker: #trait_ident#generics
        };
        syn::parse2::<TypeParam>(extra_generic)
    }

    /// Alright, what on earth are we doing here? Well, we want to make sure that the
    /// trait specified in the macro exists. We also want to make sure that all the stucts
    /// provided actually implement that trait. So we generate something that looks like the following:
    ///
    /// ```
    /// fn _test_func_name_ImplCheck() {
    ///     fn check<'a, 'b, T: ExampleTrait<'a, 'b>>(item: T) {}
    ///     check::<ExampleStruct>;
    ///     check::<ExampleStruct2>;
    /// }
    /// ```
    pub fn output_impl_checks(&self, fn_ident: &Ident) -> TokenStream {
        let impl_check_fn_ident = format_ident!("_{}_ImplCheck", fn_ident);

        // Create our `ExampleTrait<'a, 'b>` extra generic param:
        let extra_generic = Self::generate_type_param_for_trait(&self.base_trait).unwrap();

        // Now add it to the end of a copy of the trait generics:
        let mut fn_generics = self.base_trait.generics.clone();
        fn_generics.params.push(GenericParam::Type(extra_generic));

        let checks: Vec<TokenStream> = self
            .structs
            .iter()
            .map(|struc| {
                let struct_ident = &struc.ident;
                // Call the check() function for this specific implementation:
                quote_spanned!(struct_ident.span()=>
                    check::<#struct_ident>;
                )
            })
            .collect();

        (quote! {
            #[allow(path_statements)]
            #[allow(non_snake_case)]
            fn #impl_check_fn_ident() {
                fn check#fn_generics(item: TraitChecker) {}
                #(#checks)*
            }
        })
        .into()
    }
}

impl Parse for TraitAndImpls {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let trait_ident = input.parse::<Ident>()?;
        let trait_generics = input.parse::<Generics>()?;
        input.parse::<Token![=]>()?;

        let comma_separated_struct_idents: Punctuated<IdentAndGenerics, Token![,]> =
            input.parse_terminated(IdentAndGenerics::parse)?;

        let struct_idents: Vec<IdentAndGenerics> = comma_separated_struct_idents
            .into_pairs()
            .map(|i| i.into_value())
            .collect();

        Ok(TraitAndImpls {
            base_trait: IdentAndGenerics {
                ident: trait_ident,
                generics: trait_generics,
            },
            structs: struct_idents,
        })
    }
}

#[cfg(test)]
mod test {
    use quote::quote;

    use super::TraitAndImpls;
    #[test]
    fn parses_basic_arg() {
        let basic = quote! {
            ExampleTrait = ExampleStruct1, ExampleStruct2
        };

        let result = syn::parse2::<TraitAndImpls>(basic).unwrap();
        assert_eq!(result.base_trait.ident.to_string(), "ExampleTrait");
        assert_eq!(result.structs.len(), 2);
        assert_eq!(result.structs[0].ident.to_string(), "ExampleStruct1");
        assert_eq!(result.structs[1].ident.to_string(), "ExampleStruct2");
    }

    #[test]
    fn parses_generic_arg() {
        let generics = quote! {
            ExampleTrait<'a, B> = ExampleStruct1<'c, D>, ExampleStruct2<'e, F>
        };

        let result = syn::parse2::<TraitAndImpls>(generics).unwrap();
        let mut gm = result.base_trait.generics.lifetimes();

        assert_eq!(gm.next().unwrap().lifetime.ident.to_string(), "a")
    }
}
