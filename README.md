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
| `u8`, `u16`, `u32`, `u64`, `u128` | Big-endian bytes |
| `i8`, `i16`, `i32`, `i64`, `i128` | Offset binary (flip sign bit) |
| `f32`, `f64` | IEEE 754 with sign-aware bit flipping |
| `char` | Big-endian `u32` code point |
| `String` / `&str` | Sentinel-escaped with `0x00` terminator |
| `&[u8]` | Sentinel-escaped with `0x7F` terminator |
| `Option<T>` | `0x00` for `None`, `0x01` + value for `Some` |
| `Vec<T>`, sequences | `0x01` + element per entry, `0x00` terminator |
| Maps | `0x01` + key + value per entry, `0x00` terminator |
| Tuples, structs | Fields concatenated in order (fixed-length) |
| Enums | `u32` variant index + variant data |
| `()`, unit structs | Zero bytes |

## Encoding Details

### Integers

Unsigned integers are stored in big-endian byte order. Signed integers are converted to unsigned via offset binary (`value.wrapping_add(2^(bits-1))`) then stored big-endian. This ensures negative values sort before positive values.

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
