use pseudo_error_derive::StackError;

#[derive(Debug, StackError)]
enum UnitVariant {
    First,
}

impl core::fmt::Display for UnitVariant {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "unit")
    }
}

impl core::error::Error for UnitVariant {}

fn main() {}
