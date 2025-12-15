mod de;
mod error;
mod ser;

pub use de::{from_bytes, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_bytes, Serializer};

/*
enum Data {
  UInt8(u8),
  UInt16(u16),
  Int8(i8),
  Vec(Vec<Data>),
  Str(String),
  Float32(f32),
}

trait Encoder {
  fn encode(&self) -> Vec<u8>;
}

trait Decoder {
  fn decode(data: &[u8]) -> (Self, &[u8])
  where
    Self: Sized;
}

impl Encoder for u8 {
  fn encode(&self) -> Vec<u8> {
    let mut buffer: Vec<u8> = Vec::new();
    buffer.push(*self);
    buffer
  }
}

impl Encoder for i8 {
  fn encode(&self) -> Vec<u8> {
    let mut buffer: Vec<u8> = Vec::new();
    let val = (*self as u8).wrapping_add(128);
    buffer.push(val);
    buffer
  }
}

impl Decoder for i8 {
  fn decode(data: &[u8]) -> (Self, &[u8]) {
    let val = data[0].wrapping_sub(128);
    (val as i8, &data[1..])
  }
}

impl<A : Encoder> Encoder for Vec<A> {
  fn encode(&self) -> Vec<u8> {
    let mut buffer: Vec<u8> = Vec::new();
    let length = self.len() as u16;
    buffer.extend_from_slice(&length.to_le_bytes());
    for item in self {
      buffer.push(0); // Separator byte
      buffer.extend_from_slice(&item.encode());
    }
    buffer.push(1); // End byte
    buffer
  }
}

impl<A : Decoder> Decoder for Vec<A> {
  fn decode(data: &[u8]) -> (Self, &[u8]) {
    let mut items: Vec<A> = Vec::new();
    let mut data = data;
    while data[0] != 1 { // End byte
      assert_eq!(data[0], 0); // Separator byte
      data = &data[1..];
      let (item, rest) = A::decode(data);
      items.push(item);
      data = rest;
    }
    assert_eq!(data[0], 1); // End byte
    data = &data[1..];
    (items, data)
  }
}
const SENTINEL: u8 = 0x7F;

impl Encoder for [u8] {
  fn encode(&self) -> Vec<u8> {
    let mut buffer: Vec<u8> = Vec::new();
    for byte   in self {
      buffer.push(*byte);
      if byte == &SENTINEL {
        buffer.push(SENTINEL);
      }
    }
    buffer.push(SENTINEL);
    buffer.push(0x00);
    buffer
  }
}

pub fn encode_slice_with_sentinel(data: &[u8], sentinel: u8) -> Vec<u8> {
  let mut buffer: Vec<u8> = Vec::new();
  for byte   in data {
    buffer.push(*byte);
    if byte == &sentinel {
      buffer.push(sentinel);
    }
  }
  buffer.push(sentinel);
  buffer.push(0x00);
  buffer
}

pub fn decode_slice_with_sentinel(data: &[u8], sentinel: u8) -> (Vec<u8>, &[u8]) {
  let mut bytes: Vec<u8> = Vec::new();
  let mut data = data;
  loop {
    let byte = data[0];
    data = &data[1..];
    if byte == sentinel {
      let next_byte = data[0];
      data = &data[1..];
      if next_byte == 0x00 {
        break;
      } else if next_byte == sentinel {
        bytes.push(sentinel);
      } else {
        panic!("Invalid encoding");
      }
    } else {
      bytes.push(byte);
    }
  }
  (bytes, data)
}
/*
impl Decoder for [u8] {
  fn decode(data: &[u8]) -> (Self, &[u8]) {
    let mut bytes: Vec<u8> = Vec::new();
    let mut data = data;
    loop {
      let byte = data[0];
      data = &data[1..];
      if byte == SENTINEL {
        let next_byte = data[0];
        data = &data[1..];
        if next_byte == 0x00 {
          break;
        } else if next_byte == SENTINEL {
          bytes.push(SENTINEL);
        } else {
          panic!("Invalid encoding");
        }
      } else {
        bytes.push(byte);
      }
    }
    (bytes.as_slice(), data)
  }
}
*/
/*
impl Encoder for &[u8] {
  fn encode(&self) -> Vec<u8> {
    let mut buffer: Vec<u8> = Vec::new();
    for byte in *self {
      buffer.push(0); // Separator byte
      buffer.push(*byte);
    }
    buffer.push(1); // End byte
    buffer
  }
}
  */
/*
impl Decoder for &[u8] {
  fn decode(data: &[u8]) -> (Self, &[u8]) {
    let mut bytes: Vec<u8> = Vec::new();
    let mut data = data;
    while data[0] != 1 { // End byte
      assert_eq!(data[0], 0); // Separator byte
      data = &data[1..];
      bytes.push(data[0]);
      data = &data[1..];
    }
    assert_eq!(data[0], 1); // End byte
    data = &data[1..];
    (bytes.as_slice(), data)
  }
}
  */


impl<A: Encoder> Encoder for Option<A> {
  fn encode(&self) -> Vec<u8> {
    let mut buffer: Vec<u8> = Vec::new();
    match self {
      Some(value) => {
        buffer.push(1); // Some
        buffer.extend_from_slice(&value.encode());
      }
      None => {
        buffer.push(0); // None
      }
    }
    buffer
  }
}

impl<A: Decoder> Decoder for Option<A> {
  fn decode(data: &[u8]) -> (Self, &[u8]) {
    let tag = data[0];
    let data = &data[1..];
    match tag {
      1 => {
        let (value, rest) = A::decode(data);
        (Some(value), rest)
      }
      0 => (None, data),
      _ => panic!("Invalid tag for Option"),
    }
  }
}
  */