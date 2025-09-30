use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet};
use syn::visit::Visit;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Field, Fields, Generics, Ident, Member,
    parse_macro_input, spanned::Spanned,
};

#[proc_macro_derive(StackError, attributes(source, stack_error, location))]
pub fn derive_stack_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand(input: DeriveInput) -> syn::Result<TokenStream2> {
    let ident = input.ident;
    let generics = input.generics;

    match input.data {
        Data::Struct(data) => expand_struct(ident, generics, data),
        Data::Enum(data) => expand_enum(ident, generics, data),
        Data::Union(_) => Err(syn::Error::new(
            Span::call_site(),
            "StackError cannot be derived for unions",
        )),
    }
}

fn expand_struct(ident: Ident, generics: Generics, data: DataStruct) -> syn::Result<TokenStream2> {
    let style = match &data.fields {
        Fields::Named(_) => FieldsStyle::Named,
        Fields::Unnamed(_) => FieldsStyle::Unnamed,
        Fields::Unit => {
            return Err(syn::Error::new(
                ident.span(),
                "unit structs do not support #[derive(StackError)]",
            ));
        }
    };

    let fields = collect_fields(&data.fields)?;
    let location_index = resolve_location(&fields, style.allows_names(), ident.span())?;
    let source = resolve_source(&fields, style.allows_names())?;

    let mut generics = generics;
    let mut bounds = BoundsTracker::new(&generics);
    if let Some(info) = &source {
        bounds.collect(&fields[info.index].ty, info.is_terminal);
    }
    bounds.apply(&mut generics);

    let location_member = &fields[location_index].member;
    let next_body = match &source {
        Some(info) => build_next_struct(&fields[info.index].member, info.is_terminal),
        None => quote! { ::core::option::Option::None },
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::pseudo_backtrace::StackError for #ident #ty_generics #where_clause {
            fn location(&self) -> &'static ::core::panic::Location<'static> {
                self.#location_member
            }

            fn next<'a>(&'a self) -> ::core::option::Option<::pseudo_backtrace::ErrorDetail<'a>> {
                #next_body
            }
        }
    })
}

fn expand_enum(ident: Ident, generics: Generics, data: DataEnum) -> syn::Result<TokenStream2> {
    let mut variant_infos = Vec::with_capacity(data.variants.len());
    let mut errors: Option<syn::Error> = None;

    for variant in data.variants {
        let style = match &variant.fields {
            Fields::Named(_) => FieldsStyle::Named,
            Fields::Unnamed(_) => FieldsStyle::Unnamed,
            Fields::Unit => {
                errors = combine_error(
                    errors,
                    syn::Error::new(
                        variant.ident.span(),
                        "unit variants do not support #[derive(StackError)]",
                    ),
                );
                continue;
            }
        };

        let fields = match collect_fields(&variant.fields) {
            Ok(fields) => fields,
            Err(err) => {
                errors = combine_error(errors, err);
                continue;
            }
        };

        let location_index =
            match resolve_location(&fields, style.allows_names(), variant.ident.span()) {
                Ok(index) => index,
                Err(err) => {
                    errors = combine_error(errors, err);
                    continue;
                }
            };

        let source = match resolve_source(&fields, style.allows_names()) {
            Ok(source) => source,
            Err(err) => {
                errors = combine_error(errors, err);
                continue;
            }
        };

        let source_binding = source
            .as_ref()
            .map(|_| format_ident!("__stack_error_source"));

        variant_infos.push(VariantInfo {
            ident: variant.ident,
            style,
            fields,
            location_index,
            source,
            location_binding: format_ident!("__stack_error_location"),
            source_binding,
        });
    }

    if let Some(err) = errors {
        return Err(err);
    }

    let mut generics = generics;
    let mut bounds = BoundsTracker::new(&generics);
    for variant in &variant_infos {
        if let Some(source) = &variant.source {
            bounds.collect(&variant.fields[source.index].ty, source.is_terminal);
        }
    }
    bounds.apply(&mut generics);

    let location_arms = variant_infos.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let pattern = variant.location_pattern();
        let value = &variant.location_binding;
        quote! {
            Self::#variant_ident #pattern => #value
        }
    });

    let next_arms = variant_infos.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let pattern = variant.source_pattern();
        let body = variant.next_body();
        quote! {
            Self::#variant_ident #pattern => #body
        }
    });

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::pseudo_backtrace::StackError for #ident #ty_generics #where_clause {
            fn location(&self) -> &'static ::core::panic::Location<'static> {
                match self {
                    #(#location_arms,)*
                }
            }

            fn next<'a>(&'a self) -> ::core::option::Option<::pseudo_backtrace::ErrorDetail<'a>> {
                match self {
                    #(#next_arms,)*
                }
            }
        }
    })
}

