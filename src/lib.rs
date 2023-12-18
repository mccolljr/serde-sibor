#![forbid(unsafe_code)]
//! # `serde` implementation for the SIBOR binary format.
//!
//! SIBOR is a binary format that is designed to be simple to implement, fast to encode and decode,
//! and relatively compact. In order to achieve these goals, the number of features is kept to a
//! minimum, and some types are not supported:
//!
//! - SIBOR is not self-describing. The schema must be known in advance.
//! - SIBOR does not have a concept of "optional" fields. All fields must have a value.
//! - SIBOR does not support maps. All maps must be encoded as sequences of key-value pairs.
//! - SIBOR treats all signed integers, unsigned integers, and floats as 64-bit values.
//! - SIBOR encodes all unsigned integers using a variable-length encoding.
//! - SIBOR encodes all signed integers using a variable-length zigzag encoding.
//! - SIBOR encodes all floats using a 64-bit IEEE 754 encoding. The bits are treated as a u64 and encoded using the variable-length encoding.
//! - SIBOR does not check the length of strings or sequences before decoding them. This means that a maliciously-crafted SIBOR document could cause a denial-of-service attack.
//!
//! SIBOR is meant to be used when you want a quick and dirty way to serialize and deserialize binary data of a known schema.
//! It does not have any built-in support for schema evolution, so such support must be implemented by the user.

/// Deserialization types and functions.
pub mod de;
/// Error types and functions.
pub mod error;
/// Serialization types and functions.
pub mod ser;

/// Tests for the crate.
#[cfg(test)]
mod tests;

pub use crate::de::Deserializer;
pub use crate::error::Error;
pub use crate::ser::Serializer;

/// Get the number of bytes required to encode a value.
pub fn encoded_size<V>(v: V) -> crate::error::Result<usize>
where
    V: ::serde::Serialize,
{
    struct SizeWriter {
        written: usize,
    }

    impl ::std::io::Write for SizeWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.written += buf.len();
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let mut size_writer = SizeWriter { written: 0 };
    let mut ser = crate::ser::Serializer::new(&mut size_writer);
    v.serialize(&mut ser)?;
    Ok(size_writer.written)
}

/// Encode a value into a writer.
pub fn to_writer<V, W>(v: V, w: W) -> crate::error::Result<()>
where
    V: ::serde::Serialize,
    W: ::std::io::Write,
{
    let mut ser = crate::ser::Serializer::new(w);
    v.serialize(&mut ser)
}

/// Encode a value into a byte vector.
pub fn to_bytes<V>(v: V) -> crate::error::Result<Vec<u8>>
where
    V: ::serde::Serialize,
{
    let mut buf = Vec::<u8>::new();
    to_writer(v, &mut buf)?;
    Ok(buf)
}

/// Decode a value from a reader.
pub fn from_reader<'de, V, R>(r: R) -> crate::error::Result<V>
where
    V: ::serde::de::DeserializeOwned,
    R: ::std::io::Read,
{
    V::deserialize(&mut crate::de::Deserializer::new(r))
}

/// Decode a value from a byte slice.
pub fn from_bytes<'de, V>(buf: &'de [u8]) -> crate::error::Result<V>
where
    V: ::serde::Deserialize<'de>,
{
    V::deserialize(&mut crate::de::Deserializer::new(buf))
}
