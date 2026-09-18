#![deny(warnings)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! Procedural derive macro for `gpui-navigation` route segments.

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Data, DeriveInput, Error, parse_macro_input, spanned::Spanned};

/// Implements `gpui_navigation::RouteSegment` for a route enum.
///
/// The generated implementation provides type erasure, checked downcasting,
/// equality, type identification, and debug/display formatting for use by
/// `gpui-navigation::NavPath`.
#[proc_macro_derive(RouteSegment)]
pub fn derive_route_segment(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if !input.generics.params.is_empty() {
        return Error::new_spanned(
            &input.generics,
            "RouteSegment cannot be derived for generic types",
        )
        .to_compile_error()
        .into();
    }

    if !matches!(&input.data, Data::Enum(_)) {
        return Error::new_spanned(&input.ident, "RouteSegment can only be derived for enums")
            .to_compile_error()
            .into();
    }

    let crate_path = match crate_path() {
        Ok(path) => path,
        Err(error) => {
            return Error::new(input.span(), error).to_compile_error().into();
        }
    };

    let name = &input.ident;

    quote! {
        #[automatically_derived]
        impl #crate_path::RouteSegment for #name
        where
            Self: ::core::fmt::Debug
                + ::core::cmp::PartialEq
                + 'static,
        {
            fn as_any(&self) -> &dyn ::core::any::Any {
                self
            }

            fn equals(
                &self,
                other: &dyn #crate_path::RouteSegment,
            ) -> bool {
                other
                    .as_any()
                    .downcast_ref::<Self>()
                    .is_some_and(|other| self == other)
            }

            fn type_name(&self) -> &'static str {
                ::core::any::type_name::<Self>()
            }

            fn fmt_debug(
                &self,
                formatter: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::fmt::Result {
                ::core::fmt::Debug::fmt(self, formatter)
            }

            fn fmt_display(
                &self,
                formatter: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::fmt::Result {
                ::core::fmt::Debug::fmt(self, formatter)
            }
        }
    }
    .into()
}

fn crate_path() -> Result<proc_macro2::TokenStream, String> {
    match crate_name("gpui-navigation") {
        Ok(FoundCrate::Itself) => Ok(quote!(crate)),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            Ok(quote!(::#ident))
        }
        Err(error) => Err(format!(
            "could not locate the `gpui-navigation` dependency: {error}"
        )),
    }
}
