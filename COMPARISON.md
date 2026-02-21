# Comparison with Similar Crates

Several Rust crates provide order-preserving binary serialization. This document compares lexcode with the main alternatives.

## Overview

| Feature | lexcode | bytekey2 | ordcode | memcomparable |
|---|---|---|---|---|
| Order-preserving | ✅ | ✅ | ✅ | ✅ |
| Variable-length integers | ✅ | ❌ | ✅ | ❌ |
| Cross-width integer compatibility | ✅ | ❌ | ❌ | ❌ |
| Descending sort order | ❌ | ❌ | ✅ | ✅ |
| `deserialize_any` support | ❌ | ❌ | ✅ (opt-in) | ❌ |
| Zero-overhead fixed bytes | ✅ (`FixedBytes`) | ❌ | ❌ | ❌ |
| Serde-based | ✅ | ✅ | ✅ | ✅ |

## [bytekey2](https://crates.io/crates/bytekey2)

A fork of the original `bytekey` crate.

- **Integer encoding**: Fixed-width big-endian. Every `u64` uses 8 bytes regardless of value. Signed integers use offset binary (flip sign bit).
- **Strings**: Sentinel encoding, similar to lexcode.
- **No cross-width compatibility**: `5u16` and `5u64` produce different byte sequences, so changing the integer width of a field is a breaking change.
- **Maintenance**: Largely unmaintained.

**When to choose bytekey2**: If you need a simple, well-known encoding and don't care about compactness or cross-width compatibility.

## [ordcode](https://crates.io/crates/ordcode)

The most feature-rich order-preserving format.

- **Integer encoding**: Variable-length, similar philosophy to lexcode.
- **Descending order**: Supports encoding values in reverse order for descending sorts via a separate API.
- **Self-describing mode**: Optionally supports `deserialize_any`, allowing dynamically-typed deserialization at the cost of additional overhead.
- **No cross-width compatibility**: Integer width is part of the encoding, so changing a field from `u16` to `u32` is a breaking change.

**When to choose ordcode**: If you need descending sort order or `deserialize_any` support.

## [memcomparable](https://crates.io/crates/memcomparable)

Based on the encoding schemes used by TiDB and CockroachDB.

- **Integer encoding**: Fixed-width, using 8-byte groups with padding. Optimized for database key encoding rather than compactness.
- **Descending order**: Supports reverse encoding.
- **No variable-length integers**: Every integer uses a fixed number of bytes.
- **No cross-width compatibility**: Encoding depends on the Rust type width.

**When to choose memcomparable**: If you're building a database engine and want an encoding that closely matches what TiDB/CockroachDB use internally.

## Non-order-preserving alternatives

These crates are popular for binary serialization but do **not** preserve lexicographic ordering:

- **[bincode](https://crates.io/crates/bincode)**: Fast, compact, widely used. Uses little-endian fixed-width encoding.
- **[postcard](https://crates.io/crates/postcard)**: Designed for `#![no_std]` and embedded. Uses varint encoding but not in an order-preserving way.

## lexcode's distinguishing features

- **Cross-width integer compatibility**: Encoding a value is independent of the Rust integer width. `42u8`, `42u16`, and `42u64` all produce identical bytes. This means you can widen integer fields without breaking existing encoded data.
- **Variable-length integers**: Small values use fewer bytes (e.g., values 0–127 use 1 byte).
- **`FixedBytes<N>`**: A zero-overhead wrapper type for fixed-size byte arrays (hashes, UUIDs, etc.) that writes exactly N raw bytes with no framing or escaping.
