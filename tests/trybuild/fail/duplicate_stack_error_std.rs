use pseudo_backtrace::StackError;

#[derive(Debug, StackError)]
struct DupStdAttr {
    #[stack_error(std)]
    #[stack_error(std)]
    source: std::io::Error,
    #[location]
    location: &'static core::panic::Location<'static>,
}

impl core::fmt::Display for DupStdAttr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        "dup".fmt(f)
    }
}
impl core::error::Error for DupStdAttr {}

fn main() {}
