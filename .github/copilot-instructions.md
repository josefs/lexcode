# Copilot Instructions

## Build & Test

```sh
cargo build
cargo test                          # full suite (includes proptest)
cargo test prop_u32                 # run a single test by name
cargo test -- --nocapture           # show stdout in test output
```

## Architecture

lexcode is a serde-based binary serialization format that preserves lexicographic ordering — when `a < b`, the serialized bytes satisfy `bytes(a) < bytes(b)` under byte-wise comparison.

The crate implements serde's `Serializer` and `Deserializer` traits across four modules:

- **`ser.rs`** — `Serializer` writes into a `Vec<u8>`. Fixed-size types (integers, floats, chars) are encoded inline; variable-length types (strings, byte slices, sequences, maps) use sentinel-based framing.
- **`de.rs`** — `Deserializer` reads from a `&[u8]` slice, advancing a cursor. Uses three accessor helpers: `SeqAccessor` (variable-length sequences with `0x01`/`0x00` framing), `FixedLenAccessor` (tuples/structs with known field count), and `MapAccessor`.
- **`error.rs`** — Shared `Error` enum (`Message`, `Eof`, `TrailingCharacters`) implementing both `serde::ser::Error` and `serde::de::Error`.
- **`lib.rs`** — Re-exports the public API: `to_bytes`, `from_bytes`, `Serializer`, `Deserializer`, `Error`, `Result`.

## Key Conventions

- **Encoding invariant**: every encoding must preserve lexicographic order. Signed integers use offset binary (flip sign bit). Floats use sign-aware bit flipping. Variable-length data uses sentinel encoding (sentinel byte doubled as escape, terminated with `sentinel + 0x00`).
- **Strings vs bytes**: strings use sentinel `0x00`; byte slices use sentinel `0x7F`.
- **Sequences/maps** prefix each element with `0x01` and end with `0x00`. Tuples and structs concatenate fields directly (no framing) since their length is known at compile time.
- **`deserialize_any` is intentionally unsupported** — the format is not self-describing.
- **Testing**: all types are verified with proptest using two macro-generated test families in `tests/prop_test.rs`: `roundtripping_test!` (serialize → deserialize roundtrip) and `ordpreserving_test!` (ordering preservation). When adding a new supported type, add both a roundtrip and an order-preservation test.
- **Rust edition 2024** with `serde` derive for serialization.
