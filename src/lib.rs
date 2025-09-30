#![no_std]

/// One layer in a stack of chained errors.
#[derive(Debug, Clone)]
pub enum ErrorDetail<'a> {
    Stacked(&'a dyn StackError),
    End(&'a dyn core::error::Error),
}

impl<'a> ErrorDetail<'a> {
    /// Returns the underlying error for this stack layer.
    pub fn source(&'a self) -> &'a dyn core::error::Error {
        match self {
            ErrorDetail::Stacked(stack_error) => stack_error,
            ErrorDetail::End(error) => error,
        }
    }

    /// Returns the recorded source location when available.
    pub fn location(&self) -> Option<&'static core::panic::Location<'static>> {
        match self {
            ErrorDetail::Stacked(stack_error) => Some(stack_error.location()),
            _ => None,
        }
    }
}

impl<'a, E> From<&'a E> for ErrorDetail<'a>
where
    E: StackError + Sized,
{
    fn from(stack_error: &'a E) -> Self {
        ErrorDetail::Stacked(stack_error)
    }
}

/// Error types that can report a stack trace-like chain.
pub trait StackError: core::error::Error {
    /// Returns the source location of this error.
    fn location(&self) -> &'static core::panic::Location<'static>;
    /// Returns the next detail in the stack.
    fn next<'a>(&'a self) -> Option<ErrorDetail<'a>>;
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
    source: Option<ErrorDetail<'a>>,
}

impl<'a> Iter<'a> {
    const fn new<E>(source: &'a E) -> Self
    where
        E: StackError,
    {
        Iter {
            source: Some(ErrorDetail::Stacked(source)),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = ErrorDetail<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.source.take() {
            Some(detail) => {
                match &detail {
                    ErrorDetail::Stacked(stack_error) => {
                        self.source = stack_error.next();
                    }
                    _ => {}
                };
                Some(detail)
            }
            None => None,
        }
    }
}

/// Formatter for a single stack layer that remembers its index.
#[derive(Debug, Clone)]
pub struct StackWriter<'a> {
    layer: usize,
    source: ErrorDetail<'a>,
}

impl<'a> StackWriter<'a> {
    /// Returns the zero-based layer index for this entry.
    pub fn layer(&self) -> usize {
        self.layer
    }
    /// Returns the error detail captured for this layer.
    pub fn detail(&'a self) -> ErrorDetail<'a> {
        self.source.clone()
    }
}

impl<'a> core::fmt::Display for StackWriter<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.source {
            ErrorDetail::Stacked(stack_error) => {
                write!(
                    f,
                    "{}: {}, at {}",
                    self.layer,
                    stack_error,
                    stack_error.location()
                )
            }
            ErrorDetail::End(error) => {
                write!(f, "{}: {}", self.layer, error,)
            }
        }
    }
}

/// Iterator adapter that yields formatted stack entries.
#[derive(Debug, Clone)]
pub struct StackChain<'a> {
    layer: usize,
    source: ErrorDetail<'a>,
}

impl<'a> Iterator for StackChain<'a> {
    type Item = StackWriter<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.layer == usize::MAX {
            return None;
        }

        let out = StackWriter {
            layer: self.layer,
            source: self.source.clone(),
        };

        match self.source {
            ErrorDetail::Stacked(stack_error) => {
                if let Some(next) = stack_error.next() {
                    self.source = next;
                    self.layer += 1;
                } else {
                    self.layer = usize::MAX;
                }
            }
            ErrorDetail::End(_) => {
                self.layer = usize::MAX;
            }
        }

        Some(out)
    }
}

impl<'a> core::fmt::Display for StackChain<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for w in self.clone() {
            writeln!(f, "{}", w)?;
        }

        Ok(())
    }
}

/// Convenience helpers for types implementing [`StackError`].
pub trait StackErrorExt: StackError {
    /// Returns a [`StackChain`] that walks this error stack from the top.
    fn to_chain<'a>(&'a self) -> StackChain<'a>
    where
        Self: Sized,
    {
        StackChain {
            layer: 0,
            source: ErrorDetail::from(self),
        }
    }

    /// Returns the deepest [`ErrorDetail`] in the chain.
    fn last<'a>(&'a self) -> ErrorDetail<'a>
    where
        Self: Sized,
    {
        let mut detail = ErrorDetail::from(self);
        while let Some(next) = self.next() {
            detail = next;
        }
        detail
    }
}
