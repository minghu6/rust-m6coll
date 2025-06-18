#![feature(extend_one)]

use derive_syn_parse::Parse;
use m6syn::Punctuated;
use proc_macro::TokenStream;
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use quote::{ToTokens, quote};
use syn::{
    Attribute,
    Data::*,
    DeriveInput,
    Fields::*,
    FieldsNamed, FieldsUnnamed, Ident, Item, Meta, Token,
    Type::{self, *},
    Path,
    parse_macro_input,
    spanned::Spanned,
};

macro_rules! ident {
    ($s:expr) => {
        Ident::new($s, proc_macro2::Span::call_site())
    };
}

const REPR_TYPES: [&str; 10] = [
    "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128",
];

const MALFORMED_MSG: &str = "Repr is malformed, expecting repr(u*/i*)";

#[proc_macro_error]
#[proc_macro_derive(FromBytes)]
pub fn derive_from_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let reprs = find_reprs(&input.attrs);

    let expanded = match &input.data {
        Struct(data_struct) => match &data_struct.fields {
            Named(fields_named) => {
                from_bytes_data_struct_named(&input.ident, fields_named)
            }
            Unnamed(fields_unnamed) => from_bytes_data_struct_unnamed(
                reprs,
                &input.ident,
                fields_unnamed,
            ),
            Unit => abort_call_site!("ZST structure"),
        },
        Enum(_data_enum) => from_bytes_data_enum(reprs, &input.ident),
        Union(_data_union) => unimplemented!(),
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

#[proc_macro_error]
#[proc_macro_derive(ToBytes)]
pub fn derive_to_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let reprs = find_reprs(&input.attrs);

    let expanded = match &input.data {
        Struct(data_struct) => match &data_struct.fields {
            Named(fields_named) => {
                to_bytes_data_struct_named(&input.ident, fields_named)
            }
            Unnamed(fields_unnamed) => to_bytes_data_struct_unnamed(
                reprs,
                &input.ident,
                fields_unnamed,
            ),
            Unit => abort_call_site!("ZST structure"),
        },
        Enum(_data_enum) => to_bytes_data_enum(reprs, &input.ident),
        Union(_data_union) => unimplemented!(),
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

/// impl `pub fn to_bits(self) -> i*/u*`
#[proc_macro_error]
#[proc_macro_attribute]
pub fn derive_to_bits(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as Item);
    let ty = parse_macro_input!(attr as Ident);

    let Some(name) = parse_item_name(&item)
    else {
        unimplemented!()
    };

    let mut expanded = match &item {
        Item::Enum(item_enum) => {
            let mut unnamed_fields = vec![];

            for variant in item_enum.variants.iter() {
                match &variant.fields {
                    Named(_) => unimplemented!(),
                    Unnamed(fields_unnamed) => {
                        if fields_unnamed.unnamed.len() != 1 {
                            abort!(fields_unnamed.unnamed.span(), "Expect just one field")
                        }

                        unnamed_fields.push(&variant.ident);
                    },
                    Unit => (),
                }
            }

            let unnamed_fields_impl = unnamed_fields.into_iter().map(|field_name| quote! {
                Self::#field_name(v) => unsafe { std::mem::transmute(v) },
            }).collect::<proc_macro2::TokenStream>();

            if unnamed_fields_impl.is_empty() {
                quote! {
                    impl #name {
                        pub fn to_bits(self) -> #ty {
                            match self {
                                #unnamed_fields_impl
                                _ => unsafe { std::mem::transmute(self) }
                            }
                        }
                    }
                }
            }
            else {
                let dst_ty = ident!(match ty.to_string().as_str() {
                    "u8" => "u16",
                    "u16" => "u32",
                    "u32" => "u64",
                    "u64" => "u128",
                    _ => unimplemented!()
                });

                quote! {
                    impl #name {
                        pub fn to_bits(self) -> #ty {
                            match self {
                                #unnamed_fields_impl
                                _ => unsafe { std::mem::transmute::<_, #dst_ty>(self) as #ty }
                            }
                        }
                    }
                }
            }

        },
        Item::Struct(_) | Item::Union(_) => quote! {
            impl #name {
                pub fn to_bits(self) -> #ty {
                    unsafe { std::mem::transmute(self) }
                }
            }
        },
        _ => unimplemented!(),
    };

    expanded.extend(quote! {
        // impl Into<#ty> for #name {
        //     fn into(self) -> #ty {
        //         self.to_bits()
        //     }
        // }

        #item
    });

    TokenStream::from(expanded)
}

