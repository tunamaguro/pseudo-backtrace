mod ast;
mod attr;
mod expand;

use proc_macro::TokenStream;
use syn::{
    DeriveInput,
    parse_macro_input,
};

#[proc_macro_derive(StackError, attributes(source, stack_error, location))]
pub fn derive_stack_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let result = ast::Input::from_input(&input).and_then(expand::expand);
    match result {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}
