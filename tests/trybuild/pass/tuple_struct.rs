use pseudo_backtrace::StackError;

#[derive(Debug, StackError)]
struct Tuple(
    &'static str,
    #[location] &'static core::panic::Location<'static>,
);

impl core::fmt::Display for Tuple {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl core::error::Error for Tuple {}

fn main() {}
