use pseudo_backtrace::{LocatedError, StackError, StackErrorExt};
use thiserror::Error;

#[derive(Debug)]
struct Leaf;

impl core::fmt::Display for Leaf {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("leaf")
    }
}

impl core::error::Error for Leaf {}

// Pattern 1: explicit #[location] + #[source] on the same LocatedError field
#[derive(Debug, StackError)]
struct ExplictBoth {
    #[location]
    #[source] // treated as stacked by default
    inner: LocatedError<Leaf>,
}

impl core::fmt::Display for ExplictBoth {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("explict-both")
    }
}

impl core::error::Error for ExplictBoth {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.inner)
    }
}

impl From<LocatedError<Leaf>> for ExplictBoth {
    fn from(inner: LocatedError<Leaf>) -> Self {
        Self { inner }
    }
}

// Pattern 2: implicit location from source when the source is LocatedError<_>
#[derive(Debug, StackError)]
struct ImplicitFromSource {
    #[source]
    inner: LocatedError<Leaf>,
}

impl core::fmt::Display for ImplicitFromSource {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("implicit-from-source")
    }
}

impl core::error::Error for ImplicitFromSource {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.inner)
    }
}

fn main() {
    let leaf = Leaf;
    let located: LocatedError<Leaf> = LocatedError::from(leaf);
    let a = ExplictBoth::from(located);
    println!("{}", a.to_chain());

    let located2: LocatedError<Leaf> = LocatedError::from(Leaf);
    let b = ImplicitFromSource { inner: located2 };
    println!("{}", b.to_chain());

    // Enum pattern: put only LocatedError inside a variant (works nicely with thiserror's transparent)
    #[derive(Debug, Error, StackError)]
    enum TransparentEnum {
        // thiserror transparent variant: uses inner Display and Error::source
        // pseudo-backtrace: uses inner.location() for StackError::location via fallback
        #[error(transparent)]
        IoTransparent(LocatedError<Leaf>),

        // non-transparent but still only LocatedError inside: add #[source] to link as next
        #[error("io error")]
        IoWithSource(#[source] LocatedError<Leaf>),
    }

    let e1 = TransparentEnum::IoTransparent(LocatedError::from(Leaf));
    println!("{}", e1.to_chain());

    let e2 = TransparentEnum::IoWithSource(LocatedError::from(Leaf));
    println!("{}", e2.to_chain());
}
