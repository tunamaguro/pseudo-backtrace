use pseudo_backtrace::{StackError, StackErrorExt};

#[derive(Debug)]
pub struct ErrorA(());

impl core::fmt::Display for ErrorA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "ErrorA".fmt(f)
    }
}

impl core::error::Error for ErrorA {}

#[derive(Debug, StackError)]
pub struct ErrorB {
    #[stack_error(end)]
    source: ErrorA,
    location: &'static core::panic::Location<'static>,
}

impl core::fmt::Display for ErrorB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "ErrorB".fmt(f)
    }
}

impl core::error::Error for ErrorB {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl From<ErrorA> for ErrorB {
    #[track_caller]
    fn from(value: ErrorA) -> Self {
        ErrorB {
            source: value,
            location: core::panic::Location::caller(),
        }
    }
}

#[derive(Debug, StackError)]
pub struct ErrorC {
    source: ErrorB,
    location: &'static core::panic::Location<'static>,
}

impl core::fmt::Display for ErrorC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "ErrorC".fmt(f)
    }
}

impl core::error::Error for ErrorC {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl From<ErrorB> for ErrorC {
    #[track_caller]
    fn from(value: ErrorB) -> Self {
        ErrorC {
            source: value,
            location: core::panic::Location::caller(),
        }
    }
}

fn main() {
    let a = ErrorA(());
    let b = ErrorB::from(a);
    let c = ErrorC::from(b);

    println!("{}", c.to_chain())
}
