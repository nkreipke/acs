/// Boilerplate to use rust-sdl2 with anyhow as it likes to return Result<T, String>.
/// String does not implement Error.

use std::error::Error;
use std::fmt::{Display, Formatter};

pub trait CompatErrorTypes {
    type RealError: Error;

    fn compat_err(self) -> Self::RealError;
}

pub trait CompatErrorResultTypes {
    type Value;
    type RealError: Error;

    fn compat_err(self) -> Result<Self::Value, Self::RealError>;
}

impl CompatErrorTypes for String {
    type RealError = CompatStringError;

    fn compat_err(self) -> Self::RealError {
        CompatStringError(self)
    }
}

impl<V, E: CompatErrorTypes> CompatErrorResultTypes for Result<V, E> {
    type Value = V;
    type RealError = E::RealError;

    fn compat_err(self) -> Result<Self::Value, Self::RealError> {
        self.map_err(|e| e.compat_err())
    }
}

#[derive(Debug)]
pub struct CompatStringError(String);

impl Display for CompatStringError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for CompatStringError {}