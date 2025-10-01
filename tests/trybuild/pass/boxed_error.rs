use pseudo_backtrace::StackError;

#[derive(Debug, StackError)]
struct BoxError {
    #[stack_error(end)]
    source: Box<dyn core::error::Error>,
    location: &'static core::panic::Location<'static>,
}
impl core::fmt::Display for BoxError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "error with option")
    }
}

impl core::error::Error for BoxError {}

fn main() {}
