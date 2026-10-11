//! This crate is only designed to be used for `slipstream` crates.

use darling::FromDeriveInput;
use heck::ToShoutySnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};

mod inspect;

/// Separate function because you cannot call proc macro functions directly.
fn derive_inspect_inner(input: TokenStream, is_bitfield: bool) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let parsed = inspect::Input::from_derive_input(&input).expect("failed to parse input");
    TokenStream::from(parsed.into_token_stream(is_bitfield))
}

/// Fields not marked with `pub` will not be shown in the property window by default.
/// This can be overriden with `hidden = false`, the attribute will always take priority over field
/// visibility.
#[proc_macro_derive(Inspect, attributes(inspect))]
pub fn derive_inspect(input: TokenStream) -> TokenStream {
    derive_inspect_inner(input, false)
}

struct TyAttrArg {
    ty: syn::Type,
}

impl syn::parse::Parse for TyAttrArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self { ty: input.parse()? })
    }
}

fn is_inspect_attr(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("inspect")
}

#[proc_macro_attribute]
pub fn inspect_bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    let bitfield_ty = syn::parse_macro_input!(args as TyAttrArg).ty;

    let mut item = {
        let input = input.clone();
        syn::parse_macro_input!(input as syn::DeriveInput)
    };

    // Read the `#[inspect(..)]` attributes first, while they are still there.
    let inspect_derived = match inspect::Input::from_derive_input(&item) {
        Ok(parsed) => parsed.into_token_stream(true),
        Err(err) => return TokenStream::from(err.write_errors()),
    };

    // Strip them from the struct and its fields, so the compiler and `#[bitfield]`
    // never see an attribute nobody has declared.
    item.attrs.retain(|attr| !is_inspect_attr(attr));
    if let syn::Data::Struct(data) = &mut item.data {
        for field in data.fields.iter_mut() {
            field.attrs.retain(|attr| !is_inspect_attr(attr));
        }
    }

    let expanded = quote! {
        #[bitfield_struct::bitfield(#bitfield_ty)]
        #item
        #inspect_derived
    };

    TokenStream::from(expanded)
}

#[derive(darling::FromVariant)]
#[darling(attributes(try_from))]
struct FieldVariant {
    pub ident: syn::Ident,
    pub fields: darling::ast::Fields<()>,
    pub discriminant: Option<syn::Expr>,
    #[darling(default)]
    pub ignore: bool,
}

#[derive(darling::FromDeriveInput)]
#[darling(attributes(try_from))]
struct Input {
    pub ident: syn::Ident,
    pub data: darling::ast::Data<FieldVariant, ()>,
}

#[proc_macro_attribute]
pub fn try_from(args: TokenStream, input: TokenStream) -> TokenStream {
    let ty = syn::parse_macro_input!(args as TyAttrArg).ty;

    let raw_input = syn::parse_macro_input!(input as syn::DeriveInput);
    let input = match Input::from_derive_input(&raw_input) {
        Ok(i) => i,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let Input { ident, data } = input;
    let darling::ast::Data::Enum(enum_data) = data else {
        return darling::Error::unsupported_shape(
            "The `from_bytes` attribute is only supported on enums",
        )
        .write_errors()
        .into();
    };

    // Check whether the enum has only unit variants
    let is_unit = enum_data
        .iter()
        .all(|variant| variant.fields.style == darling::ast::Style::Unit);

    if !is_unit {
        return darling::Error::unsupported_shape(
            "The `from_bytes` attribute is not supported on enums with non-unit variants",
        )
        .write_errors()
        .into();
    }

    let const_idents = enum_data
        .iter()
        .filter_map(|v| {
            (!v.ignore).then(|| {
                format_ident!(
                    "__TRY_FROM_MATCH_CONST_{}",
                    v.ident.clone().to_string().to_shouty_snake_case()
                )
            })
        })
        .collect::<Vec<_>>();

    let pattern_consts = enum_data
        .iter()
        .filter(|v| !v.ignore)
        .zip(const_idents.iter())
        .map(|(variant, const_ident)| {
            let Some(disc) = &variant.discriminant else {
                return darling::Error::unsupported_shape(
                    "every variant must have an explicitly set discriminant",
                )
                .write_errors()
                .into();
            };

            quote! {
                const #const_ident: #ty = #disc;
            }
        })
        .collect::<Vec<_>>();

    let patterns = enum_data
        .iter()
        .filter(|v| !v.ignore)
        .zip(const_idents.iter())
        .map(|(variant, const_ident)| {
            let ident = &variant.ident;
            quote! {
                Self::#const_ident => Self::#ident
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        #raw_input

        impl #ident {
            #(#pattern_consts)*
        }

        impl TryFrom<#ty> for #ident {
            type Error = slipstream_shared::error::SlipstreamError;

            fn try_from(value: #ty) -> Result<Self, Self::Error> {
                Ok(match value {
                    #(#patterns),*,
                    _ => return Err(slipstream_shared::error::CorruptionError {
                        reason: format!("variant {value} out of range for enum `{}`", stringify!(#ident)),
                        ..Default::default()
                    }.into())
                })
            }
        }
    };

    TokenStream::from(expanded)
}
