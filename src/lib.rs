#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub use pseudo_backtrace_derive::StackError;

/// One layer in a stack of chained errors.
#[derive(Debug, Clone)]
pub enum ErrorChain<'a> {
    /// A stacked error
    Stacked(&'a dyn StackError),
    /// A [core::error::Error].
    Std(&'a dyn core::error::Error),
}

impl<'a> ErrorChain<'a> {
    /// Returns lower-level error
    pub fn next(&self) -> Option<ErrorChain<'a>> {
        match self {
            ErrorChain::Stacked(stack_error) => stack_error.next(),
            ErrorChain::Std(error) => error.source().map(ErrorChain::Std),
        }
    }

    /// Into the iterator
    pub const fn into_iter(self) -> Iter<'a> {
        Iter { stack: Some(self) }
    }

    /// Returns the underlying error for this stack layer.
    pub const fn inner(&'a self) -> &'a dyn core::error::Error {
        match self {
            ErrorChain::Stacked(stack_error) => stack_error,
            ErrorChain::Std(error) => error,
        }
    }

    /// Returns the recorded source location when available.
    pub fn location(&self) -> Option<&'static core::panic::Location<'static>> {
        match self {
            ErrorChain::Stacked(stack_error) => Some(stack_error.location()),
            _ => None,
        }
    }
}

impl core::fmt::Display for ErrorChain<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ErrorChain::Stacked(stack_error) => {
                write!(f, "{}, at {}", stack_error, stack_error.location())
            }
            ErrorChain::Std(error) => error.fmt(f),
        }
    }
}

impl core::error::Error for ErrorChain<'_> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        self.inner().source()
    }
}

impl<'a, E> From<&'a E> for ErrorChain<'a>
where
    E: StackError + Sized,
{
    fn from(stack_error: &'a E) -> Self {
        ErrorChain::Stacked(stack_error)
    }
}

/// Error types that can report a stack trace-like chain.
pub trait StackError: core::error::Error {
    /// Returns the source location of this error.
    fn location(&self) -> &'static core::panic::Location<'static>;
    /// Returns the next detail in the stack.
    fn next<'a>(&'a self) -> Option<ErrorChain<'a>>;
    /// Creates an iterator over this error's stack details.
    fn iter<'a>(&'a self) -> Iter<'a>
    where
        Self: Sized,
    {
        Iter::new(self)
    }
}

/// Iterator over individual error stack entries.
#[derive(Debug, Clone)]
pub struct Iter<'a> {
    stack: Option<ErrorChain<'a>>,
}

impl<'a> Iter<'a> {
    const fn new<E>(source: &'a E) -> Self
    where
        E: StackError,
    {
        Iter {
            stack: Some(ErrorChain::Stacked(source)),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = ErrorChain<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.stack.take() {
            Some(detail) => {
                self.stack = detail.next();
                Some(detail)
            }
            None => None,
        }
    }
}

/// Wrapper that records the call-site for any `core::error::Error` and exposes it as a [StackError].
///
/// This is useful when you already have an error type that implements [core::error::Error] but cannot be modified to derive [StackError].
///
/// # Examples
/// ```
/// # extern crate std;
/// use pseudo_backtrace::{LocatedError, StackError};
///
/// fn assert_stack_error<T:StackError>(){}
///     
/// assert_stack_error::<LocatedError<std::io::Error>>();
/// ```
#[derive(Debug)]
pub struct LocatedError<E> {
    source: E,
    location: &'static core::panic::Location<'static>,
}

impl<E> LocatedError<E> {
    /// Returns the inner value
    pub fn into_inner(self) -> E {
        self.source
    }
}

impl<E> core::fmt::Display for LocatedError<E>
where
    E: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.source.fmt(f)
    }
}

impl<E> core::error::Error for LocatedError<E>
where
    E: core::error::Error,
{
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        self.source.source()
    }
}

impl<E> StackError for LocatedError<E>
where
    E: core::error::Error,
{
    fn location(&self) -> &'static core::panic::Location<'static> {
        self.location
    }

    fn next<'a>(&'a self) -> Option<ErrorChain<'a>> {
        self.source.source().map(ErrorChain::Std)
    }
}

