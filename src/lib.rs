pub mod de;
pub mod error;
pub mod ser;

#[cfg(test)]
mod tests;

pub use crate::de::Deserializer;
pub use crate::error::Error;
pub use crate::ser::Serializer;

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

pub fn to_writer<V, W>(v: V, w: W) -> crate::error::Result<()>
where
    V: ::serde::Serialize,
    W: ::std::io::Write,
{
    let mut ser = crate::ser::Serializer::new(w);
    v.serialize(&mut ser)
}

pub fn to_bytes<V>(v: V) -> crate::error::Result<Vec<u8>>
where
    V: ::serde::Serialize,
{
    let mut buf = Vec::<u8>::new();
    to_writer(v, &mut buf)?;
    Ok(buf)
}

pub fn from_reader<'de, V, R>(r: R) -> crate::error::Result<V>
where
    V: ::serde::de::DeserializeOwned,
    R: ::std::io::Read,
{
    V::deserialize(&mut crate::de::Deserializer::new(r))
}

pub fn from_bytes<'de, V>(buf: &'de [u8]) -> crate::error::Result<V>
where
    V: ::serde::Deserialize<'de>,
{
    V::deserialize(&mut crate::de::Deserializer::new(buf))
}
