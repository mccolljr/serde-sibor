/// Error type for this crate, shared by the serializer and deserializer.
#[derive(Debug, ::thiserror::Error)]
pub enum Error {
    /// Errors that don't fit into any other category.
    #[error("{0}")]
    Generic(String),

    /// Errors related to IO operations
    #[error("{0}")]
    Io(::std::io::Error),

    /// Errors related to usage of unsupported types.
    #[error("unsupported: {0}")]
    Unsupported(String),

    /// Errors related to values that are not valid for the given type.
    #[error("invalid {0}")]
    Invalid(String),
}

/// Result type for this crate.
pub type Result<T> = ::std::result::Result<T, Error>;

impl ::serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Generic(msg.to_string())
    }
}

impl ::serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Generic(msg.to_string())
    }
}
