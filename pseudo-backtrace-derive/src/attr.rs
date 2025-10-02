use syn::{Attribute, Error, Ident, Result, parse::Parse};

#[derive(Clone, Default)]
pub struct Attrs<'a> {
    pub source: Option<Source<'a>>,
    pub location: Option<Location<'a>>,
    pub stack_error: Option<StackError<'a>>,
}

impl<'a> Attrs<'a> {
    pub fn from_syn(inputs: &'a [Attribute]) -> Result<Self> {
        let mut result = Self::default();

        for attr in inputs {
            if attr.path().is_ident("source") {
                attr.meta.require_path_only()?;
                if result.source.is_some() {
                    return Err(Error::new_spanned(attr, "duplicate `#[source]` attribute"));
                }
                result.source = Some(Source { original: attr });
                continue;
            }

            if attr.path().is_ident("location") {
                attr.meta.require_path_only()?;
                if result.location.is_some() {
                    return Err(Error::new_spanned(
                        attr,
                        "duplicate `#[location]` attribute",
                    ));
                }
                result.location = Some(Location { original: attr });
                continue;
            }

            if attr.path().is_ident("stack_error") {
                let kind: StackErrorKind = attr.parse_args()?;
                if result.stack_error.is_some() {
                    return Err(Error::new_spanned(
                        attr,
                        "duplicate `#[stack_error(...)]` attribute",
                    ));
                }
                result.stack_error = Some(StackError {
                    original: attr,
                    kind,
                });
                continue;
            }
        }

        Ok(result)
    }
}

#[derive(Clone)]
pub struct Source<'a> {
    #[allow(unused)]
    pub original: &'a Attribute,
}

#[derive(Clone)]
pub struct Location<'a> {
    #[allow(unused)]
    pub original: &'a Attribute,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StackErrorKind {
    Stacked,
    Std,
}

impl Parse for StackErrorKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == "std" {
            Ok(Self::Std)
        } else if ident == "stacked" {
            Ok(Self::Stacked)
        } else {
            Err(Error::new(
                input.span(),
                format!(
                    "invalid `#[stack_error({})]` attribute. expected `std` or `stacked`",
                    ident
                ),
            ))
        }
    }
}

#[derive(Clone)]
pub struct StackError<'a> {
    #[allow(unused)]
    pub original: &'a Attribute,
    pub kind: StackErrorKind,
}
