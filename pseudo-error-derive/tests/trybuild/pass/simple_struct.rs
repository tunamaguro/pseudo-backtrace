use pseudo_error_derive::StackError;
use pseudo_backtrace::StackError as StackErrorTrait;

#[track_caller]
fn location() -> &'static core::panic::Location<'static> {
    core::panic::Location::caller()
}

#[derive(Debug, StackError)]
pub struct LeafError {
    #[stack_error(end)]
    source: std::io::Error,
    #[location]
    location: &'static core::panic::Location<'static>,
}

impl LeafError {
    pub fn new() -> Self {
        LeafError {
            source: std::io::Error::from_raw_os_error(1),
            location: location(),
        }
    }
}

impl core::fmt::Display for LeafError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "leaf")
    }
}

impl core::error::Error for LeafError {}

#[derive(Debug, StackError)]
pub struct WrappedError {
    #[source]
    inner: LeafError,
    #[location]
    location: &'static core::panic::Location<'static>,
}

impl WrappedError {
    pub fn new(inner: LeafError) -> Self {
        WrappedError {
            inner,
            location: location(),
        }
    }
}

impl core::fmt::Display for WrappedError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "wrapped")
    }
}

impl core::error::Error for WrappedError {}

fn assert_stack_error<T: StackErrorTrait>() {}

pub fn smoke() {
    let leaf = LeafError::new();
    let wrapped = WrappedError::new(leaf);
    assert_stack_error::<WrappedError>();
    let _ = wrapped;
}

fn main() {}
