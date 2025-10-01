use pseudo_backtrace::StackError;

#[derive(Debug)]
struct NotError;
impl core::fmt::Display for NotError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        "not-error".fmt(f)
    }
}
// no core::error::Error impl

#[derive(Debug, StackError)]
struct BadOuter {
    #[stack_error(std)]
    source: NotError,
    #[location]
    location: &'static core::panic::Location<'static>,
}

impl core::fmt::Display for BadOuter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        "bad".fmt(f)
    }
}
impl core::error::Error for BadOuter {}

fn main() {}
