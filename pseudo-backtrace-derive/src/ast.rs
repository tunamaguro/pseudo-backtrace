use proc_macro2::Span;
use syn::{Error, Generics, Ident, Index, Member, Result};

use crate::attr::Attrs;

pub enum Input<'a> {
    Struct(Struct<'a>),
    Enum(Enum<'a>),
}

impl<'a> Input<'a> {
    pub fn from_input(input: &'a syn::DeriveInput) -> Result<Self> {
        match &input.data {
            syn::Data::Struct(data_struct) => {
                Struct::from_syn(input, data_struct).map(Input::Struct)
            }
            syn::Data::Enum(data_enum) => Enum::from_syn(input, data_enum).map(Input::Enum),
            syn::Data::Union(_) => Err(Error::new_spanned(input, "union is not supported")),
        }
    }
}

#[derive(Clone)]
pub struct Struct<'a> {
    pub ident: Ident,
    pub generics: &'a Generics,
    pub fields: Vec<Field<'a>>,
}

impl<'a> Struct<'a> {
    pub fn from_syn(input: &'a syn::DeriveInput, data: &'a syn::DataStruct) -> Result<Self> {
        let fields = Field::from_fields(&data.fields)?;
        Ok(Self {
            ident: input.ident.clone(),
            generics: &input.generics,
            fields,
        })
    }
}

#[derive(Clone)]
pub struct Enum<'a> {
    pub ident: Ident,
    pub generics: &'a Generics,
    pub variants: Vec<Variant<'a>>,
}

impl<'a> Enum<'a> {
    pub fn from_syn(input: &'a syn::DeriveInput, data: &'a syn::DataEnum) -> Result<Self> {
        let variants = data
            .variants
            .iter()
            .map(Variant::from_syn)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            ident: input.ident.clone(),
            generics: &input.generics,
            variants,
        })
    }
}

#[derive(Clone)]
pub struct Variant<'a> {
    pub original: &'a syn::Variant,
    pub ident: Ident,
    pub fields: Vec<Field<'a>>,
}

impl<'a> Variant<'a> {
    pub fn from_syn(input: &'a syn::Variant) -> Result<Self> {
        Ok(Self {
            original: input,
            ident: input.ident.clone(),
            fields: Field::from_fields(&input.fields)?,
        })
    }
}

#[derive(Clone)]
pub struct Field<'a> {
    pub original: &'a syn::Field,
    pub attrs: Attrs<'a>,
    pub member: Member,
    pub ty: syn::Type,
}

impl<'a> Field<'a> {
    pub fn from_fields(fields: &'a syn::Fields) -> Result<Vec<Self>> {
        if matches!(fields, syn::Fields::Unit) {
            return Err(Error::new_spanned(
                fields,
                "unit struct and unit variant are not supported",
            ));
        }

        fields
            .iter()
            .enumerate()
            .map(|(i, f)| Self::from_syn(i, f))
            .collect()
    }

    pub fn from_syn(i: usize, field: &'a syn::Field) -> Result<Self> {
        let attrs = Attrs::from_syn(&field.attrs)?;
        let member = match &field.ident {
            Some(ident) => Member::Named(ident.clone()),
            None => Member::Unnamed(Index {
                index: i as u32,
                span: Span::call_site(),
            }),
        };

        Ok(Self {
            original: field,
            attrs,
            member,
            ty: field.ty.clone(),
        })
    }
}
