use serde::Deserialize;
use serde::de::value::U32Deserializer;
use serde::de::{
    self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess,
    VariantAccess, Visitor, IntoDeserializer,
};

use crate::error::{Error, Result};

pub struct Deserializer<'de> {
  input: &'de [u8],
}

impl<'de> Deserializer<'de> {
  pub fn from_bytes(input: &'de [u8]) -> Self {
    Deserializer { input }
  }
}

pub fn from_bytes<'a, T>(s: &'a [u8]) -> Result<T>
where
  T: Deserialize<'a>,
{
  let mut deserializer = Deserializer::from_bytes(s);
  let t = T::deserialize(&mut deserializer)?;
  if deserializer.input.is_empty() {
    Ok(t)
  } else {
    Err(Error::TrailingCharacters)
  }
}

// TODO: don't require &mut self everywhere
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
    let byte = self.input[0];
    self.input = &self.input[1..];
    match byte {
      0 => visitor.visit_bool(false),
      1 => visitor.visit_bool(true),
      _ => Err(Error::Message("Invalid boolean value".to_string())),
    }
  }

  fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let byte = self.input[0];
    self.input = &self.input[1..];
    let value = (byte as u8).wrapping_sub(128);
    visitor.visit_i8(value as i8)
  }

  fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let bytes = &self.input[0..2];
    self.input = &self.input[2..];
    let val = u16::from_be_bytes([bytes[0], bytes[1]]).wrapping_sub(32768);
    visitor.visit_i16(val as i16)
  }

  fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let bytes = &self.input[0..4];
    self.input = &self.input[4..];
    let val = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]).wrapping_sub(2147483648);
    visitor.visit_i32(val as i32)
  }

  fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let bytes = &self.input[0..8];
    self.input = &self.input[8..];
    let val = u64::from_be_bytes([
      bytes[0], bytes[1], bytes[2], bytes[3],
      bytes[4], bytes[5], bytes[6], bytes[7],
    ]).wrapping_sub(9223372036854775808);
    visitor.visit_i64(val as i64)
  }

  fn deserialize_i128<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
      where
          V: Visitor<'de>, {
    let bytes = &self.input[0..16];
    self.input = &self.input[16..];
    let val = u128::from_be_bytes([
      bytes[0], bytes[1], bytes[2], bytes[3],
      bytes[4], bytes[5], bytes[6], bytes[7],
      bytes[8], bytes[9], bytes[10], bytes[11],
      bytes[12], bytes[13], bytes[14], bytes[15],
    ]).wrapping_sub(170141183460469231731687303715884105728);
    visitor.visit_i128(val as i128)
  }

  fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let byte = self.input[0];
    self.input = &self.input[1..];
    visitor.visit_u8(byte)
  }

  fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let bytes = &self.input[0..2];
    self.input = &self.input[2..];
    let val = u16::from_be_bytes([bytes[0], bytes[1]]);
    visitor.visit_u16(val)
  }

  fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let bytes = &self.input[0..4];
    self.input = &self.input[4..];
    let val = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    visitor.visit_u32(val)
  }

  fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let bytes = &self.input[0..8];
    self.input = &self.input[8..];
    let val = u64::from_be_bytes([
      bytes[0], bytes[1], bytes[2], bytes[3],
      bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    visitor.visit_u64(val)
  }

  fn deserialize_u128<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
      where
          V: Visitor<'de>, {
    let bytes = &self.input[0..16];
    self.input = &self.input[16..];
    let val = u128::from_be_bytes([
      bytes[0], bytes[1], bytes[2], bytes[3],
      bytes[4], bytes[5], bytes[6], bytes[7],
      bytes[8], bytes[9], bytes[10], bytes[11],
      bytes[12], bytes[13], bytes[14], bytes[15],
    ]);
    visitor.visit_u128(val)
  }

  fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let bytes = &self.input[0..4];
    self.input = &self.input[4..];
    let mut v = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    const SIGN_MASK: u32 = 1 << 31;
    if (v & SIGN_MASK) == 0 {
      v = !v;
    } else {
      v = v ^ SIGN_MASK;
    }
    let f = f32::from_bits(v);
    visitor.visit_f32(f)
  }

  fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let bytes = &self.input[0..8];
    self.input = &self.input[8..];
    let mut v = u64::from_be_bytes([
      bytes[0], bytes[1], bytes[2], bytes[3],
      bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    const SIGN_MASK: u64 = 1 << 63;
    if (v & SIGN_MASK) == 0 {
      v = !v;
    } else {
      v = v ^ SIGN_MASK;
    }
    let f = f64::from_bits(v);
    visitor.visit_f64(f)
  }

  fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let bytes = &self.input[0..4];
    self.input = &self.input[4..];
    let code_point = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
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
    let byte = self.input[0];
    self.input = &self.input[1..];
    match byte {
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
    visitor.visit_seq(self)
  }

  fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    struct TupleAccess<'a, 'de> {
      deserializer: &'a mut Deserializer<'de>,
      remaining: usize,
    }
    impl <'de, 'a> SeqAccess<'de> for TupleAccess<'a, 'de> {
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
    visitor.visit_seq(TupleAccess {
      deserializer: self,
      remaining: len,
    })
  }

  fn deserialize_tuple_struct<V>(
          self,
          _name: &'static str,
          len: usize,
          visitor: V,
      ) -> Result<V::Value>
      where
          V: Visitor<'de> {
    self.deserialize_tuple(len, visitor)
  }

  fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_map(self)
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
    struct StructAccess<'a, 'de> {
      deserializer: &'a mut Deserializer<'de>,
      remaining: usize,
    }
    impl <'de, 'a> SeqAccess<'de> for StructAccess<'a, 'de> {
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
    visitor.visit_seq(StructAccess {
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
//    let variant_index = self.deserialize_u32(de::IgnoredAny)?;
//    visitor.visit_enum(variant_index)
//
    visitor.visit_enum(U32Deserializer::new({
      let bytes = &self.input[0..4];
      self.input = &self.input[4..];
      u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }))
    //
    /*
    let imm_self = self;
    visitor.visit_enum(imm_self)
    */
  }

  fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_unit()
  }

  fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value>
      where
          V: Visitor<'de>
  {
      Err(Error::Message("deserialize_identifier is not supported".to_string())) // TODO
  }

}

