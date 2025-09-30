# StackError Derive Macro Design

## Goal
Implement a `#[derive(StackError)]` proc macro that generates `StackError` implementations for structs and enum variants following the rules in `MEMO.md`.

## Supported Targets
- Structs and tuple structs.
- Enums whose variants are struct-like or tuple-like.
- Unit structs and unit enum variants are rejected with a compile error.

## Parsing Strategy
1. Use `syn::parse_macro_input!` to obtain a `DeriveInput`.
2. Branch on `DeriveInput::data`:
   - `Data::Struct`: extract a single `Shape` describing the fields.
   - `Data::Enum`: map each variant into a `Shape`, validating that all variants provide a location field.
3. `Shape` captures:
   - Field list with meta information (identifier/index, type, attrs).
   - Optional references to the `source` and `location` fields after resolution.
   - Flags for whether the source field is a terminal (`#[stack_error(end)]`).

## Attribute Handling
- Supported attributes: `#[source]`, `#[stack_error(end)]`, `#[location]`.
- All other outer attributes are ignored.
- While scanning attributes, record occurrences and enforce exclusivity:
  - Multiple `#[location]` or multiple `#[source]` on the same field → compile error.
  - Both `#[source]` and `#[stack_error(end)]` on the same field → allowed (terminal source) and treated as `End`.
  - Multiple candidate fields marked with the same attribute kind → compile error.
- Tuples require explicit attributes for both source and location because field names are inaccessible.

## Field Resolution
For each shape, locate special fields in priority order:
1. `location` field:
   - Prefer explicit `#[location]`.
   - Otherwise fallback to field named `location` (only for struct-like shapes).
   - If still missing → compile error.
2. `source` field:
   - Prefer explicit `#[source]`.
   - Otherwise field with `#[stack_error(end)]` (mark as terminal).
   - Otherwise field named `source` (struct-like only).
   - If still missing, the generated `next()` returns `None`.
- If multiple fields satisfy the same priority level (e.g., two fields named `source`) → compile error.

## Generated Implementation
```
impl StackError for <Type> {
    fn location(&self) -> &'static Location {
        self.<location_field>
    }

    fn next<'a>(&'a self) -> Option<ErrorDetail<'a>> {
        <expanded logic>
    }
}
```

`next()` expansion:
```
match <source_field> {
    None => None,
    Some(reference) => Some(match is_terminal {
        true => ErrorDetail::End(reference as &'a dyn Error),
        false => ErrorDetail::Stacked(reference as &'a dyn StackError),
    }),
}
```
- When no source field is resolved, the method simply returns `None`.
- Terminal sources (`#[stack_error(end)]`) produce `ErrorDetail::End`.
- Otherwise, produce `ErrorDetail::Stacked`.

### Borrow Handling
- Ensure references produce `&dyn ...` by coercing with `as` casts.
- For owned fields (not references), borrow with `&self.field`.
- `next()` should borrow `self` immutably and return references with lifetime `'a`.

## Generics
- Mirror the original generics and where clause in the generated impl block.
- When the resolved source field references type parameters declared on the target type, extend the `where` clause:
  - For stacked sources, require the generic parameter to implement both `StackError` and `core::error::Error` (the latter satisfies the spec note).
  - For terminal (`#[stack_error(end)]`) sources, require only `core::error::Error`.
- Use `syn::visit::Visit` to detect which generics appear in the source field type and apply deduplicated bounds.
- Existing `where` predicates are preserved and extended.

## Errors & Diagnostics
- Use `syn::Error::new_spanned` for precise diagnostics.
- Combine multiple errors via `Accumulator` pattern (Vec of `syn::Error`) and convert to `TokenStream` using `CombineExt` pattern from `thiserror` as reference.
- Provide descriptive messages (e.g., "missing #[location] or field named `location`").

## Testing Strategy
- Add UI-style tests using `trybuild` is deferred; start with inline `tests` module in crate root using `quote::quote!` to ensure expansions compile (`syn::parse_quote!` + `macrotest` later).
- Provide documentation examples in `lib.rs` doc comments once implementation stabilizes.
