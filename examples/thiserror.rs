//! exmaple with `thiserror`
//!
//! For details on the `core::panic::Location` support in the `thiserror` crate, see:
//! https://github.com/dtolnay/thiserror/pull/291

use core::panic::Location;
use pseudo_backtrace::{StackError, StackErrorExt};
use thiserror::Error;

#[derive(Debug, Error, StackError)]
#[error("leaf layer: {message}")]
struct LeafError {
    message: &'static str,
    #[stack_error(std)]
    source: std::io::Error,
    location: &'static Location<'static>,
}

impl LeafError {
    #[track_caller]
    fn new(message: &'static str) -> Self {
        Self {
            message,
            source: std::io::Error::other("device failure"),
            location: Location::caller(),
        }
    }
}

#[derive(Debug, Error, StackError)]
#[error("mid layer: {context}")]
struct MidError {
    context: &'static str,
    source: LeafError,
    location: &'static Location<'static>,
}

impl MidError {
    #[track_caller]
    fn new(context: &'static str) -> Self {
        Self {
            context,
            source: LeafError::new("leaf malfunction"),
            location: Location::caller(),
        }
    }
}

#[derive(Debug, Error, StackError)]
#[error("top layer: {operation}")]
struct TopError {
    operation: &'static str,
    source: MidError,
    location: &'static Location<'static>,
}

impl TopError {
    #[track_caller]
    fn new(operation: &'static str) -> Self {
        Self {
            operation,
            source: MidError::new("mid initialization"),
            location: Location::caller(),
        }
    }
}

fn main() {
    let error = TopError::new("startup");

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
