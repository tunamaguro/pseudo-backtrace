#![no_std]

#[derive(Debug, Clone)]
pub enum ErrorDetail<'a> {
    Stacked(&'a dyn StackError),
    End(&'a dyn core::error::Error),
}

impl<'a> ErrorDetail<'a> {
    pub fn source(&'a self) -> &'a dyn core::error::Error {
        match self {
            ErrorDetail::Stacked(stack_error) => stack_error,
            ErrorDetail::End(error) => error,
        }
    }

    pub fn location(&self) -> Option<&'static core::panic::Location<'static>> {
        match self {
            ErrorDetail::Stacked(stack_error) => Some(stack_error.location()),
            _ => None,
        }
    }
}

pub trait StackError: core::error::Error {
    /// Returns the source location of this error
    fn location(&self) -> &'static core::panic::Location<'static>;
    /// Return next level error
    fn next<'a>(&'a self) -> Option<ErrorDetail<'a>>;
    fn to_detail<'a>(&'a self) -> ErrorDetail<'a>
    where
        Self: Sized,
    {
        ErrorDetail::Stacked(self)
    }
    fn iter<'a>(&'a self) -> Iter<'a>
    where
        Self: Sized,
    {
        Iter::new(self)
    }
}

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

#[derive(Debug, Clone)]
pub struct StackWriter<'a> {
    layer: usize,
    source: ErrorDetail<'a>,
}

impl<'a> StackWriter<'a> {
    pub fn layer(&self) -> usize {
        self.layer
    }
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

pub trait StackErrorExt: StackError {
    fn to_chain<'a>(&'a self) -> StackChain<'a>
    where
        Self: Sized,
    {
        StackChain {
            layer: 0,
            source: self.to_detail(),
        }
    }

    fn last<'a>(&'a self) -> ErrorDetail<'a>
    where
        Self: Sized,
    {
        let mut detail = self.to_detail();
        while let Some(next) = self.next() {
            detail = next;
        }
        detail
    }
}
