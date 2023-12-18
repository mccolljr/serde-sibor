use crate::error::{Error, Result};

pub struct CollectionSerializer<'a, W> {
    remaining: usize,
    ser: &'a mut Serializer<W>,
}
impl<'a, W> ::serde::ser::SerializeSeq for CollectionSerializer<'a, W>
where
    W: ::std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        if self.remaining < 1 {
            return Err(Error::Generic(
                "tried to serialize too many elements in collection".into(),
            ));
        }
        self.remaining -= 1;
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

pub struct TupleSerializer<'a, W> {
    ser: &'a mut Serializer<W>,
}

impl<'a, W> ::serde::ser::SerializeTuple for TupleSerializer<'a, W>
where
    W: ::std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W> ::serde::ser::SerializeTupleStruct for TupleSerializer<'a, W>
where
    W: ::std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W> ::serde::ser::SerializeTupleVariant for TupleSerializer<'a, W>
where
    W: ::std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W> ::serde::ser::SerializeStruct for TupleSerializer<'a, W>
where
    W: ::std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W> ::serde::ser::SerializeStructVariant for TupleSerializer<'a, W>
where
    W: ::std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

pub struct Serializer<W> {
    writer: W,
}

impl<W> Serializer<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W> Serializer<W>
where
    W: ::std::io::Write,
{
    pub fn zigzag(&self, v: i64) -> u64 {
        let mut unsigned = (v as u64) << 1;
        if v < 0 {
            unsigned = !unsigned;
        }
        unsigned
    }

    pub fn sizeof_uvarint(&self, v: &u64) -> Result<usize> {
        let mut v = *v;
        let mut size = 1usize;
        while v >= 0x80 {
            size += 1;
            v >>= 7;
        }
        Ok(size)
    }

    pub fn sizeof_varint(&self, v: &i64) -> Result<usize> {
        let unsigned = self.zigzag(*v);
        self.sizeof_uvarint(&unsigned)
    }

    pub fn sizeof_float(&self, v: &f64) -> Result<usize> {
        return self.sizeof_uvarint(&v.to_bits());
    }

    pub fn sizeof_bool(&self, _: bool) -> Result<usize> {
        Ok(1)
    }

    pub fn sizeof_string(&self, v: &str) -> Result<usize> {
        self.sizeof_bytes(v.as_bytes())
    }

    pub fn sizeof_bytes(&self, v: &[u8]) -> Result<usize> {
        let len64 = u64::try_from(v.len()).map_err(|e| Error::Generic(e.to_string()))?;
        Self::combine_sizes([self.sizeof_uvarint(&len64)?, v.len()])
    }

    pub fn write_exact(&mut self, buf: &[u8]) -> Result<()> {
        self.writer.write_all(buf).map_err(Error::Io)
    }

    pub fn write_u8(&mut self, v: u8) -> Result<()> {
        self.write_exact(&[v])
    }

    pub fn write_uvarint(&mut self, mut v: u64) -> Result<()> {
        while v >= 0x80 {
            self.write_u8((v & 0x7f) as u8 | 0x80)?;
            v >>= 7;
        }
        self.write_u8((v & 0x7f) as u8)?;
        Ok(())
    }

    pub fn write_ivarint(&mut self, v: i64) -> Result<()> {
        let unsigned = self.zigzag(v);
        self.write_uvarint(unsigned)
    }

    pub fn write_float(&mut self, v: f64) -> Result<()> {
        self.write_uvarint(v.to_bits())
    }

    pub fn write_bool(&mut self, v: bool) -> Result<()> {
        self.write_u8(if v { 1 } else { 0 })
    }

    pub fn write_bytes(&mut self, v: &[u8]) -> Result<()> {
        let len64 = u64::try_from(v.len()).map_err(|e| Error::Generic(e.to_string()))?;
        self.write_uvarint(len64)?;
        self.write_exact(v)?;
        Ok(())
    }

    pub fn write_string(&mut self, v: &str) -> Result<()> {
        self.write_bytes(v.as_bytes())
    }

    fn combine_sizes(sizes: impl IntoIterator<Item = usize>) -> Result<usize> {
        sizes
            .into_iter()
            .fold(Some(0), |agg, v| Some(agg? + v))
            .ok_or_else(|| Error::Generic("size too large".into()))
    }
}

impl<'a, W> ::serde::ser::Serializer for &'a mut Serializer<W>
where
    W: ::std::io::Write,
{
    type Error = Error;
    type Ok = ();

    type SerializeSeq = CollectionSerializer<'a, W>;

    type SerializeTuple = TupleSerializer<'a, W>;

    type SerializeTupleStruct = TupleSerializer<'a, W>;

    type SerializeTupleVariant = TupleSerializer<'a, W>;

    type SerializeMap = ::serde::ser::Impossible<(), Error>;

    type SerializeStruct = TupleSerializer<'a, W>;

    type SerializeStructVariant = TupleSerializer<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.write_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.write_ivarint(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.write_ivarint(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.write_ivarint(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.write_ivarint(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.write_uvarint(v as u64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.write_uvarint(v as u64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.write_uvarint(v as u64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.write_uvarint(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.write_float(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.write_float(v)
    }

    fn serialize_char(self, _: char) -> Result<Self::Ok> {
        Err(Error::Unsupported("serialize char".into()))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.write_string(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.write_bytes(v)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Err(Error::Unsupported("serialize option".into()))
    }

    fn serialize_some<T: ?Sized>(self, _: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        Err(Error::Unsupported("serialize option".into()))
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
    ) -> Result<Self::Ok> {
        self.write_uvarint(variant_index as u64)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        self.write_uvarint(variant_index as u64)?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        match len {
            Some(remaining) => {
                let len64 = u64::try_from(remaining).map_err(|e| Error::Generic(e.to_string()))?;
                self.write_uvarint(len64)?;
                Ok(CollectionSerializer {
                    remaining,
                    ser: self,
                })
            }
            None => Err(Error::Unsupported("serialize seq (unsized)".into())),
        }
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple> {
        Ok(TupleSerializer { ser: self })
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(TupleSerializer { ser: self })
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.write_uvarint(variant_index as u64)?;
        Ok(TupleSerializer { ser: self })
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::Unsupported("serialize map".into()))
    }

    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct> {
        Ok(TupleSerializer { ser: self })
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.write_uvarint(variant_index as u64)?;
        Ok(TupleSerializer { ser: self })
    }
}
