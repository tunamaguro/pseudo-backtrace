use pseudo_error_derive::StackError;

#[derive(Debug, StackError)]
struct DuplicateLocation {
    #[location]
    first: &'static core::panic::Location<'static>,
    #[location]
    second: &'static core::panic::Location<'static>,
}

impl core::fmt::Display for DuplicateLocation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "duplicate")
    }
}

impl core::error::Error for DuplicateLocation {}

fn main() {}
