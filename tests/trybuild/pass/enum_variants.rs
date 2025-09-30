use pseudo_backtrace::StackError;
use pseudo_backtrace::StackError as StackErrorTrait;

#[track_caller]
fn location() -> &'static core::panic::Location<'static> {
    core::panic::Location::caller()
}

#[derive(Debug, StackError)]
pub struct TerminalError {
    #[stack_error(end)]
    source: std::io::Error,
    #[location]
    location: &'static core::panic::Location<'static>,
}

impl core::fmt::Display for TerminalError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "terminal")
    }
}

impl core::error::Error for TerminalError {}

#[derive(Debug, StackError)]
pub enum EnumError {
    StructVariant {
        #[source]
        inner: TerminalError,
        #[location]
        error_location: &'static core::panic::Location<'static>,
    },
    TupleVariant(
        #[source] TerminalError,
        #[location] &'static core::panic::Location<'static>,
    ),
}

impl EnumError {
    pub fn from_struct(inner: TerminalError) -> Self {
        EnumError::StructVariant {
            inner,
            error_location: location(),
        }
    }

    pub fn from_tuple(inner: TerminalError) -> Self {
        EnumError::TupleVariant(inner, location())
    }
}

impl core::fmt::Display for EnumError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EnumError::StructVariant { .. } => write!(f, "enum-struct"),
            EnumError::TupleVariant(..) => write!(f, "enum-tuple"),
        }
    }
}

impl core::error::Error for EnumError {}

fn assert_stack_error<T: StackErrorTrait>() {}

pub fn smoke() {
    assert_stack_error::<TerminalError>();
    assert_stack_error::<EnumError>();
}

fn main() {}