/// impl `pub unsafe fn from_bits(value: #ty) -> Self`
#[proc_macro_error]
#[proc_macro_attribute]
pub fn derive_from_bits(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as Item);
    let ty = parse_macro_input!(attr as Ident);

    let Some(name) = parse_item_name(&item)
    else {
        unimplemented!()
    };

    // let fname = ident!(format!("from_{}", ty.to_string()).as_str());

    TokenStream::from(quote! {
        impl #name {
            pub unsafe fn from_bits(value: #ty) -> Self {
                unsafe { std::mem::transmute(value) }
            }
        }

        #item
    })
}

/// impl safe `fn from_bits(value: #ty) -> Self`
#[proc_macro_error]
#[proc_macro_attribute]
pub fn safe_derive_from_bits(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as Item);
    let ty = parse_macro_input!(attr as Ident);

    let Some(name) = parse_item_name(&item)
    else {
        unimplemented!()
    };

    // let fname = ident!(format!("from_{}", ty.to_string()).as_str());

    TokenStream::from(quote! {
        impl #name {
            pub fn from_bits(value: #ty) -> Self {
                unsafe { std::mem::transmute(value) }
            }
        }

        #item
    })
}

///
/// ```no_main
/// impl Into<#ty> for #name {
///     fn into(self) -> #ty {
///         unsafe { std::mem::transmute(self.to_bits()) }
///     }
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn derive_to_bits_into(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as Item);
    let ty = parse_macro_input!(attr as Path);

    let Some(name) = parse_item_name(&item)
    else {
        unimplemented!()
    };

    let mut expanded = quote! {
        impl Into<#ty> for #name {
            fn into(self) -> #ty {
                unsafe { std::mem::transmute(self.to_bits()) }
            }
        }
    };

    expanded.extend_one(item.to_token_stream());

    TokenStream::from(expanded)
}

/// impl `ToBytes` based on `to_bits` method
#[proc_macro_error]
#[proc_macro_attribute]
pub fn derive_to_bits_to_bytes(
    _attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let item = parse_macro_input!(item as Item);

    let Some(name) = parse_item_name(&item)
    else {
        unimplemented!()
    };

    let mut expanded = quote! {
        impl ToBytes for #name {
            fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
                self.to_bits().to_le_bytes()
            }

            fn to_be_bytes(self) -> [u8; size_of::<Self>()] {
                self.to_bits().to_be_bytes()
            }
        }
    };

    expanded.extend_one(item.to_token_stream());

    TokenStream::from(expanded)
}

