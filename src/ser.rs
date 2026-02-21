use serde::{ser, Serialize};

use crate::error::{Error, Result};
use crate::varint;

pub struct Serializer {
    output: Vec<u8>,
    raw_byte_mode: bool,
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut serializer = Serializer { output: Vec::new(), raw_byte_mode: false };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
  type Ok = ();
  type Error = Error;

  type SerializeSeq = Self;
  type SerializeTuple = Self;
  type SerializeTupleStruct = Self;
  type SerializeTupleVariant = Self;
  type SerializeMap = Self;
  type SerializeStruct = Self;
  type SerializeStructVariant = Self;

  fn serialize_bool(self, v: bool) -> Result<()> {
      self.output.push(if v { 1 } else { 0 });
      Ok(())
  }

  fn serialize_i8(self, v: i8) -> Result<()> {
    varint::encode_sint(v as i128, &mut self.output);
    Ok(())
  }

  fn serialize_i16(self, v: i16) -> Result<()> {
    varint::encode_sint(v as i128, &mut self.output);
    Ok(())
  }

  fn serialize_i32(self, v: i32) -> Result<()> {
    varint::encode_sint(v as i128, &mut self.output);
    Ok(())
  }

  fn serialize_i64(self, v: i64) -> Result<()> {
    varint::encode_sint(v as i128, &mut self.output);
    Ok(())
  }

  fn serialize_i128(self, v: i128) -> std::result::Result<Self::Ok, Self::Error> {
    varint::encode_sint(v, &mut self.output);
    Ok(())
  }

  fn serialize_u8(self, v: u8) -> Result<()> {
    if self.raw_byte_mode {
      self.output.push(v);
    } else {
      varint::encode_uint(v as u128, &mut self.output);
    }
    Ok(())
  }

  fn serialize_u16(self, v: u16) -> Result<()> {
    varint::encode_uint(v as u128, &mut self.output);
    Ok(())
  }

  fn serialize_u32(self, v: u32) -> Result<()> {
    varint::encode_uint(v as u128, &mut self.output);
    Ok(())
  }

  fn serialize_u64(self, v: u64) -> Result<()> {
    varint::encode_uint(v as u128, &mut self.output);
    Ok(())
  }

  fn serialize_u128(self, v: u128) -> std::result::Result<Self::Ok, Self::Error> {
    varint::encode_uint(v, &mut self.output);
    Ok(())
  }

  fn serialize_f32(self, v: f32) -> Result<()> {
    let v = v.to_bits();
    const SIGN_MASK: u32 = 1 << 31;
    if (v & SIGN_MASK) != 0 {
        // Negative number: flip all bits
        let v = !v;
        self.output.extend_from_slice(&v.to_be_bytes());
    } else {
        // Positive number: flip sign bit
        let v = v ^ SIGN_MASK;
        self.output.extend_from_slice(&v.to_be_bytes());
    }
    Ok(())
  }

  fn serialize_f64(self, v: f64) -> Result<()> {
    let v = v.to_bits();
    const SIGN_MASK: u64 = 1 << 63;
    if (v & SIGN_MASK) != 0 {
        // Negative number: flip all bits
        let v = !v;
        self.output.extend_from_slice(&v.to_be_bytes());
    } else {
        // Positive number: flip sign bit
        let v = v ^ SIGN_MASK;
        self.output.extend_from_slice(&v.to_be_bytes());
    }
    Ok(())
  }

  fn serialize_char(self, c: char) -> Result<()> {
    varint::encode_uint(c as u128, &mut self.output);
    Ok(())
  }

  fn serialize_str(self, v: &str) -> Result<()> {
    self.serialize_with_sentinel(v.as_bytes(), 0x00)
  }

  fn serialize_bytes(self, v: &[u8]) -> Result<()> {
    self.serialize_with_sentinel(v, 0x7F)
  }

  fn serialize_none(self) -> Result<()> {
    self.output.push(0x00);
    Ok(())
  }

  fn serialize_some<T>(self, value: &T) -> Result<()>
  where
      T: ?Sized + Serialize,
  {
    self.output.push(0x01);
    value.serialize(self)
  }

  fn serialize_unit(self) -> Result<()> {
    Ok(())
  }

