<p align="center">
  <img src="logo.png" alt="lexcode logo" width="200">
</p>

# lexcode

A [serde](https://serde.rs)-based binary serialization format that **preserves lexicographic ordering**. When two values satisfy `a < b`, their serialized byte representations satisfy `bytes(a) < bytes(b)` under standard byte-wise comparison.

Preserving lexicographic ordering is useful when storing
keys in ordered key-value stores and databases (LSM trees, B-trees, etc.) where sort order must be maintained at the byte level.

Note that lexcode does not support preserving the ordering of explicit implementations of `Ord` or `PartialOrd`. The ordering lexcode will use for e.g. a struct is to first compare the first field, then the second field etc., regardless of any `Ord` and `PartialOrd` implementation.

## Usage

Serialize and deserialize any serde-compatible type:

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Record {
    category: u32,
    name: String,
    score: i64,
}

let record = Record { category: 1, name: "alice".into(), score: -42 };

// Serialize
let bytes = lexcode::to_bytes(&record).unwrap();

// Deserialize
let decoded: Record = lexcode::from_bytes(&bytes).unwrap();
assert_eq!(record, decoded);
```

The encoded bytes preserve ordering — records sort first by `category`, then by `name`, then by `score`, matching the struct field order:

```rust
let a = lexcode::to_bytes(&Record { category: 1, name: "alice".into(), score: 10 }).unwrap();
let b = lexcode::to_bytes(&Record { category: 1, name: "bob".into(),   score: 5  }).unwrap();
let c = lexcode::to_bytes(&Record { category: 2, name: "alice".into(), score: 99 }).unwrap();

assert!(a < b); // same category, "alice" < "bob"
assert!(b < c); // category 1 < category 2
```

## Supported Types

| Type | Encoding |
|---|---|
| `bool` | 1 byte (`0x00` / `0x01`) |
| `u8`, `u16`, `u32`, `u64`, `u128` | Variable-length unsigned varint |
| `i8`, `i16`, `i32`, `i64`, `i128` | Variable-length signed varint (sign bit + magnitude) |
| `f32`, `f64` | IEEE 754 with sign-aware bit flipping (fixed-width) |
| `char` | Variable-length unsigned varint of code point |
| `String` / `&str` | Sentinel-escaped with `0x00` terminator |
| `&[u8]` | Sentinel-escaped with `0x7F` terminator |
| `Option<T>` | `0x00` for `None`, `0x01` + value for `Some` |
| `Vec<T>`, sequences | `0x01` + element per entry, `0x00` terminator |
| Maps | `0x01` + key + value per entry, `0x00` terminator |
| Tuples, structs | Fields concatenated in order (fixed-length) |
| Enums | Varint variant index + variant data |
| `()`, unit structs | Zero bytes |

## Encoding Details

### Integers

Integers use a variable-length encoding that preserves lexicographic ordering while compressing small values into fewer bytes.

#### Unsigned integers

The number of leading 1-bits in the first byte (and optionally a second header byte for large values) determines how many additional data bytes follow. A 0-bit terminates the unary prefix. The remaining header bits plus the extra bytes store the value in big-endian.

| First byte pattern | Total bytes | Data bits | Value range |
|---|---|---|---|
| `0xxxxxxx` | 1 | 7 | 0 – 127 |
| `10xxxxxx` + 1 byte | 2 | 14 | 128 – 16,511 |
| `110xxxxx` + 2 bytes | 3 | 21 | 16,512 – 2,113,663 |
| `1110xxxx` + 3 bytes | 4 | 28 | … |
| `11110xxx` + 4 bytes | 5 | 35 | |
| `111110xx` + 5 bytes | 6 | 42 | |
| `1111110x` + 6 bytes | 7 | 49 | |
| `11111110` + 7 bytes | 8 | 56 | |

When the first byte is `0xFF`, a second header byte extends the scheme in the same way, supporting up to 18 total bytes for the full `u128` range.

Within each level, values are stored as an offset from the level's base, ensuring that all values at a shorter encoding are strictly less than all values at a longer encoding.

#### Signed integers

Bit 7 of the first byte is the **sign bit** (1 = non-negative, 0 = negative). The remaining 7 bits begin the same unary-prefix scheme for the magnitude:

- **Non-negative** values encode the magnitude directly after the sign bit.
- **Negative** values encode `|v| − 1` as the magnitude, then bitwise-complement all bytes (except the sign bit stays 0). This reverses the ordering so that more-negative values produce lexicographically smaller byte sequences.

Small values near zero (both positive and negative) are encoded compactly in 1 byte. For example, values −64 to 63 fit in a single byte.

### Floats

IEEE 754 floats use sign-aware bit manipulation: positive floats have their sign bit flipped; negative floats have all bits flipped. This produces a total ordering over all non-NaN values.

### Strings and Byte Slices

Variable-length types use sentinel encoding to allow unambiguous termination without length prefixes. Each occurrence of the sentinel byte within the data is escaped by doubling it (`sentinel, 0x01`), and the sequence is terminated with `sentinel, 0x00`.

### Sequences and Maps

Variable-length collections prefix each element with `0x01` and end with `0x00`. This preserves element-wise lexicographic comparison.

## Testing

The test suite uses [proptest](https://crates.io/crates/proptest) for property-based testing, verifying both **roundtrip correctness** and **order preservation** across all supported types.

```sh
cargo test
```

## Limitations

- **`deserialize_any` is not supported.** Since lexcode is a non-self-describing binary format, the deserializer must know the expected type at compile time. This means dynamically-typed values like `serde_json::Value` cannot be deserialized from lexcode.

## Disclaimer

This library is in no way associated with Lex Fridman. This library cannot conduct a 4-hour podcast interview with your data structures.

## License

Licensed under the [MIT License](LICENSE).
