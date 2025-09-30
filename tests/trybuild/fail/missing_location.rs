use pseudo_backtrace::StackError;

#[derive(Debug, StackError)]
struct MissingLocation {
    #[stack_error(end)]
    source: std::io::Error,
}

impl core::fmt::Display for MissingLocation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "missing")
    }
}

impl core::error::Error for MissingLocation {}

fn main() {}