  fn serialize_unit_variant(
      self,
      _name: &'static str,
      variant_index: u32,
      _variant: &'static str,
  ) -> Result<()> {
    self.serialize_u32(variant_index)
  }

  fn serialize_newtype_struct<T>(
      self,
      _name: &'static str,
      value: &T,
  ) -> Result<()>
  where
      T: ?Sized + Serialize,
  {
    value.serialize(self)
  }

  fn serialize_newtype_variant<T>(
      self,
      _name: &'static str,
      variant_index: u32,
      _variant: &'static str,
      value: &T,
  ) -> Result<()>
  where
      T: ?Sized + Serialize,
  {
    self.serialize_u32(variant_index)?;
    value.serialize(self)
  }

  fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
    Ok(self)
  }

  fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
    Ok(self)
  }

  fn serialize_tuple_struct(
      self,
      name: &'static str,
      _len: usize,
  ) -> Result<Self::SerializeTupleStruct> {
    if name == crate::fixed_bytes::FIXED_BYTES_NAME {
      self.raw_byte_mode = true;
    }
    Ok(self)
  }

  fn serialize_tuple_variant(
      self,
      _name: &'static str,
      variant_index: u32,
      _variant: &'static str,
      _len: usize,
  ) -> Result<Self::SerializeTupleVariant> {
    self.serialize_u32(variant_index)?;
    Ok(self)
  }

  fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
    Ok(self)
  }

  fn serialize_struct(
      self,
      _name: &'static str,
      _len: usize,
  ) -> Result<Self::SerializeStruct> {
    Ok(self)
  }

  fn serialize_struct_variant(
      self,
      _name: &'static str,
      variant_index: u32,
      _variant: &'static str,
      _len: usize,
  ) -> Result<Self::SerializeStructVariant> {
    self.serialize_u32(variant_index)?;
    Ok(self)
  }

  fn serialize_unit_struct(self, _name: &'static str) -> std::result::Result<Self::Ok, Self::Error> {
    Ok(())
  }
}

impl Serializer {
  fn serialize_with_sentinel(&mut self, data: &[u8], sentinel: u8) -> Result<()> {
    for byte in data {
      self.output.push(*byte);
      if byte == &sentinel {
        self.output.push(0x01);
      }
    }
    self.output.push(sentinel);
    self.output.push(0x00);
    Ok(())
  }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_element<T>(&mut self, value: &T) -> Result<()>
  where
      T: ?Sized + Serialize,
  {
    self.output.push(0x01); // Element separator
    value.serialize(&mut **self)
  }

  fn end(self) -> Result<()> {
    self.output.push(0x00); // End of sequence
    Ok(())
  }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_element<T>(&mut self, value: &T) -> Result<()>
  where
      T: ?Sized + Serialize,
  {
    value.serialize(&mut **self)
  }

  fn end(self) -> Result<()> {
    Ok(())
  }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_field<T>(&mut self, value: &T) -> Result<()>
  where
      T: ?Sized + Serialize,
  {
    value.serialize(&mut **self)
  }

  fn end(self) -> Result<()> {
    self.raw_byte_mode = false;
    Ok(())
  }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_field<T>(&mut self, value: &T) -> Result<()>
  where
      T: ?Sized + Serialize,
  {
    value.serialize(&mut **self)
  }

  fn end(self) -> Result<()> {
    Ok(())
  }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_key<T>(&mut self, key: &T) -> Result<()>
  where
      T: ?Sized + Serialize,
  {
    self.output.push(0x01); // Key separator
    key.serialize(&mut **self)
  }

  fn serialize_value<T>(&mut self, value: &T) -> Result<()>
  where
      T: ?Sized + Serialize,
  {
    value.serialize(&mut **self)
  }

  fn end(self) -> Result<()> {
    self.output.push(0x00); // End of map
    Ok(())
  }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
  where
      T: ?Sized + Serialize,
  {
    value.serialize(&mut **self)
  }

  fn end(self) -> Result<()> {
    Ok(())
  }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
  where
      T: ?Sized + Serialize,
  {
    value.serialize(&mut **self)
  }

  fn end(self) -> Result<()> {
    Ok(())
  }
}