/// impl `FromBytes` based on `from_bits` & `i*/u*::from_xx_bytes`
#[proc_macro_error]
#[proc_macro_attribute]
pub fn derive_from_bits_from_bytes(
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let item = parse_macro_input!(item as Item);
    let ty = parse_macro_input!(attr as Ident);

    let Some(name) = parse_item_name(&item)
    else {
        unimplemented!()
    };

    let mut expanded = quote! {
        impl FromBytes for #name {
            fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
                Self::from_bits(#ty::from_le_bytes(bytes))
            }

            fn from_be_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
                Self::from_bits(#ty::from_be_bytes(bytes))
            }
        }
    };

    expanded.extend_one(item.to_token_stream());

    TokenStream::from(expanded)
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn derive_as(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as Item);

    #[derive(Parse)]
    struct DeriveAs {
        as_type: Ident,
        _eq_token: Token![=],
        _gt_token: Token![>],
        traits: Punctuated<Ident, Token![,]>,
    }

    let Some(name) = parse_item_name(&item)
    else {
        unimplemented!()
    };

    let derive_as = parse_macro_input!(attr as DeriveAs);

    let ty = derive_as.as_type;
    let mut expanded = derive_as.traits.into_iter().map(|trait_name| {
        match trait_name.to_string().as_str() {
            "PartialEq" => quote! {
                impl PartialEq for #name {
                    fn eq(&self, other: &Self) -> bool {
                        let self_v: #ty = unsafe {
                            std::mem::transmute_copy(self)
                        };

                        let other_v: #ty = unsafe {
                            std::mem::transmute_copy(other)
                        };

                        self_v == other_v
                    }
                }
            },
            "Hash" => quote! {
                impl std::hash::Hash for #name {
                    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                        let v: u16 = unsafe {
                            std::mem::transmute_copy(self)
                        };
                        v.hash(state);
                    }
                }
            },
            _ => unimplemented!()
        }
    }).collect::<proc_macro2::TokenStream>();

    expanded.extend_one(item.to_token_stream());

    TokenStream::from(expanded)
}

fn from_bytes_data_struct_named(
    struct_name: &Ident,
    fields_named: &FieldsNamed,
) -> proc_macro2::TokenStream {
    let fields = parse_fields_named(fields_named);

    /* from_xx_bytes */

    let from_le_bytes_code_pieces = fields
        .iter()
        .map(|(name, ty)| {
            quote! {
                let #name = #ty::from_le_bytes(
                    bytes[ptr..ptr+size_of::<#ty>()].try_into().unwrap()
                );
                ptr += size_of::<#ty>();
            }
        })
        .collect::<proc_macro2::TokenStream>();

    let from_be_bytes_code_pieces = fields
        .iter()
        .map(|(name, ty)| {
            quote! {
                let #name = #ty::from_be_bytes(
                    bytes[ptr..ptr+size_of::<#ty>()].try_into().unwrap()
                );
                ptr += size_of::<#ty>();
            }
        })
        .collect::<proc_macro2::TokenStream>();

    let names = fields
        .iter()
        .map(|(name, _ty)| quote! { #name, })
        .collect::<proc_macro2::TokenStream>();

    quote! {
        impl FromBytes for #struct_name {
            fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
                let mut ptr = 0;

                #from_le_bytes_code_pieces;

                Self {
                    #names
                }
            }

            fn from_be_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
                let mut ptr = 0;

                #from_be_bytes_code_pieces;

                Self {
                    #names
                }
            }
        }
    }
}

