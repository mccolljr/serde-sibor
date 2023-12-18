#[derive(Debug, ::thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Generic(String),

    #[error("{0}")]
    Io(::std::io::Error),

    #[error("unsupported: {0}")]
    Unsupported(String),

    #[error("invalid {0}")]
    Invalid(String),
}

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
