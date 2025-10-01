use pseudo_backtrace::StackError;

#[derive(Debug, StackError)]
struct MultiStd {
    #[stack_error(std)]
    first: std::io::Error,
    #[stack_error(std)]
    second: std::io::Error,
    #[location]
    location: &'static core::panic::Location<'static>,
}

impl core::fmt::Display for MultiStd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        "multi".fmt(f)
    }
}
impl core::error::Error for MultiStd {}

fn main() {}