fn from_bytes_data_struct_unnamed(
    reprs: Vec<Ident>,
    struct_name: &Ident,
    fields_unnamed: &FieldsUnnamed,
) -> proc_macro2::TokenStream {
    if reprs
        .into_iter()
        .find(|repr| repr.to_string() == "transparent")
        .is_none()
    {
        abort!(struct_name.span(), "Lacking repr(transparent)")
    };

    let fields = fields_unnamed.unnamed.iter().collect::<Vec<_>>();

    if fields.is_empty() {
        abort!(fields_unnamed.span(), "Empty tuple")
    }
    else if fields.len() > 1 {
        abort!(fields_unnamed.span(), "Too many tuple elements")
    }

    match &fields[0].ty {
        Array(array) => {
            let elem_ty = &array.elem;
            let elem_len = &array.len;

            quote! {
                impl FromBytes for #struct_name {
                    fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
                        if size_of::<#elem_ty>() == 1 {
                            Self(bytes)
                        }
                        else {
                            let mut arr: [#elem_ty; #elem_len] = unsafe { std::mem::zeroed() };

                            for i in 0..#elem_len {
                                arr[i] = #elem_ty::from_le_bytes(
                                    bytes[i * size_of::<#elem_ty>()..(i + 1) * size_of::<#elem_ty>()].try_into().unwrap()
                                );
                            }

                            Self(arr)
                        }
                    }

                    fn from_be_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
                        if size_of::<#elem_ty>() == 1 {
                            Self(bytes)
                        }
                        else {
                            let mut arr: [#elem_ty; #elem_len] = unsafe { std::mem::zeroed() };

                            for i in 0..#elem_len {
                                arr[i] = #elem_ty::from_be_bytes(
                                    bytes[i * size_of::<#elem_ty>()..(i + 1) * size_of::<#elem_ty>()].try_into().unwrap()
                                );
                            }

                            Self(arr)
                        }
                    }
                }
            }
        }
        Path(typath) => {
            let Some(bits_type) = typath.path.get_ident()
            else {
                abort!(fields[0].ty.span(), MALFORMED_MSG)
            };

            let Some(bits_type) = REPR_TYPES
                .iter()
                .find(|&&x| x == bits_type.to_string())
                .map(|_| bits_type)
            else {
                abort!(fields[0].ty.span(), MALFORMED_MSG)
            };

            quote! {
                impl FromBytes for #struct_name {
                    fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
                        Self(#bits_type::from_le_bytes(bytes))
                    }

                    fn from_be_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
                        Self(#bits_type::from_be_bytes(bytes))
                    }
                }
            }
        }
        _ => abort!(fields[0].ty.span(), MALFORMED_MSG),
    }
}

fn from_bytes_data_enum(
    reprs: Vec<Ident>,
    struct_name: &Ident,
) -> proc_macro2::TokenStream {
    let Some(bits_type) = reprs.into_iter().find_map(|repr| {
        let repr_name = repr.to_string();

        REPR_TYPES.iter().find(|&&x| x == repr_name).map(|_| repr)
    })
    else {
        abort!(struct_name.span(), MALFORMED_MSG)
    };

    quote! {
        impl FromBytes for #struct_name {
            fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
                unsafe { std::mem::transmute(#bits_type::from_le_bytes(bytes)) }
            }

            fn from_be_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
                unsafe { std::mem::transmute(#bits_type::from_be_bytes(bytes)) }
            }
        }
    }
}

fn to_bytes_data_struct_named(
    struct_name: &Ident,
    fields_named: &FieldsNamed,
) -> proc_macro2::TokenStream {
    let fields = parse_fields_named(fields_named);

    /* to_xx_bytes */

    let to_le_bytes_code_pieces = fields.iter().map(|(field_name, field_ty)| {
        quote! {
            arr[ptr..ptr+size_of::<#field_ty>()].copy_from_slice(&self.#field_name.to_le_bytes());
            ptr += size_of::<#field_ty>();
        }
    }).collect::<proc_macro2::TokenStream>();

    let to_be_bytes_code_pieces = fields.iter().map(|(field_name, field_ty)|
        quote! {
            arr[ptr..ptr+size_of::<#field_ty>()].copy_from_slice(&self.#field_name.to_be_bytes());
            ptr += size_of::<#field_ty>();
        }
    ).collect::<proc_macro2::TokenStream>();

    quote! {
        impl ToBytes for #struct_name {
            fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
                let mut arr = [0; size_of::<Self>()];
                let mut ptr = 0;

                #to_le_bytes_code_pieces;

                arr
            }

            fn to_be_bytes(self) -> [u8; size_of::<Self>()] {
                let mut arr = [0; size_of::<Self>()];
                let mut ptr = 0;

                #to_be_bytes_code_pieces;

                arr
            }
        }
    }
}

fn to_bytes_data_struct_unnamed(
    reprs: Vec<Ident>,
    struct_name: &Ident,
    fields_unnamed: &FieldsUnnamed,
) -> proc_macro2::TokenStream {
    if reprs
        .into_iter()
        .find(|repr| repr.to_string() == "transparent")
        .is_none()
    {
        abort!(struct_name.span(), "Lacking repr(transparent)")
    };

    let fields = fields_unnamed.unnamed.iter().collect::<Vec<_>>();

    if fields.is_empty() {
        abort!(fields_unnamed.span(), "Empty tuple")
    }
    else if fields.len() > 1 {
        abort!(fields_unnamed.span(), "Too many tuple elements")
    }

    match &fields[0].ty {
        Array(array) => {
            let elem_ty = &array.elem;

            quote! {
                impl ToBytes for #struct_name {
                    fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
                        let mut arr = [0; size_of::<Self>()];
                        let mut ptr = 0;

                        for elem in self.0.into_iter() {
                            arr[ptr..ptr + size_of::<#elem_ty>()].copy_from_slice(&elem.to_le_bytes());
                            ptr += size_of::<#elem_ty>();
                        }

                        arr
                    }

                    fn to_be_bytes(self) -> [u8; size_of::<Self>()] {
                        let mut arr = [0; size_of::<Self>()];
                        let mut ptr = 0;

                        for elem in self.0.into_iter() {
                            arr[ptr..ptr + size_of::<#elem_ty>()].copy_from_slice(&elem.to_be_bytes());
                            ptr += size_of::<#elem_ty>();
                        }

                        arr
                    }
                }
            }
        }
        Path(_typath) => {
            quote! {
                impl ToBytes for #struct_name {
                    fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
                        self.0.to_le_bytes()
                    }

                    fn to_be_bytes(self) -> [u8; size_of::<Self>()] {
                        self.0.to_be_bytes()
                    }
                }
            }
        }
        _ => abort!(fields[0].ty.span(), MALFORMED_MSG),
    }
}

