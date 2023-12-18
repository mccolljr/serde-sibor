use crate::error::{Error, Result};

struct DeserializeTuple<'a, R> {
    de: &'a mut Deserializer<R>,
}

impl<'de, 'a, R> ::serde::de::SeqAccess<'de> for DeserializeTuple<'a, R>
where
    R: ::std::io::Read,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let x = seed.deserialize(&mut *self.de)?;
        Ok(Some(x))
    }
}

struct DeserializeEnum<'a, R> {
    de: &'a mut Deserializer<R>,
}

impl<'de, 'a, R> ::serde::de::EnumAccess<'de> for DeserializeEnum<'a, R>
where
    R: ::std::io::Read,
{
    type Error = Error;

    type Variant = DeserializeEnumVariant<'a, R>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(&mut *self.de)?;
        Ok((variant, DeserializeEnumVariant { de: &mut *self.de }))
    }
}

struct DeserializeEnumVariant<'a, R> {
    de: &'a mut Deserializer<R>,
}

impl<'de, 'a, R> ::serde::de::VariantAccess<'de> for DeserializeEnumVariant<'a, R>
where
    R: ::std::io::Read,
{
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn tuple_variant<V>(self, _: usize, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        v.visit_seq(DeserializeTuple { de: &mut *self.de })
    }

    fn struct_variant<V>(self, _: &'static [&'static str], v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        v.visit_seq(DeserializeTuple { de: &mut *self.de })
    }
}

struct DeserializeCollection<'a, R> {
    remaining: usize,
    de: &'a mut Deserializer<R>,
}

impl<'de, 'a, R> ::serde::de::SeqAccess<'de> for DeserializeCollection<'a, R>
where
    R: ::std::io::Read,
{
    type Error = Error;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.remaining < 1 {
            return Ok(None);
        }
        self.remaining -= 1;

        let x = seed.deserialize(&mut *self.de)?;
        Ok(Some(x))
    }
}

pub struct Deserializer<R> {
    reader: R,
}

impl<R> Deserializer<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R> Deserializer<R>
where
    R: ::std::io::Read,
{
    const MAX_VARINT_BYTES: u64 = 10;

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.reader.read_exact(&mut buf[..]).map_err(Error::Io)
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8];
        self.read_exact(&mut buf[..])?;
        Ok(buf[0])
    }

    pub fn read_uvarint(&mut self) -> Result<u64> {
        let mut v = 0u64;
        for i in 0u64..Self::MAX_VARINT_BYTES {
            let b = self.read_u8()? as u64;
            v |= (b & 0x7f) << (i * 7);
            if b < 0x80 {
                return Ok(v);
            }
        }

        Err(Error::Invalid("variable integer encoding".into()))
    }

    pub fn read_ivarint(&mut self) -> Result<i64> {
        let unsigned = self.read_uvarint()?;
        let mut signed = (unsigned >> 1) as i64;
        if (unsigned & 0x1) > 0 {
            signed = !signed;
        }
        Ok(signed)
    }

    pub fn read_float(&mut self) -> Result<f64> {
        let unsigned = self.read_uvarint()?;
        Ok(f64::from_bits(unsigned))
    }

    pub fn read_bool(&mut self) -> Result<bool> {
        let b = self.read_u8()?;
        match b {
            1 => Ok(true),
            0 => Ok(false),
            _ => Err(Error::Invalid("boolean encoding".into())),
        }
    }

    pub fn read_bytes(&mut self, min: usize, max: usize) -> Result<Vec<u8>> {
        let len64 = self.read_uvarint()?;
        let len = usize::try_from(len64).map_err(|e| Error::Generic(e.to_string()))?;
        if len < min || len > max {
            return Err(Error::Invalid(format!("length: {len}")));
        }
        let mut raw = vec![0u8; len];
        self.read_exact(&mut raw[..])?;
        Ok(raw)
    }

    pub fn read_string(&mut self, min: usize, max: usize) -> Result<String> {
        let raw = self.read_bytes(min, max)?;
        String::from_utf8(raw).map_err(|e| Error::Generic(e.to_string()))
    }
}

impl<'de, 'a, R> ::serde::Deserializer<'de> for &'a mut Deserializer<R>
where
    R: ::std::io::Read,
{
    type Error = Error;

    fn deserialize_any<V>(self, _: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::Unsupported("deserialize any".into()))
    }

    fn deserialize_bool<V>(self, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let x = self.read_bool()?;
        v.visit_bool(x)
    }

    fn deserialize_i8<V>(self, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(v)
    }

    fn deserialize_i16<V>(self, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(v)
    }

    fn deserialize_i32<V>(self, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(v)
    }

    fn deserialize_i64<V>(self, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let x = self.read_ivarint()?;
        v.visit_i64(x)
    }

    fn deserialize_u8<V>(self, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_u64(v)
    }

    fn deserialize_u16<V>(self, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_u64(v)
    }

    fn deserialize_u32<V>(self, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_u64(v)
    }

    fn deserialize_u64<V>(self, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let x = self.read_uvarint()?;
        v.visit_u64(x)
    }

    fn deserialize_f32<V>(self, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_f64(v)
    }

    fn deserialize_f64<V>(self, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let x = self.read_float()?;
        v.visit_f64(x)
    }

    fn deserialize_char<V>(self, _: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::Unsupported("deserialize char".into()))
    }

    fn deserialize_str<V>(self, _: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::Unsupported("deserialize &str".into()))
    }

    fn deserialize_string<V>(self, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let x = self.read_string(0, usize::MAX)?;
        v.visit_string(x)
    }

    fn deserialize_bytes<V>(self, _: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::Unsupported("deserialize &[u8]".into()))
    }

    fn deserialize_byte_buf<V>(self, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let x = self.read_bytes(0, usize::MAX)?;
        v.visit_byte_buf(x)
    }

    fn deserialize_option<V>(self, _: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::Unsupported("deserialize option".into()))
    }

    fn deserialize_unit<V>(self, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        v.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _: &'static str, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        v.visit_unit()
    }

    fn deserialize_newtype_struct<V>(self, _: &'static str, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        v.visit_seq(DeserializeTuple { de: self })
    }

    fn deserialize_seq<V>(self, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let remaining64 = self.read_uvarint()?;
        let remaining = usize::try_from(remaining64).map_err(|e| Error::Generic(e.to_string()))?;
        v.visit_seq(DeserializeCollection {
            remaining,
            de: self,
        })
    }

    fn deserialize_tuple<V>(self, _: usize, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        v.visit_seq(DeserializeTuple { de: self })
    }

    fn deserialize_tuple_struct<V>(self, _: &'static str, _: usize, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        v.visit_seq(DeserializeTuple { de: self })
    }

    fn deserialize_map<V>(self, _: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::Unsupported("deserialize map".into()))
    }

    fn deserialize_struct<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        v: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        v.visit_seq(DeserializeTuple { de: self })
    }

    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        v: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        v.visit_enum(DeserializeEnum { de: self })
    }

    fn deserialize_identifier<V>(self, v: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_u64(v)
    }

    fn deserialize_ignored_any<V>(self, _: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::Unsupported("deserialize any (ignored)".into()))
    }
}
