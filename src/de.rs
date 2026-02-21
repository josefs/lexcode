use serde::Deserialize;
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};

use crate::error::{Error, Result};
use crate::varint;

pub struct Deserializer<'de> {
  input: &'de [u8],
  raw_byte_mode: bool,
}

impl<'de> Deserializer<'de> {
  pub fn from_bytes(input: &'de [u8]) -> Self {
    Deserializer { input, raw_byte_mode: false }
  }

  fn read_bytes(&mut self, n: usize) -> Result<&'de [u8]> {
    if self.input.len() < n {
      return Err(Error::Eof);
    }
    let (head, tail) = self.input.split_at(n);
    self.input = tail;
    Ok(head)
  }

  fn read_u8(&mut self) -> Result<u8> {
    if self.input.is_empty() {
      return Err(Error::Eof);
    }
    let b = self.input[0];
    self.input = &self.input[1..];
    Ok(b)
  }

  fn read_u32_varint(&mut self) -> Result<u32> {
    let (v, consumed) = varint::decode_uint(self.input)?;
    self.input = &self.input[consumed..];
    u32::try_from(v).map_err(|_| Error::Message("integer overflow: value does not fit in u32".into()))
  }

  fn read_raw_u32(&mut self) -> Result<u32> {
    let bytes = self.read_bytes(4)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
  }

  fn read_raw_u64(&mut self) -> Result<u64> {
    let bytes = self.read_bytes(8)?;
    Ok(u64::from_be_bytes([
      bytes[0], bytes[1], bytes[2], bytes[3],
      bytes[4], bytes[5], bytes[6], bytes[7],
    ]))
  }

  fn deserialize_with_sentinel(&mut self, sentinel: u8) -> Result<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();
    loop {
      let byte = self.read_u8()?;
      if byte == sentinel {
        let next_byte = self.read_u8()?;
        if next_byte == 0x00 {
          break;
        } else if next_byte == 0x01 {
          bytes.push(sentinel);
          continue;
        } else {
          return Err(Error::Message("Invalid encoding".to_string()));
        }
      } else {
        bytes.push(byte);
      }
    }
    Ok(bytes)
  }
}

