use proc_macro2::Span;
use quote::quote;
use syn::{Error, Ident, Result, spanned::Spanned};

use crate::{
    ast::{Enum, Field, Input, Struct, Variant},
    attr::StackErrorKind,
};

pub fn expand(input: Input<'_>) -> Result<proc_macro2::TokenStream> {
    match input {
        Input::Struct(input) => expand_struct(input),
        Input::Enum(input) => expand_enum(input),
    }
}

impl Field<'_> {
    fn is_located_error(&self) -> bool {
        is_located_error(&self.ty)
    }

    fn stack_error_kind(&self) -> StackErrorKind {
        self.attrs
            .stack_error
            .as_ref()
            .map(|f| f.kind)
            .unwrap_or(StackErrorKind::Stacked)
    }
}

impl Struct<'_> {
    fn location_fn(&self) -> Result<proc_macro2::TokenStream> {
        let location = find_location(&self.fields, self.span)?;
        let location_member = location.member.clone();

        let body = if location.is_located_error() {
            quote! { ::pseudo_backtrace::StackError::location(&self.#location_member) }
        } else {
            quote! { self.#location_member }
        };

        Ok(quote! {
            fn location(&self) -> &'static ::core::panic::Location<'static> {
                #body
            }
        })
    }

    fn next_fn(&self) -> Result<proc_macro2::TokenStream> {
        let source = find_source(&self.fields)?;

        let body = if let Some(source) = source {
            let member = source.member.clone();
            let kind = source.stack_error_kind();
            match (is_option_type(&source.ty), kind) {
                (true, StackErrorKind::Stacked) => {
                    quote! {
                        self.#member
                            .as_ref()
                            .map(|__s| ::pseudo_backtrace::Chain::Stacked(__s.as_dyn_stack_error()))
                    }
                }
                (true, StackErrorKind::Std) => {
                    quote! {
                        self.#member
                            .as_ref()
                            .map(|__s| ::pseudo_backtrace::Chain::Std(__s.as_dyn_std_error()))
                    }
                }
                (false, StackErrorKind::Stacked) => {
                    quote! {
                        ::core::option::Option::Some(::pseudo_backtrace::Chain::Stacked(
                            self.#member.as_dyn_stack_error(),
                        ))
                    }
                }
                (false, StackErrorKind::Std) => {
                    quote! {
                        ::core::option::Option::Some(::pseudo_backtrace::Chain::Std(
                            self.#member.as_dyn_std_error(),
                        ))
                    }
                }
            }
        } else {
            quote! { ::core::option::Option::None }
        };

        Ok(quote! {
            fn next<'pseudo_backtrace>(&'pseudo_backtrace self) -> ::core::option::Option<::pseudo_backtrace::Chain<'pseudo_backtrace>> {
                use ::pseudo_backtrace::private::AsDynStdError as _;
                use ::pseudo_backtrace::private::AsDynStackError as _;
                #body
            }
        })
    }
}

fn expand_struct(input: Struct<'_>) -> Result<proc_macro2::TokenStream> {
    let location_fn = input.location_fn()?;
    let next_fn = input.next_fn()?;

    let ident = input.ident.clone();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics ::pseudo_backtrace::StackError for #ident #ty_generics #where_clause {
           #location_fn
           #next_fn
        }
    })
}

