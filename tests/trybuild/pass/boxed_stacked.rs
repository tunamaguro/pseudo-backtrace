use pseudo_backtrace::StackError;

#[track_caller]
fn location() -> &'static core::panic::Location<'static> {
    core::panic::Location::caller()
}

#[derive(Debug, StackError)]
struct Inner {
    #[stack_error(std)]
    source: std::io::Error,
    #[location]
    location: &'static core::panic::Location<'static>,
}
impl core::fmt::Display for Inner {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "inner")
    }
}
impl core::error::Error for Inner {}

#[derive(Debug, StackError)]
struct Outer {
    #[source]
    inner: std::boxed::Box<Inner>,
    #[location]
    location: &'static core::panic::Location<'static>,
}
impl core::fmt::Display for Outer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "outer")
    }
}
impl core::error::Error for Outer {}

fn main() {
    let _ = Outer {
        inner: std::boxed::Box::new(Inner {
            source: std::io::Error::other("oh"),
            location: location(),
        }),
        location: location(),
    };
}
