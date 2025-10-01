use pseudo_backtrace::{LocatedError, StackError};

#[derive(Debug, StackError)]
enum A {
    TupleTransparent(LocatedError<std::io::Error>),
    TupleAttrbute(
        #[location]
        #[source]
        LocatedError<std::io::Error>,
    ),
    StructTransparent {
        source: LocatedError<std::io::Error>,
    },
    StructAttribute {
        #[location]
        #[source]
        foo: LocatedError<std::io::Error>,
    },
}

impl core::fmt::Display for A {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "A")
    }
}

impl core::error::Error for A {}

#[derive(Debug, StackError)]
struct B(
    #[location]
    #[source]
    LocatedError<std::io::Error>,
    &'static str,
);

impl core::fmt::Display for B {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "B")
    }
}

impl core::error::Error for B {}

fn main() {}
