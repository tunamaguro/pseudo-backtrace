use pseudo_backtrace::StackError;

#[derive(Debug, StackError)]
struct BadAttr {
    #[stack_error(foo)]
    source: std::io::Error,
    #[location]
    location: &'static core::panic::Location<'static>,
}

impl core::fmt::Display for BadAttr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        "bad".fmt(f)
    }
}
impl core::error::Error for BadAttr {}

fn main() {}
