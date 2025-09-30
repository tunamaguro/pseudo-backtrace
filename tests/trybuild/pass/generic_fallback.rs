use pseudo_backtrace::StackError;
use pseudo_backtrace::StackError as StackErrorTrait;

#[track_caller]
fn location() -> &'static core::panic::Location<'static> {
    core::panic::Location::caller()
}

#[derive(Debug, StackError)]
pub struct LeafError {
    #[stack_error(end)]
    source: std::io::Error,
    location: &'static core::panic::Location<'static>,
}

impl LeafError {
    pub fn new() -> Self {
        Self {
            source: std::io::Error::from_raw_os_error(2),
            location: location(),
        }
    }
}

impl core::fmt::Display for LeafError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "leaf")
    }
}

impl core::error::Error for LeafError {}

#[derive(Debug, StackError)]
pub struct GenericWrapper<T> {
    source: T,
    location: &'static core::panic::Location<'static>,
}

impl<T> GenericWrapper<T>
where
    T: StackErrorTrait,
{
    pub fn new(source: T) -> Self {
        Self {
            source,
            location: location(),
        }
    }
}

impl<T> core::fmt::Display for GenericWrapper<T>
where
    T: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "generic")
    }
}

impl<T> core::error::Error for GenericWrapper<T>
where
    T: StackErrorTrait + core::fmt::Display + core::fmt::Debug,
{
}

#[derive(Debug, StackError)]
pub enum MixedEnum {
    NoSource {
        location: &'static core::panic::Location<'static>,
    },
    WithSource {
        #[source]
        inner: GenericWrapper<LeafError>,
        location: &'static core::panic::Location<'static>,
    },
}

impl MixedEnum {
    pub fn new_with_source(inner: GenericWrapper<LeafError>) -> Self {
        MixedEnum::WithSource {
            inner,
            location: location(),
        }
    }

    pub fn no_source() -> Self {
        MixedEnum::NoSource {
            location: location(),
        }
    }
}

impl core::fmt::Display for MixedEnum {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MixedEnum::NoSource { .. } => write!(f, "no-source"),
            MixedEnum::WithSource { .. } => write!(f, "with-source"),
        }
    }
}

impl core::error::Error for MixedEnum {}

fn assert_stack_error<T: StackErrorTrait>() {}

pub fn smoke() {
    let leaf = LeafError::new();
    let wrapper = GenericWrapper::new(leaf);
    let with_source = MixedEnum::new_with_source(wrapper);
    let _no_source = MixedEnum::no_source();
    assert_stack_error::<GenericWrapper<LeafError>>();
    assert_stack_error::<MixedEnum>();
    let _ = with_source;
}

fn main() {}