#[derive(Clone)]
struct FieldInfo {
    member: Member,
    ident: Option<Ident>,
    ty: syn::Type,
    attrs: FieldAttrs,
    span: Span,
}

#[derive(Clone, Copy)]
enum FieldsStyle {
    Named,
    Unnamed,
}

impl FieldsStyle {
    fn allows_names(self) -> bool {
        matches!(self, FieldsStyle::Named)
    }
}

#[derive(Default, Clone)]
struct FieldAttrs {
    is_source: bool,
    is_location: bool,
    is_terminal: bool,
}

struct SourceInfo {
    index: usize,
    is_terminal: bool,
}

struct VariantInfo {
    ident: Ident,
    style: FieldsStyle,
    fields: Vec<FieldInfo>,
    location_index: usize,
    source: Option<SourceInfo>,
    location_binding: Ident,
    source_binding: Option<Ident>,
}

fn collect_fields(fields: &Fields) -> syn::Result<Vec<FieldInfo>> {
    let mut out = Vec::new();

    match fields {
        Fields::Named(named) => {
            for field in named.named.iter() {
                out.push(build_field_info(field, out.len(), true)?);
            }
        }
        Fields::Unnamed(unnamed) => {
            for (idx, field) in unnamed.unnamed.iter().enumerate() {
                out.push(build_field_info(field, idx, false)?);
            }
        }
        Fields::Unit => {}
    }

    Ok(out)
}

fn build_field_info(field: &Field, index: usize, named: bool) -> syn::Result<FieldInfo> {
    let attrs = parse_field_attrs(field)?;
    let member = if named {
        Member::Named(field.ident.clone().expect("named field missing ident"))
    } else {
        Member::Unnamed(syn::Index::from(index))
    };

    Ok(FieldInfo {
        member,
        ident: field.ident.clone(),
        ty: field.ty.clone(),
        attrs,
        span: field.span(),
    })
}

fn parse_field_attrs(field: &Field) -> syn::Result<FieldAttrs> {
    let mut attrs = FieldAttrs::default();

    for attr in &field.attrs {
        if attr.path().is_ident("source") {
            if attrs.is_source {
                return Err(syn::Error::new_spanned(
                    attr,
                    "duplicate #[source] attribute",
                ));
            }
            attrs.is_source = true;
            continue;
        }

        if attr.path().is_ident("location") {
            if attrs.is_location {
                return Err(syn::Error::new_spanned(
                    attr,
                    "duplicate #[location] attribute",
                ));
            }
            attrs.is_location = true;
            continue;
        }

        if attr.path().is_ident("stack_error") {
            match attr.parse_args_with(|input: syn::parse::ParseStream| {
                let ident: Ident = input.parse()?;
                if ident == "end" {
                    Ok(())
                } else {
                    Err(syn::Error::new(ident.span(), "expected `end`"))
                }
            }) {
                Ok(()) => {
                    if attrs.is_terminal {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "duplicate #[stack_error(end)] attribute",
                        ));
                    }
                    attrs.is_terminal = true;
                }
                Err(err) => {
                    return Err(syn::Error::new_spanned(
                        attr,
                        format!("invalid #[stack_error] attribute: {}", err),
                    ));
                }
            }

            continue;
        }
    }

    Ok(attrs)
}

