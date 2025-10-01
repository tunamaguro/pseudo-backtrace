# pseudo-backtrace

[![Crates.io Version](https://img.shields.io/crates/v/pseudo-backtrace)](https://crates.io/crates/pseudo-backtrace)
[![docs.rs](https://img.shields.io/docsrs/pseudo-backtrace)](https://docs.rs/pseudo-backtrace/latest/pseudo_backtrace/)
[![Crates.io License](https://img.shields.io/crates/l/pseudo-backtrace)](https://github.com/tunamaguro/pseudo-backtrace/blob/main/LICENSE-MIT)


This is a library that makes it easy to create error types that can track error propagation history.

## Example

```rust
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
    #[stack_error(std)]
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


# fn main() {
    let a = ErrorA(());
    let b = ErrorB::from(a);
    let c = ErrorC::from(b);

    println!("{}", c.to_chain())
    // will be printed to standard output as follows:
    // 0: ErrorC, at examples/simple.rs:74:13
    // 1: ErrorB, at examples/simple.rs:73:13
    // 2: ErrorA
# }
```

## Using `#[derive(StackError)]`

Deriving `StackError` requires two types of fields:

1. **Required Field**:
   - A field holding a `&'static core::panic::Location<'static>`. This is mandatory.
   - The field can be named `location` or marked with the `#[location]` attribute.

2. **Optional Field**:
   - A field representing the next error in the stack trace. This field is optional.
   - It can be marked with either:
     - `#[stack_error(std)]`: Treats the next error as a type implementing `core::error::Error`.
     - `#[stack_error(stacked)]`: Treats the next error as a type implementing `StackError`.
     - `#[source]` or a field named `source`: Defaults to `#[stack_error(stacked)]`.

Note that the macro only implements `StackError`, so users must manually implement `core::error::Error`.

### Using `LocatedError` as both `location` and `source`

You can embed a `LocatedError<T>` field and use it for both the `location` and the `source` in one of the following ways:

- Explicitly mark the same field with both attributes:

```rust
#[derive(Debug, pseudo_backtrace::StackError)]
pub struct WrappedIo {
    #[location]
    #[source] // or `#[stack_error(stacked)]` / `#[stack_error(std)]`
    inner: pseudo_backtrace::LocatedError<std::io::Error>,
}

impl core::fmt::Display for WrappedIo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "wrapped-io")
    }
}

impl core::error::Error for WrappedIo {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}
```

- Or, if you omit an explicit `#[location]`, the derive will automatically use the `#[source]` field when its type is `LocatedError<_>` as the location provider:

```rust
#[derive(Debug, pseudo_backtrace::StackError)]
pub struct WrappedIo {
    #[source]
    inner: pseudo_backtrace::LocatedError<std::io::Error>,
}

impl core::fmt::Display for WrappedIo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "wrapped-io")
    }
}

impl core::error::Error for WrappedIo {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}
```

In both cases, the generated `impl StackError` will return `inner.location()` as the location and will chain to `inner` as the next error.
