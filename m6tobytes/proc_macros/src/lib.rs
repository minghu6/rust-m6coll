use proc_macro::TokenStream;
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Ident, Meta};

#[proc_macro_error]
#[proc_macro_derive(ToBits)]
pub fn derive_to_bits(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut maybe_to_bits_type: Option<Ident> = None;

    for attr in input.attrs {
        if let Meta::List(metalist) = attr.meta {
            if let Some(ident) = metalist.path.get_ident() {
                if ident.to_string() == "repr" {
                    let Ok(repr_type) = metalist.parse_args::<Ident>() else {
                        abort_call_site!("Repr is malformed, expecting repr(u*/i*)");
                    };

                    let repr_type_str = repr_type.to_string();

                    let repr_types = [
                        "u8", "u16", "u32", "u64", "u128",
                        "i8", "i16", "i32", "i64", "i128",
                    ];

                    if repr_types.iter().find(|&&x| x == repr_type_str).is_some() {
                        maybe_to_bits_type = Some(repr_type.to_owned());
                    }
                }
            }
        }
    }

    let Some(to_bits_type) = maybe_to_bits_type else {
        abort_call_site!("No repr(u*/i*) attribute")
    };

    let struct_name = input.ident;

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl #struct_name {
            pub fn to_bits(self) -> #to_bits_type {
                unsafe { std::mem::transmute(self) }
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
