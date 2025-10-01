use core::panic::UnwindSafe;

use crate::StackError;

#[doc(hidden)]
pub trait AsDynStdError<'a>: SealedStd {
    fn as_dyn_std_error(&self) -> &(dyn core::error::Error + 'a);
}

impl<'a, T: core::error::Error + 'a> AsDynStdError<'a> for T {
    #[inline]
    fn as_dyn_std_error(&self) -> &(dyn core::error::Error + 'a) {
        self
    }
}

impl<'a> AsDynStdError<'a> for dyn core::error::Error + 'a {
    #[inline]
    fn as_dyn_std_error(&self) -> &(dyn core::error::Error + 'a) {
        self
    }
}

impl<'a> AsDynStdError<'a> for dyn core::error::Error + Send + 'a {
    #[inline]
    fn as_dyn_std_error(&self) -> &(dyn core::error::Error + 'a) {
        self
    }
}

impl<'a> AsDynStdError<'a> for dyn core::error::Error + Send + Sync + 'a {
    #[inline]
    fn as_dyn_std_error(&self) -> &(dyn core::error::Error + 'a) {
        self
    }
}

impl<'a> AsDynStdError<'a> for dyn core::error::Error + Send + Sync + UnwindSafe + 'a {
    #[inline]
    fn as_dyn_std_error(&self) -> &(dyn core::error::Error + 'a) {
        self
    }
}

#[doc(hidden)]
pub trait AsDynStackError<'a>: SealedStack {
    fn as_dyn_stack_error(&self) -> &(dyn StackError + 'a);
}

impl<'a, T: StackError + 'a> AsDynStackError<'a> for T {
    #[inline]
    fn as_dyn_stack_error(&self) -> &(dyn StackError + 'a) {
        self
    }
}

impl<'a> AsDynStackError<'a> for dyn StackError + 'a {
    #[inline]
    fn as_dyn_stack_error(&self) -> &(dyn StackError + 'a) {
        self
    }
}

impl<'a> AsDynStackError<'a> for dyn StackError + Send + 'a {
    #[inline]
    fn as_dyn_stack_error(&self) -> &(dyn StackError + 'a) {
        self
    }
}

impl<'a> AsDynStackError<'a> for dyn StackError + Send + Sync + 'a {
    #[inline]
    fn as_dyn_stack_error(&self) -> &(dyn StackError + 'a) {
        self
    }
}

impl<'a> AsDynStackError<'a> for dyn StackError + Send + Sync + UnwindSafe + 'a {
    #[inline]
    fn as_dyn_stack_error(&self) -> &(dyn StackError + 'a) {
        self
    }
}

#[doc(hidden)]
pub trait SealedStd {}
impl<T: core::error::Error> SealedStd for T {}
impl SealedStd for dyn core::error::Error + '_ {}
impl SealedStd for dyn core::error::Error + Send + '_ {}
impl SealedStd for dyn core::error::Error + Send + Sync + '_ {}
impl SealedStd for dyn core::error::Error + Send + Sync + UnwindSafe + '_ {}

#[doc(hidden)]
pub trait SealedStack {}
impl<T: StackError> SealedStack for T {}
impl SealedStack for dyn StackError + '_ {}
impl SealedStack for dyn StackError + Send + '_ {}
impl SealedStack for dyn StackError + Send + Sync + '_ {}
impl SealedStack for dyn StackError + Send + Sync + UnwindSafe + '_ {}
