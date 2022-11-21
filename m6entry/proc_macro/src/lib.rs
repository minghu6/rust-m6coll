#![allow(unused_imports)]
#![allow(unused_import_braces)]

extern crate proc_macro;

use std::convert::identity;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Result};
use syn::token::{Paren, Priv, Token, Brace};
use syn::{
    parse_macro_input, Expr, Ident, Lit, LitInt, LitStr, MacroDelimiter, Path,
    Token,
};

use proc_macro_utils::{*, ident};

////////////////////////////////////////////////////////////////////////////////
//// Define New Custom Error

struct DefTOEnt {
    lit_meta_nums: LitInt,
}


impl Parse for DefTOEnt {
    fn parse(input: ParseStream) -> Result<Self> {
        let lit_meta_nums: LitInt = input.parse()?;

        Ok(Self { lit_meta_nums })
    }
}


#[proc_macro]
pub fn deftoent(input: TokenStream) -> TokenStream {
    let DefTOEnt { lit_meta_nums } = parse_macro_input!(input as DefTOEnt);

    let meta_nums: usize = lit_meta_nums.base10_parse().unwrap();

    let mut qs = quote! {};

    let struct_name = ident!("TOEntry{meta_nums}");

    /* Define Structure */

    let lbrace = lbrace();
    let rbrace = rbrace();
    let sign_less = sign_less();
    let sign_gt = sign_gt();


    qs.extend(quote! {
        pub struct #struct_name #sign_less
    });

    for i in 1..=meta_nums {
        let ti = ident!("T{i}");

        qs.extend(quote! {
            #ti,
        })
    }

    qs.extend(quote! {
         #sign_gt
    });


    TokenStream::from(qs)
}
