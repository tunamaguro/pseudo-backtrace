use proc_macro2::Span;
use syn::{Error, Result};

use crate::ast::Field;

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

fn find_source<'a>(fields: &[Field<'a>], source_span: Span) -> Result<Field<'a>> {
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
            return Ok(first.clone());
        }
        _ => {}
    }

    // find named `source`
    let mut it = fields.iter().filter(|f| match &f.member {
        syn::Member::Named(ident) => ident == "source",
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
        "need `#[source]`, `#[stack_error(stacked|std)]` attribute or field named `source`",
    ))
}

fn find_located_error<'a>(fields: &[Field<'a>]) -> Result<Option<Field<'a>>> {
    let mut it = fields.iter().filter(|f| is_located_error(&f.ty));
    match (it.next(), it.next()) {
        (Some(_), Some(second)) => {
            return Err(Error::new_spanned(
                second.original,
                "duplicate `LocatedError` field",
            ));
        }
        (Some(first), None) => {
            return Ok(Some(first.clone()));
        }
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
