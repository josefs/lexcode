mod de;
mod error;
mod fixed_bytes;
mod ser;
mod varint;

pub use de::{from_bytes, Deserializer};
pub use error::{Error, Result};
pub use fixed_bytes::FixedBytes;
pub use ser::{to_bytes, Serializer};

