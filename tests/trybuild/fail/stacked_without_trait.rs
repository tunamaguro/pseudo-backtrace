use pseudo_backtrace::StackError;

#[derive(Debug)]
struct NotStacked(std::io::Error);
impl core::fmt::Display for NotStacked {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}
impl core::error::Error for NotStacked {}

#[derive(Debug, StackError)]
struct BadOuter {
    #[stack_error(stacked)]
    source: NotStacked,
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