fn to_bytes_data_enum(
    reprs: Vec<Ident>,
    struct_name: &Ident,
) -> proc_macro2::TokenStream {
    let Some(bits_type) = reprs.into_iter().find_map(|repr| {
        let repr_name = repr.to_string();

        REPR_TYPES.iter().find(|&&x| x == repr_name).map(|_| repr)
    })
    else {
        abort!(struct_name.span(), MALFORMED_MSG)
    };

    quote! {
        impl ToBytes for #struct_name {
            fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
                let bits: #bits_type = unsafe { std::mem::transmute(self) };

                bits.to_le_bytes()
            }

            fn to_be_bytes(self) -> [u8; size_of::<Self>()] {
                let bits: #bits_type = unsafe { std::mem::transmute(self) };

                bits.to_be_bytes()
            }
        }
    }
}

fn find_reprs(attrs: &[Attribute]) -> Vec<Ident> {
    attrs
        .iter()
        .filter_map(|attr| {
            if let Meta::List(metalist) = &attr.meta {
                if let Some(meta_name) = metalist.path.get_ident() {
                    if meta_name.to_string() == "repr" {
                        let Ok(meta_arg) = metalist.parse_args::<Ident>()
                        else {
                            abort!(meta_name.span(), MALFORMED_MSG)
                        };

                        return Some(meta_arg);
                    }
                }
            }

            None
        })
        .collect::<Vec<_>>()
}

fn parse_fields_named(fields_named: &FieldsNamed) -> Vec<(&Ident, &Type)> {
    fields_named
        .named
        .pairs()
        .map(|pair| {
            let field = pair.into_value();
            (field.ident.as_ref().unwrap(), &field.ty)
        })
        .collect::<Vec<_>>()
}

fn parse_item_name(item: &Item) -> Option<&Ident> {
    Some(match item {
        Item::Enum(item_enum) => &item_enum.ident,
        Item::Struct(item_struct) => &item_struct.ident,
        Item::Union(item_union) => &item_union.ident,
        _ => return None,
    })
}
