use pseudo_backtrace::StackError;

#[derive(Debug, StackError)]
struct ErrorOpt<'a> {
    source: Option<&'a (dyn core::error::Error + 'static)>,
    location: &'static core::panic::Location<'static>,
}
impl core::fmt::Display for ErrorOpt<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "error with option")
    }
}

impl core::error::Error for ErrorOpt<'_> {}

fn main() {}