impl Deserializer<'_> {
  fn deserialize_with_sentinel(&mut self, sentinel: u8) -> Result<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();
    loop {
      let byte = self.input[0];
      self.input = &self.input[1..];
      if byte == sentinel {
        let next_byte = self.input[0];
        self.input = &self.input[1..];
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

impl<'de, 'a> SeqAccess<'de> for Deserializer<'de> {
  type Error = Error;

  fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
  where
    T: DeserializeSeed<'de>,
  {
    let byte = self.input[0];
    self.input = &self.input[1..];
    match byte {
      0x00 => Ok(None),
      0x01 => {
        let value = seed.deserialize(&mut *self)?;
        Ok(Some(value))
      }
      _ => Err(Error::Message("Invalid sequence encoding".to_string())),
    }
  }
}

impl<'de, 'a> MapAccess<'de> for Deserializer<'de> {
  type Error = Error;

  fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
  where
    K: DeserializeSeed<'de>,
  {
    let byte = self.input[0];
    self.input = &self.input[1..];
    match byte {
      0x00 => Ok(None),
      0x01 => {
        let key = seed.deserialize(&mut *self)?;
        Ok(Some(key))
      }
      _ => Err(Error::Message("Invalid map encoding".to_string())),
    }
  }

  fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
  where
    V: DeserializeSeed<'de>,
  {
    let value = seed.deserialize(&mut *self)?;
    Ok(value)
  }
}

impl<'de> EnumAccess<'de> for Deserializer<'de> {
  type Error = Error;
  type Variant = Self;

  fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self)>
  where
    V: DeserializeSeed<'de>,
  {
//    let idx = u32::decode(&mut self)?;
    let bytes = &self.input[0..4];
    let mut mut_self = self;
    mut_self.input = &mut_self.input[4..];
    let variant_index = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

//    let variant_index = (&mut self).deserialize_u32(de::IgnoredAny)?;
    let val = seed.deserialize(variant_index.into_deserializer())?;
    let imm_self = mut_self;
    Ok((val, imm_self))
    /*
    let mut mut_self = self;
    let v = seed.deserialize(&mut mut_self)?;
    let imm_self = mut_self;
    Ok((v, imm_self))
    */
  }
}

impl<'de> VariantAccess<'de> for Deserializer<'de> {
  type Error = Error;

  fn unit_variant(self) -> Result<()> {
    Ok(())
  }

  fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
  where
    T: DeserializeSeed<'de>,
  {
    let mut mut_self = self;
    seed.deserialize(&mut mut_self)
  }

  fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    struct TupleVariantAccess<'a, 'de> {
      deserializer: &'a mut Deserializer<'de>,
      remaining: usize,
    }
    impl <'de, 'a> SeqAccess<'de> for TupleVariantAccess<'a, 'de> {
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
    let mut mut_self = self;
    visitor.visit_seq(TupleVariantAccess {
      deserializer: &mut mut_self,
      remaining: len,
    })
  }

  fn struct_variant<V>(
      self,
      _fields: &'static [&'static str],
      visitor: V,
  ) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_seq(self)
  }
}