fn resolve_location(
    fields: &[FieldInfo],
    allow_name: bool,
    missing_span: Span,
) -> syn::Result<usize> {
    let mut index = None;

    for (idx, field) in fields.iter().enumerate() {
        if field.attrs.is_location {
            if index.is_some() {
                return Err(syn::Error::new(
                    field.span,
                    "multiple fields marked with #[location]",
                ));
            }
            index = Some(idx);
        }
    }

    if let Some(idx) = index {
        return Ok(idx);
    }

    if allow_name
        && let Some((idx, _)) = fields
            .iter()
            .enumerate()
            .find(|(_, field)| matches!(&field.ident, Some(ident) if ident == "location"))
        {
            return Ok(idx);
        }

    Err(syn::Error::new(
        missing_span,
        "missing #[location] attribute or field named `location`",
    ))
}

fn resolve_source(fields: &[FieldInfo], allow_name: bool) -> syn::Result<Option<SourceInfo>> {
    let mut source_candidates: Vec<usize> = Vec::new();
    let mut terminal_candidates: Vec<usize> = Vec::new();

    for (idx, field) in fields.iter().enumerate() {
        if field.attrs.is_source {
            source_candidates.push(idx);
        }
        if field.attrs.is_terminal {
            terminal_candidates.push(idx);
        }
    }

    if source_candidates.len() > 1 {
        let span = fields[source_candidates[1]].span;
        return Err(syn::Error::new(
            span,
            "multiple fields marked with #[source]",
        ));
    }

    if source_candidates.len() == 1 {
        let idx = source_candidates[0];
        let is_terminal = fields[idx].attrs.is_terminal;
        return Ok(Some(SourceInfo {
            index: idx,
            is_terminal,
        }));
    }

    if terminal_candidates.len() > 1 {
        let span = fields[terminal_candidates[1]].span;
        return Err(syn::Error::new(
            span,
            "multiple fields marked with #[stack_error(end)]",
        ));
    }

    if let Some(idx) = terminal_candidates.first().copied() {
        return Ok(Some(SourceInfo {
            index: idx,
            is_terminal: true,
        }));
    }

    if allow_name
        && let Some((idx, _)) = fields
            .iter()
            .enumerate()
            .find(|(_, field)| matches!(&field.ident, Some(ident) if ident == "source"))
        {
            return Ok(Some(SourceInfo {
                index: idx,
                is_terminal: false,
            }));
        }

    Ok(None)
}

fn build_next_struct(member: &Member, is_terminal: bool) -> TokenStream2 {
    if is_terminal {
        quote! {
            ::core::option::Option::Some(::pseudo_backtrace::ErrorDetail::End(
                &self.#member as &'a dyn ::core::error::Error,
            ))
        }
    } else {
        quote! {
            ::core::option::Option::Some(::pseudo_backtrace::ErrorDetail::Stacked(
                &self.#member as &'a dyn ::pseudo_backtrace::StackError,
            ))
        }
    }
}

