# pseudo-backtrace

![Crates.io Version](https://img.shields.io/crates/v/pseudo-backtrace)
![docs.rs](https://img.shields.io/docsrs/pseudo-backtrace)
![Crates.io License](https://img.shields.io/crates/l/pseudo-backtrace)


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
    // will be printed to standard output as follows:
    // 0: ErrorC, at examples/simple.rs:74:13
    // 1: ErrorB, at examples/simple.rs:73:13
    // 2: ErrorA
}
```

## Using `#[derive(StackError)]`
Deriving requires three kinds of fields:

- `#[location]` for an `&'static core::panic::Location<'static>` captured with `#[track_caller]`.
- `#[source]` for the next layer that also implements `StackError`.
- `#[stack_error(end)]` for a terminal source that only implements `std::error::Error`.

The macro infers `#[location]` and `#[source]` from the field names `location` and `source` when attributes are omitted. Unit structs and variants are rejected because they cannot point to a location.

Since the macro only implements `StackError`, the [Error] implementation must be provided by the user.
