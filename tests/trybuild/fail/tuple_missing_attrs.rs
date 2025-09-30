use pseudo_backtrace::StackError;

#[derive(Debug, StackError)]
struct TupleMissingAttrs(std::io::Error, &'static core::panic::Location<'static>);

impl core::fmt::Display for TupleMissingAttrs {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "tuple")
    }
}

impl core::error::Error for TupleMissingAttrs {}

fn main() {}