pub fn from_bytes<'a, T>(s: &'a [u8]) -> Result<T>
where
  T: Deserialize<'a>,
{
  let mut deserializer = Deserializer::from_bytes(s);
  let t = T::deserialize(&mut deserializer)?;
  Ok(t)
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
  type Error = Error;

  fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    Err(Error::Message("deserialize_any is not supported".to_string()))
  }

  fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    match self.read_u8()? {
      0 => visitor.visit_bool(false),
      1 => visitor.visit_bool(true),
      _ => Err(Error::Message("Invalid boolean value".to_string())),
    }
  }

  fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let (v, consumed) = varint::decode_sint(self.input)?;
    self.input = &self.input[consumed..];
    let v = i8::try_from(v).map_err(|_| Error::Message("integer overflow: value does not fit in i8".into()))?;
    visitor.visit_i8(v)
  }

  fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let (v, consumed) = varint::decode_sint(self.input)?;
    self.input = &self.input[consumed..];
    let v = i16::try_from(v).map_err(|_| Error::Message("integer overflow: value does not fit in i16".into()))?;
    visitor.visit_i16(v)
  }

  fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let (v, consumed) = varint::decode_sint(self.input)?;
    self.input = &self.input[consumed..];
    let v = i32::try_from(v).map_err(|_| Error::Message("integer overflow: value does not fit in i32".into()))?;
    visitor.visit_i32(v)
  }

  fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let (v, consumed) = varint::decode_sint(self.input)?;
    self.input = &self.input[consumed..];
    let v = i64::try_from(v).map_err(|_| Error::Message("integer overflow: value does not fit in i64".into()))?;
    visitor.visit_i64(v)
  }

  fn deserialize_i128<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    let (v, consumed) = varint::decode_sint(self.input)?;
    self.input = &self.input[consumed..];
    visitor.visit_i128(v)
  }

  fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    if self.raw_byte_mode {
      let b = self.read_u8()?;
      return visitor.visit_u8(b);
    }
    let (v, consumed) = varint::decode_uint(self.input)?;
    self.input = &self.input[consumed..];
    let v = u8::try_from(v).map_err(|_| Error::Message("integer overflow: value does not fit in u8".into()))?;
    visitor.visit_u8(v)
  }

  fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let (v, consumed) = varint::decode_uint(self.input)?;
    self.input = &self.input[consumed..];
    let v = u16::try_from(v).map_err(|_| Error::Message("integer overflow: value does not fit in u16".into()))?;
    visitor.visit_u16(v)
  }

  fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let (v, consumed) = varint::decode_uint(self.input)?;
    self.input = &self.input[consumed..];
    let v = u32::try_from(v).map_err(|_| Error::Message("integer overflow: value does not fit in u32".into()))?;
    visitor.visit_u32(v)
  }

  fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let (v, consumed) = varint::decode_uint(self.input)?;
    self.input = &self.input[consumed..];
    let v = u64::try_from(v).map_err(|_| Error::Message("integer overflow: value does not fit in u64".into()))?;
    visitor.visit_u64(v)
  }

  fn deserialize_u128<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    let (v, consumed) = varint::decode_uint(self.input)?;
    self.input = &self.input[consumed..];
    visitor.visit_u128(v)
  }

  fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let mut v = self.read_raw_u32()?;
    const SIGN_MASK: u32 = 1 << 31;
    if (v & SIGN_MASK) == 0 {
      v = !v;
    } else {
      v = v ^ SIGN_MASK;
    }
    visitor.visit_f32(f32::from_bits(v))
  }

  fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let mut v = self.read_raw_u64()?;
    const SIGN_MASK: u64 = 1 << 63;
    if (v & SIGN_MASK) == 0 {
      v = !v;
    } else {
      v = v ^ SIGN_MASK;
    }
    visitor.visit_f64(f64::from_bits(v))
  }

  fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let (v, consumed) = varint::decode_uint(self.input)?;
    self.input = &self.input[consumed..];
    let code_point = v as u32;
    match std::char::from_u32(code_point) {
      Some(c) => visitor.visit_char(c),
      None => Err(Error::Message("Invalid char code point".to_string())),
    }
  }

  fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.deserialize_with_sentinel(0x00).and_then(|bytes: Vec<u8>| {
      match std::str::from_utf8(&bytes) {
        Ok(s) => visitor.visit_str(s),
        Err(_) => Err(Error::Message("Invalid UTF-8 string".to_string())),
      }
    })
  }

  fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.deserialize_str(visitor)
  }

  fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
    self.deserialize_with_sentinel(0x7F).and_then(|bytes| {
      visitor.visit_bytes(&bytes)
    })
  }

  fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.deserialize_bytes(visitor)
  }

  fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    match self.read_u8()? {
      0 => visitor.visit_none(),
      1 => visitor.visit_some(self),
      _ => Err(Error::Message("Invalid option encoding".to_string())),
    }
  }

  fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_unit()
  }

  fn deserialize_unit_struct<V>(
      self,
      _name: &'static str,
      visitor: V,
  ) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.deserialize_unit(visitor)
  }

  fn deserialize_newtype_struct<V>(
      self,
      _name: &'static str,
      visitor: V,
  ) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_newtype_struct(self)
  }

  fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_seq(SeqAccessor { deserializer: self })
  }

  fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_seq(FixedLenAccessor {
      deserializer: self,
      remaining: len,
    })
  }

  fn deserialize_tuple_struct<V>(
      self,
      name: &'static str,
      len: usize,
      visitor: V,
  ) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    if name == crate::fixed_bytes::FIXED_BYTES_NAME {
      self.raw_byte_mode = true;
      let result = visitor.visit_seq(FixedLenAccessor {
        deserializer: &mut *self,
        remaining: len,
      });
      self.raw_byte_mode = false;
      result
    } else {
      self.deserialize_tuple(len, visitor)
    }
  }

  fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_map(MapAccessor { deserializer: self })
  }

  fn deserialize_struct<V>(
      self,
      _name: &'static str,
      fields: &'static [&'static str],
      visitor: V,
  ) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_seq(FixedLenAccessor {
      deserializer: self,
      remaining: fields.len(),
    })
  }

  fn deserialize_enum<V>(
      self,
      _name: &'static str,
      _variants: &'static [&'static str],
      visitor: V,
  ) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_enum(EnumAccessor { deserializer: self })
  }

  fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_unit()
  }

  fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    Err(Error::Message("deserialize_identifier is not supported".to_string()))
  }
}

