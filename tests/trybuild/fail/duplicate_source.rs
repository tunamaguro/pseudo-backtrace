use pseudo_backtrace::StackError;

#[track_caller]
fn location() -> &'static core::panic::Location<'static> {
    core::panic::Location::caller()
}

#[derive(Debug, StackError)]
struct DuplicateSource {
    #[source]
    first: std::io::Error,
    #[source]
    second: std::io::Error,
    #[location]
    location: &'static core::panic::Location<'static>,
}

impl DuplicateSource {
    pub fn new() -> Self {
        Self {
            first: std::io::Error::from_raw_os_error(5),
            second: std::io::Error::from_raw_os_error(6),
            location: location(),
        }
    }
}

impl core::fmt::Display for DuplicateSource {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "dup-source")
    }
}

impl core::error::Error for DuplicateSource {}

fn main() {}
