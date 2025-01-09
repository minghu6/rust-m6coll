#![feature(impl_trait_in_assoc_type)]

use syn::parse::Parse;

////////////////////////////////////////////////////////////////////////////////
//// Structures

pub struct Punctuated<T, P>(syn::punctuated::Punctuated<T, P>);

////////////////////////////////////////////////////////////////////////////////
//// Implmentations

impl<T, P> IntoIterator for Punctuated<T, P> {
    type Item = T;

    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Parse, P: Parse> Parse for Punctuated<T, P> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut punc = syn::punctuated::Punctuated::new();

        while !input.is_empty() {
            punc.push_value(input.parse()?);

            if input.is_empty() {
                break;
            }

            punc.push_punct(input.parse()?);
        }

        Ok(Self(punc))
    }
}