impl Variant<'_> {
    fn make_pattern(&self, field: &Field<'_>, binding: &Ident) -> Result<proc_macro2::TokenStream> {
        match self.kind() {
            crate::ast::ContainerKind::Struct => {
                let f = self
                    .fields
                    .iter()
                    .find(|f| f.member == field.member)
                    .ok_or_else(|| {
                        Error::new_spanned(
                            self.original,
                            "same struct field is not found by unknown reason",
                        )
                    })?;
                let field_ident = &f.member;
                Ok(quote! {
                    { #field_ident: #binding, ..}
                })
            }
            crate::ast::ContainerKind::Tuple => {
                let idx = self
                    .fields
                    .iter()
                    .position(|f| f.member == field.member)
                    .ok_or_else(|| {
                        Error::new_spanned(
                            self.original,
                            "same tuple field is not found by unknown reason",
                        )
                    })?;
                let elems = self.fields.iter().enumerate().map(|(i, _)| {
                    if i == idx {
                        quote! { #binding }
                    } else {
                        quote! { _ }
                    }
                });
                Ok(quote! {
                    ( #(#elems),* )
                })
            }
        }
    }

    fn location_body(&self) -> Result<proc_macro2::TokenStream> {
        let variant_ident = self.ident.clone();
        let location = find_location(&self.fields, self.original.span())?;

        // Build a pattern that binds the location field to a local ident
        let binding = quote::format_ident!("__stack_error_location");
        let pattern = self.make_pattern(&location, &binding)?;

        // Compute the value expression
        let value = if location.is_located_error() {
            quote! { ::pseudo_backtrace::StackError::location(#binding) }
        } else {
            quote! { #binding }
        };

        Ok(quote! { #variant_ident #pattern => #value })
    }

    fn next_body(&self) -> Result<proc_macro2::TokenStream> {
        let variant_ident = self.ident.clone();
        let Some(source) = find_source(&self.fields)? else {
            let ts = match self.kind() {
                crate::ast::ContainerKind::Struct => quote! { #variant_ident { .. } => { None } },
                crate::ast::ContainerKind::Tuple => {
                    let elems = self.fields.iter().map(|_| quote! {_});
                    quote! { #variant_ident( #(#elems),* ) => { None } }
                }
            };
            return Ok(ts);
        };

        // Build a pattern that binds the source field to a local ident
        let binding = quote::format_ident!("__stack_error_source");
        let pattern = self.make_pattern(&source, &binding)?;

        // Build the body depending on Option<T> and stack_error kind
        let kind = source.stack_error_kind();
        let body = match (is_option_type(&source.ty), kind) {
            (true, StackErrorKind::Stacked) => {
                quote! {
                    #binding
                        .as_ref()
                        .map(|__s| ::pseudo_backtrace::Chain::Stacked(__s.as_dyn_stack_error()))
                }
            }
            (true, StackErrorKind::Std) => {
                quote! {
                    #binding
                        .as_ref()
                        .map(|__s| ::pseudo_backtrace::Chain::Std(__s.as_dyn_std_error()))
                }
            }
            (false, StackErrorKind::Stacked) => {
                quote! {
                    ::core::option::Option::Some(::pseudo_backtrace::Chain::Stacked(
                        #binding.as_dyn_stack_error(),
                    ))
                }
            }
            (false, StackErrorKind::Std) => {
                quote! {
                    ::core::option::Option::Some(::pseudo_backtrace::Chain::Std(
                        #binding.as_dyn_std_error(),
                    ))
                }
            }
        };

        Ok(quote! { #variant_ident #pattern => { #body } })
    }
}

fn expand_enum(input: Enum<'_>) -> Result<proc_macro2::TokenStream> {
    let location_arms = input
        .variants
        .iter()
        .map(|v| v.location_body())
        .collect::<Result<Vec<_>>>()?;

    let next_arms = input
        .variants
        .iter()
        .map(|v| v.next_body())
        .collect::<Result<Vec<_>>>()?;

    let ident = input.ident;
    let location_fn = quote! {
        fn location(&self) -> &'static ::core::panic::Location<'static> {
            use #ident::*;
            match self {
                #(#location_arms,)*
            }
        }
    };
    let next_fn = quote! {
        fn next<'pseudo_backtrace>(&'pseudo_backtrace self) -> ::core::option::Option<::pseudo_backtrace::Chain<'pseudo_backtrace>> {
            use #ident::*;
            use ::pseudo_backtrace::private::AsDynStdError as _;
            use ::pseudo_backtrace::private::AsDynStackError as _;
            match self {
                #(#next_arms,)*
            }
        }
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics ::pseudo_backtrace::StackError for #ident #ty_generics #where_clause {
           #location_fn
           #next_fn
        }
    })
}

fn find_location<'a>(fields: &[Field<'a>], source_span: Span) -> Result<Field<'a>> {
    // find #[loaction] attribute
    let mut it = fields.iter().filter(|f| f.attrs.location.is_some());
    match (it.next(), it.next()) {
        (Some(_), Some(second)) => {
            return Err(Error::new_spanned(
                second.original,
                "duplicate `#[location]` attribute.",
            ));
        }
        (Some(first), None) => {
            return Ok(first.clone());
        }
        _ => {}
    }

    // find named `location`
    let mut it = fields.iter().filter(|f| match &f.member {
        syn::Member::Named(ident) => ident == "location",
        _ => false,
    });
    if let Some(f) = it.next() {
        return Ok(f.clone());
    };

    // find `LocatedError`
    if let Some(f) = find_located_error(fields)? {
        return Ok(f.clone());
    }

    Err(Error::new(
        source_span,
        "need `#[location]` attribute or field named `location`",
    ))
}

fn find_source<'a>(fields: &[Field<'a>]) -> Result<Option<Field<'a>>> {
    // find #[source] and #[stack_error] attribute
    let mut it = fields
        .iter()
        .filter(|f| f.attrs.source.is_some() || f.attrs.stack_error.is_some());
    match (it.next(), it.next()) {
        (Some(_), Some(second)) => {
            return Err(Error::new_spanned(
                second.original,
                "duplicate `#[source]` or `#[stack_error]` attribute",
            ));
        }
        (Some(first), None) => {
            return Ok(Some(first.clone()));
        }
        _ => {}
    }

    // find named `source`
    let mut it = fields.iter().filter(|f| match &f.member {
        syn::Member::Named(ident) => ident == "source",
        _ => false,
    });
    if let Some(f) = it.next() {
        return Ok(Some(f.clone()));
    };

    // find `LocatedError`
    if let Some(f) = find_located_error(fields)? {
        return Ok(Some(f.clone()));
    }

    Ok(None)
}

fn find_located_error<'a>(fields: &[Field<'a>]) -> Result<Option<Field<'a>>> {
    let mut it = fields.iter().filter(|f| is_located_error(&f.ty));
    match (it.next(), it.next()) {
        (Some(_), Some(second)) => Err(Error::new_spanned(
            second.original,
            "duplicate `LocatedError` field",
        )),
        (Some(first), None) => Ok(Some(first.clone())),
        _ => Ok(None),
    }
}

fn is_located_error(ty: &syn::Type) -> bool {
    let ty = match ty {
        syn::Type::Reference(r) => &*r.elem,
        _ => ty,
    };

    let syn::Type::Path(type_path) = ty else {
        return false;
    };
    let Some(last) = type_path.path.segments.last() else {
        return false;
    };
    last.ident == "LocatedError"
}

fn is_option_type(ty: &syn::Type) -> bool {
    let path = match ty {
        syn::Type::Path(ty) => &ty.path,
        _ => return false,
    };
    match path.segments.last() {
        Some(last) => last.ident == "Option",
        None => false,
    }
}