// Helper for variable-length sequences (Vec, etc.)
struct SeqAccessor<'a, 'de> {
  deserializer: &'a mut Deserializer<'de>,
}

impl<'de, 'a> SeqAccess<'de> for SeqAccessor<'a, 'de> {
  type Error = Error;

  fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
  where
    T: DeserializeSeed<'de>,
  {
    match self.deserializer.read_u8()? {
      0x00 => Ok(None),
      0x01 => {
        let value = seed.deserialize(&mut *self.deserializer)?;
        Ok(Some(value))
      }
      _ => Err(Error::Message("Invalid sequence encoding".to_string())),
    }
  }
}

// Helper for fixed-length sequences (tuples, structs)
struct FixedLenAccessor<'a, 'de> {
  deserializer: &'a mut Deserializer<'de>,
  remaining: usize,
}

impl<'de, 'a> SeqAccess<'de> for FixedLenAccessor<'a, 'de> {
  type Error = Error;

  fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
  where
    T: DeserializeSeed<'de>,
  {
    if self.remaining == 0 {
      return Ok(None);
    }
    self.remaining -= 1;
    let value = seed.deserialize(&mut *self.deserializer)?;
    Ok(Some(value))
  }
}

// Helper for map access
struct MapAccessor<'a, 'de> {
  deserializer: &'a mut Deserializer<'de>,
}

impl<'de, 'a> MapAccess<'de> for MapAccessor<'a, 'de> {
  type Error = Error;

  fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
  where
    K: DeserializeSeed<'de>,
  {
    match self.deserializer.read_u8()? {
      0x00 => Ok(None),
      0x01 => {
        let key = seed.deserialize(&mut *self.deserializer)?;
        Ok(Some(key))
      }
      _ => Err(Error::Message("Invalid map encoding".to_string())),
    }
  }

  fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
  where
    V: DeserializeSeed<'de>,
  {
    seed.deserialize(&mut *self.deserializer)
  }
}

// Helper for enum access
struct EnumAccessor<'a, 'de> {
  deserializer: &'a mut Deserializer<'de>,
}

impl<'de, 'a> EnumAccess<'de> for EnumAccessor<'a, 'de> {
  type Error = Error;
  type Variant = Self;

  fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self)>
  where
    V: DeserializeSeed<'de>,
  {
    let variant_index = self.deserializer.read_u32_varint()?;
    let val = seed.deserialize(variant_index.into_deserializer())?;
    Ok((val, self))
  }
}

impl<'de, 'a> VariantAccess<'de> for EnumAccessor<'a, 'de> {
  type Error = Error;

  fn unit_variant(self) -> Result<()> {
    Ok(())
  }

  fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
  where
    T: DeserializeSeed<'de>,
  {
    seed.deserialize(&mut *self.deserializer)
  }

  fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_seq(FixedLenAccessor {
      deserializer: self.deserializer,
      remaining: len,
    })
  }

  fn struct_variant<V>(
      self,
      fields: &'static [&'static str],
      visitor: V,
  ) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_seq(FixedLenAccessor {
      deserializer: self.deserializer,
      remaining: fields.len(),
    })
  }
}