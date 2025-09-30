use core::panic::Location;
use pseudo_backtrace::{StackError, StackErrorExt};
use std::fmt;

#[derive(Debug, StackError)]
struct LeafError {
    #[stack_error(end)]
    cause: std::io::Error,
    #[location]
    location: &'static Location<'static>,
}

impl LeafError {
    #[track_caller]
    fn new() -> Self {
        Self {
            cause: std::io::Error::new(std::io::ErrorKind::Other, "leaf exploded"),
            location: Location::caller(),
        }
    }
}

impl fmt::Display for LeafError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("leaf layer failed")
    }
}

impl std::error::Error for LeafError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}

#[derive(Debug, StackError)]
struct MiddleError {
    #[source]
    source: LeafError,
    #[location]
    location: &'static Location<'static>,
}

impl MiddleError {
    #[track_caller]
    fn new() -> Self {
        Self {
            source: LeafError::new(),
            location: Location::caller(),
        }
    }
}

impl fmt::Display for MiddleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("middle layer failed")
    }
}

impl std::error::Error for MiddleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Debug, StackError)]
struct TopError {
    #[source]
    source: MiddleError,
    #[location]
    location: &'static Location<'static>,
}

impl TopError {
    #[track_caller]
    fn new() -> Self {
        Self {
            source: MiddleError::new(),
            location: Location::caller(),
        }
    }
}

impl fmt::Display for TopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("top layer failed")
    }
}

impl std::error::Error for TopError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

fn main() {
    let error = TopError::new();

    println!("Display: {error}");
    println!("pseudo-backtrace frames:");
    println!("{}", error.to_chain());

    println!("std::error::Error chain:");
    let mut current: Option<&(dyn std::error::Error + 'static)> = Some(&error);
    let mut depth = 0;
    while let Some(err) = current.take() {
        println!("  {depth}: {err}");
        depth += 1;
        current = err.source();
    }
}