impl VariantInfo {
    fn location_pattern(&self) -> TokenStream2 {
        match self.style {
            FieldsStyle::Named => {
                let field_ident = self.fields[self.location_index]
                    .ident
                    .as_ref()
                    .expect("named field missing ident")
                    .clone();
                let binding = &self.location_binding;
                quote! { { #field_ident: #binding, .. } }
            }
            FieldsStyle::Unnamed => {
                let binding = &self.location_binding;
                let patterns = self.fields.iter().enumerate().map(|(idx, _)| {
                    if idx == self.location_index {
                        quote! { #binding }
                    } else {
                        quote! { _ }
                    }
                });
                quote! { ( #(#patterns),* ) }
            }
        }
    }

    fn source_pattern(&self) -> TokenStream2 {
        match &self.source {
            Some(source) => match self.style {
                FieldsStyle::Named => {
                    let field_ident = self.fields[source.index]
                        .ident
                        .as_ref()
                        .expect("named field missing ident")
                        .clone();
                    let binding = self
                        .source_binding
                        .as_ref()
                        .expect("source binding missing");
                    quote! { { #field_ident: #binding, .. } }
                }
                FieldsStyle::Unnamed => {
                    let binding = self
                        .source_binding
                        .as_ref()
                        .expect("source binding missing");
                    let patterns = self.fields.iter().enumerate().map(|(idx, _)| {
                        if idx == source.index {
                            quote! { #binding }
                        } else {
                            quote! { _ }
                        }
                    });
                    quote! { ( #(#patterns),* ) }
                }
            },
            None => match self.style {
                FieldsStyle::Named => quote! { { .. } },
                FieldsStyle::Unnamed => {
                    let patterns = self.fields.iter().map(|_| quote! { _ });
                    quote! { ( #(#patterns),* ) }
                }
            },
        }
    }

    fn next_body(&self) -> TokenStream2 {
        match &self.source {
            Some(source) => {
                let binding = self
                    .source_binding
                    .as_ref()
                    .expect("source binding missing");
                if source.is_terminal {
                    quote! {
                        ::core::option::Option::Some(::pseudo_backtrace::ErrorDetail::End(
                            #binding as &'a dyn ::core::error::Error,
                        ))
                    }
                } else {
                    quote! {
                        ::core::option::Option::Some(::pseudo_backtrace::ErrorDetail::Stacked(
                            #binding as &'a dyn ::pseudo_backtrace::StackError,
                        ))
                    }
                }
            }
            None => quote! { ::core::option::Option::None },
        }
    }
}

struct BoundsTracker {
    params: BTreeMap<String, Ident>,
    needs_error: BTreeSet<String>,
    needs_stack: BTreeSet<String>,
}

impl BoundsTracker {
    fn new(generics: &Generics) -> Self {
        let params = generics
            .type_params()
            .map(|param| (param.ident.to_string(), param.ident.clone()))
            .collect();

        BoundsTracker {
            params,
            needs_error: BTreeSet::new(),
            needs_stack: BTreeSet::new(),
        }
    }

    fn collect(&mut self, ty: &syn::Type, is_terminal: bool) {
        let mut visitor = TypeParamCollector {
            params: &self.params,
            found: BTreeSet::new(),
        };
        visitor.visit_type(ty);

        for name in visitor.found {
            self.needs_error.insert(name.clone());
            if !is_terminal {
                self.needs_stack.insert(name);
            }
        }
    }

    fn apply(&self, generics: &mut Generics) {
        for param in generics.type_params_mut() {
            let name = param.ident.to_string();
            if self.needs_stack.contains(&name) {
                param
                    .bounds
                    .push(syn::parse_quote!(::pseudo_backtrace::StackError));
            }
            if self.needs_error.contains(&name) {
                param.bounds.push(syn::parse_quote!(::core::error::Error));
            }
        }
    }
}

struct TypeParamCollector<'a> {
    params: &'a BTreeMap<String, Ident>,
    found: BTreeSet<String>,
}

impl<'a, 'ast> Visit<'ast> for TypeParamCollector<'a> {
    fn visit_type_path(&mut self, type_path: &'ast syn::TypePath) {
        if type_path.qself.is_none()
            && let Some(segment) = type_path.path.segments.first() {
                let ident = &segment.ident;
                let name = ident.to_string();
                if self.params.contains_key(&name) {
                    self.found.insert(name);
                }
            }

        syn::visit::visit_type_path(self, type_path);
    }
}

fn combine_error(acc: Option<syn::Error>, next: syn::Error) -> Option<syn::Error> {
    match acc {
        Some(mut err) => {
            err.combine(next);
            Some(err)
        }
        None => Some(next),
    }
}