impl<E> From<E> for LocatedError<E> {
    #[track_caller]
    fn from(value: E) -> Self {
        LocatedError {
            source: value,
            location: core::panic::Location::caller(),
        }
    }
}

/// Helper for display [ErrorChain]
#[derive(Debug, Clone)]
pub struct ChainWriter<'a> {
    std_limit: usize,
    stack: ErrorChain<'a>,
}

impl<'a> core::fmt::Display for ChainWriter<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let it = self.stack.clone().into_iter();
        let std_count = 0;
        for (i, err) in it
            .enumerate()
            .take_while(move |_| std_count < self.std_limit)
        {
            write!(f, "{}: {}", i, err)?;
        }

        Ok(())
    }
}

/// Convenience helpers for types implementing [StackError].
pub trait StackErrorExt: StackError + Sized {
    /// Returns a [ChainWriter] that walks this error stack from the top and prints a single trailing non- [StackError] source when formatting.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// let err = StackErrorC::new();
    /// println!("{}", err.to_chain());
    /// // 0: StackError A, at src/main.rs:20:5
    /// // 1: StackError B, at src/main.rs:19:5
    /// // 2: StackError C, at src/main.rs:18:5  
    /// // 3: StdError A
    /// ```
    fn to_chain<'a>(&'a self) -> ChainWriter<'a> {
        self.to_chain_with_limit(1)
    }

    /// Returns a [ChainWriter] capped to `limit` trailing [core::error::Error] entries that do not implement [StackError].
    ///
    /// ## Example
    ///
    /// ```ignore
    /// let err = StackErrorC::new();
    /// println!("{}", err.to_chain_with_limit(usize::MAX));
    /// // 0: StackError A, at src/main.rs:20:5
    /// // 1: StackError B, at src/main.rs:19:5
    /// // 2: StackError C, at src/main.rs:18:5  
    /// // 3: StdError A
    /// // 4: StdError B
    /// // 5: StdError C
    /// 
    /// println!("{}", err.to_chain_with_limit(0));
    /// // 0: StackError A, at src/main.rs:20:5
    /// // 1: StackError B, at src/main.rs:19:5
    /// // 2: StackError C, at src/main.rs:18:5  
    /// ```
    fn to_chain_with_limit<'a>(&'a self, limit: usize) -> ChainWriter<'a> {
        ChainWriter {
            std_limit: limit,
            stack: ErrorChain::from(self),
        }
    }

    /// Returns the deepest [ErrorChain] in the chain.
    /// ## Example
    ///
    /// ```ignore
    /// 0: StackError A, at src/main.rs:20:5
    /// 1: StackError B, at src/main.rs:19:5
    /// 2: StackError C, at src/main.rs:18:5  
    /// 3: StdError A
    /// 4: StdError B
    /// 5: StdError C <- Return this
    /// ```
    fn last<'a>(&'a self) -> ErrorChain<'a>
    where
        Self: Sized,
    {
        self.iter().last().unwrap_or(ErrorChain::from(self))
    }

    /// Returns the deepest [StackError] in the chain
    ///
    /// ## Example
    ///
    /// ```ignore
    /// 0: StackError A, at src/main.rs:20:5
    /// 1: StackError B, at src/main.rs:19:5
    /// 2: StackError C, at src/main.rs:18:5  <- Return this
    /// 3: StdError A
    /// 4: StdError B
    /// 5: StdError C
    /// ```
    fn last_stacked<'a>(&'a self) -> &'a dyn StackError {
        self.iter()
            .filter_map(|e| match e {
                ErrorChain::Stacked(stack_error) => Some(stack_error),
                _ => None,
            })
            .last()
            .unwrap_or(self)
    }

    /// Returns the first [core::error::Error] in the chain
    ///
    /// ## Example
    ///
    /// ```ignore
    /// 0: StackError A, at src/main.rs:20:5
    /// 1: StackError B, at src/main.rs:19:5
    /// 2: StackError C, at src/main.rs:18:5
    /// 3: StdError A <- Return this
    /// 4: StdError B
    /// 5: StdError C
    /// ```
    fn last_std<'a>(&'a self) -> Option<&'a dyn core::error::Error> {
        self.iter()
            .filter_map(|e| match e {
                ErrorChain::Std(error) => Some(error),
                _ => None,
            })
            .next()
    }
}

impl<E: StackError> StackErrorExt for E {}
