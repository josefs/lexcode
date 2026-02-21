use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

pub(crate) const FIXED_BYTES_NAME: &str = "lexcode::FixedBytes";

/// A fixed-size byte array that serializes as raw bytes with zero overhead.
///
/// Unlike `[u8; N]`, which varint-encodes each byte element,
/// `FixedBytes<N>` writes each byte directly as a single raw byte.
/// This produces exactly `N` bytes of output, with no framing or escaping.
///
/// Lexicographic ordering is preserved: byte-wise comparison of the encoded
/// output matches comparison of the original arrays.
///
/// # Example
///
/// ```
/// use lexcode::FixedBytes;
///
/// let hash = FixedBytes([0xde, 0xad, 0xbe, 0xef]);
/// let bytes = lexcode::to_bytes(&hash).unwrap();
/// assert_eq!(bytes.len(), 4); // exactly 4 bytes, zero overhead
/// assert_eq!(&bytes, &[0xde, 0xad, 0xbe, 0xef]);
///
/// let decoded: FixedBytes<4> = lexcode::from_bytes(&bytes).unwrap();
/// assert_eq!(hash, decoded);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedBytes<const N: usize>(pub [u8; N]);

impl<const N: usize> From<[u8; N]> for FixedBytes<N> {
    fn from(arr: [u8; N]) -> Self {
        FixedBytes(arr)
    }
}

impl<const N: usize> From<FixedBytes<N>> for [u8; N] {
    fn from(fb: FixedBytes<N>) -> Self {
        fb.0
    }
}

impl<const N: usize> AsRef<[u8]> for FixedBytes<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> Serialize for FixedBytes<N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeTupleStruct;
        let mut ts = serializer.serialize_tuple_struct(FIXED_BYTES_NAME, N)?;
        for &byte in &self.0 {
            ts.serialize_field(&byte)?;
        }
        ts.end()
    }
}

impl<'de, const N: usize> Deserialize<'de> for FixedBytes<N> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FixedBytesVisitor<const M: usize>;

        impl<'de, const M: usize> Visitor<'de> for FixedBytesVisitor<M> {
            type Value = FixedBytes<M>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{M} raw bytes")
            }

            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<FixedBytes<M>, A::Error> {
                let mut arr = [0u8; M];
                for (i, slot) in arr.iter_mut().enumerate() {
                    *slot = seq
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(i, &self))?;
                }
                Ok(FixedBytes(arr))
            }
        }

        deserializer.deserialize_tuple_struct(FIXED_BYTES_NAME, N, FixedBytesVisitor::<N>)
    }
}
