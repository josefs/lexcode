#[cfg(test)]
mod prop_tests {
  use proptest::prelude::*;
  use proptest_derive::Arbitrary;
  use serde::Deserialize;
  use serde::Serialize;

  #[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Arbitrary)]
  struct NewtypeStruct(i64);

  #[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Arbitrary)]
  struct TupleStructI64I64(i64, i64);

  #[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Arbitrary)]
  enum E {
    V(i64, i64)
  }

  #[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Arbitrary)]
  struct Struct {
    a: u8,
    b: String,
    c: Vec<i64>,
  }

  macro_rules! roundtripping_test {
      ($name:ident, $ty:ty) => {
          proptest! {
              #[test]
              fn $name(data in any::<$ty>()) {
                  let new_data = lexcode::to_bytes(&data).and_then(|ref bytes|
                      lexcode::from_bytes::<$ty>(bytes.as_ref()))?;
                  prop_assert_eq!(data, new_data);
              }
          }
      };
  }

  roundtripping_test!(prop_u8, u8);
  roundtripping_test!(prop_u16, u16);
  roundtripping_test!(prop_u32, u32);
  roundtripping_test!(prop_u64, u64);
  roundtripping_test!(prop_u128, u128);
  roundtripping_test!(prop_i8, i8);
  roundtripping_test!(prop_i16, i16);
  roundtripping_test!(prop_i32, i32);
  roundtripping_test!(prop_i64, i64);
  roundtripping_test!(prop_i128, i128);
  roundtripping_test!(prop_string, String);
  roundtripping_test!(prop_f32, f32);
  roundtripping_test!(prop_f64, f64);
  roundtripping_test!(prop_char, char);
  roundtripping_test!(prop_option_i64, Option<i64>);
  roundtripping_test!(prop_unit, ());
  roundtripping_test!(prop_newtype_struct, NewtypeStruct);
  roundtripping_test!(prop_tuple_i64_i64, (i64, i64));
  roundtripping_test!(prop_tuple_struct_i64_i64, TupleStructI64I64);
  roundtripping_test!(prop_enum_e, E);
  roundtripping_test!(prop_vec_i64, Vec<i64>);
  roundtripping_test!(prop_map, std::collections::BTreeMap<String, i64>);
  roundtripping_test!(prop_struct, Struct);

  macro_rules! ordpreserving_test {
      ($name:ident, $ty:ty, $eq:expr) => {
          proptest! {
              #[test]
              fn $name(data1 in any::<$ty>(), data2 in any::<$ty>()) {
                  let bytes1 = lexcode::to_bytes(&data1)?;
                  let bytes2 = lexcode::to_bytes(&data2)?;
                  if data1 < data2 {
                      prop_assert!(bytes1 < bytes2);
                  } else if data1 > data2 {
                      prop_assert!(bytes1 > bytes2);
                  } else if $eq {
                      prop_assert_eq!(bytes1, bytes2);
                  } else {
                      // We don't test for equality because of e.g. -0 and +0.
                  }
              }
          }

      };
  }

  ordpreserving_test!(prop_ord_i8,  i8, true);
  ordpreserving_test!(prop_ord_i16, i16, true);
  ordpreserving_test!(prop_ord_i32, i32, true);
  ordpreserving_test!(prop_ord_i64, i64, true);
  ordpreserving_test!(prop_ord_i128, i128, true);
  ordpreserving_test!(prop_ord_u8,  u8, true);
  ordpreserving_test!(prop_ord_u16, u16, true);
  ordpreserving_test!(prop_ord_u32, u32, true);
  ordpreserving_test!(prop_ord_u64, u64, true);
  ordpreserving_test!(prop_ord_u128, u128, true);
  ordpreserving_test!(prop_ord_f32, f32, false);
  ordpreserving_test!(prop_ord_f64, f64, false);
  ordpreserving_test!(prop_ord_char, char, true);
  ordpreserving_test!(prop_ord_string, String, true);
  ordpreserving_test!(prop_ord_option_i64, Option<i64>, true);
  ordpreserving_test!(prop_ord_newtype_struct, NewtypeStruct, true);
  ordpreserving_test!(prop_ord_tuple_struct_i64_i64, TupleStructI64I64, true);
  ordpreserving_test!(prop_ord_enum_e, E, true);
  ordpreserving_test!(prop_ord_struct, Struct, true);
  ordpreserving_test!(prop_ord_map, std::collections::BTreeMap<String, i64>, true);
  ordpreserving_test!(prop_ord_unit, (), true);
  ordpreserving_test!(prop_ord_tuple_i64_i64, (i64, i64), true);
  ordpreserving_test!(prop_ord_vec_i64, Vec<i64>, true);
  ordpreserving_test!(prop_ord_vec_string, Vec<String>, true);

  macro_rules! overflow_test {
      ($name:ident, $ser_ty:ty, $de_ty:ty, $val:expr) => {
          #[test]
          fn $name() {
              let bytes = lexcode::to_bytes(&($val as $ser_ty)).unwrap();
              let result = lexcode::from_bytes::<$de_ty>(&bytes);
              assert!(result.is_err(), "expected overflow error deserializing {} as {}",
                  stringify!($ser_ty), stringify!($de_ty));
          }
      };
  }

  // Unsigned overflow
  overflow_test!(overflow_u16_as_u8, u16, u8, 256u16);
  overflow_test!(overflow_u32_as_u8, u32, u8, 1000u32);
  overflow_test!(overflow_u32_as_u16, u32, u16, 70000u32);
  overflow_test!(overflow_u64_as_u32, u64, u32, u32::MAX as u64 + 1);
  overflow_test!(overflow_u128_as_u64, u128, u64, u64::MAX as u128 + 1);

  // Signed overflow
  overflow_test!(overflow_i16_as_i8_pos, i16, i8, 128i16);
  overflow_test!(overflow_i16_as_i8_neg, i16, i8, -129i16);
  overflow_test!(overflow_i32_as_i16_pos, i32, i16, 32768i32);
  overflow_test!(overflow_i32_as_i16_neg, i32, i16, -32769i32);
  overflow_test!(overflow_i64_as_i32, i64, i32, i32::MAX as i64 + 1);
  overflow_test!(overflow_i128_as_i64, i128, i64, i64::MAX as i128 + 1);

  // Cross-width encoding equivalence: encoding a value at different integer
  // widths produces identical bytes.
  macro_rules! cross_width_unsigned_test {
      ($name:ident, $small:ty, $large:ty) => {
          proptest! {
              #[test]
              fn $name(val in any::<$small>()) {
                  let bytes_small = lexcode::to_bytes(&val).unwrap();
                  let bytes_large = lexcode::to_bytes(&(val as $large)).unwrap();
                  prop_assert_eq!(bytes_small, bytes_large,
                      "encoding of {} as {} vs {} differ", val, stringify!($small), stringify!($large));
              }
          }
      };
  }

  macro_rules! cross_width_signed_test {
      ($name:ident, $small:ty, $large:ty) => {
          proptest! {
              #[test]
              fn $name(val in any::<$small>()) {
                  let bytes_small = lexcode::to_bytes(&val).unwrap();
                  let bytes_large = lexcode::to_bytes(&(val as $large)).unwrap();
                  prop_assert_eq!(bytes_small, bytes_large,
                      "encoding of {} as {} vs {} differ", val, stringify!($small), stringify!($large));
              }
          }
      };
  }

  cross_width_unsigned_test!(cross_width_u8_u16, u8, u16);
  cross_width_unsigned_test!(cross_width_u8_u32, u8, u32);
  cross_width_unsigned_test!(cross_width_u8_u64, u8, u64);
  cross_width_unsigned_test!(cross_width_u8_u128, u8, u128);
  cross_width_unsigned_test!(cross_width_u16_u32, u16, u32);
  cross_width_unsigned_test!(cross_width_u16_u64, u16, u64);
  cross_width_unsigned_test!(cross_width_u16_u128, u16, u128);
  cross_width_unsigned_test!(cross_width_u32_u64, u32, u64);
  cross_width_unsigned_test!(cross_width_u32_u128, u32, u128);
  cross_width_unsigned_test!(cross_width_u64_u128, u64, u128);

  cross_width_signed_test!(cross_width_i8_i16, i8, i16);
  cross_width_signed_test!(cross_width_i8_i32, i8, i32);
  cross_width_signed_test!(cross_width_i8_i64, i8, i64);
  cross_width_signed_test!(cross_width_i8_i128, i8, i128);
  cross_width_signed_test!(cross_width_i16_i32, i16, i32);
  cross_width_signed_test!(cross_width_i16_i64, i16, i64);
  cross_width_signed_test!(cross_width_i16_i128, i16, i128);
  cross_width_signed_test!(cross_width_i32_i64, i32, i64);
  cross_width_signed_test!(cross_width_i32_i128, i32, i128);
  cross_width_signed_test!(cross_width_i64_i128, i64, i128);

  // Cross-width ordering: encoding values at different widths preserves
  // numerical ordering. E.g. encoding 8u8 and 200u16 should compare the
  // same way as comparing both values widened to u16.
  macro_rules! cross_width_ord_unsigned_test {
      ($name:ident, $small:ty, $large:ty) => {
          proptest! {
              #[test]
              fn $name(a in any::<$small>(), b in any::<$large>()) {
                  let bytes_a = lexcode::to_bytes(&a).unwrap();
                  let bytes_b = lexcode::to_bytes(&b).unwrap();
                  let a_wide = a as $large;
                  if a_wide < b {
                      prop_assert!(bytes_a < bytes_b,
                          "{} as {} < {} as {}, but bytes {:?} >= {:?}",
                          a, stringify!($small), b, stringify!($large), bytes_a, bytes_b);
                  } else if a_wide > b {
                      prop_assert!(bytes_a > bytes_b,
                          "{} as {} > {} as {}, but bytes {:?} <= {:?}",
                          a, stringify!($small), b, stringify!($large), bytes_a, bytes_b);
                  } else {
                      prop_assert_eq!(bytes_a, bytes_b);
                  }
              }
          }
      };
  }

  macro_rules! cross_width_ord_signed_test {
      ($name:ident, $small:ty, $large:ty) => {
          proptest! {
              #[test]
              fn $name(a in any::<$small>(), b in any::<$large>()) {
                  let bytes_a = lexcode::to_bytes(&a).unwrap();
                  let bytes_b = lexcode::to_bytes(&b).unwrap();
                  let a_wide = a as $large;
                  if a_wide < b {
                      prop_assert!(bytes_a < bytes_b,
                          "{} as {} < {} as {}, but bytes {:?} >= {:?}",
                          a, stringify!($small), b, stringify!($large), bytes_a, bytes_b);
                  } else if a_wide > b {
                      prop_assert!(bytes_a > bytes_b,
                          "{} as {} > {} as {}, but bytes {:?} <= {:?}",
                          a, stringify!($small), b, stringify!($large), bytes_a, bytes_b);
                  } else {
                      prop_assert_eq!(bytes_a, bytes_b);
                  }
              }
          }
      };
  }

  cross_width_ord_unsigned_test!(cross_width_ord_u8_u16, u8, u16);
  cross_width_ord_unsigned_test!(cross_width_ord_u8_u32, u8, u32);
  cross_width_ord_unsigned_test!(cross_width_ord_u8_u64, u8, u64);
  cross_width_ord_unsigned_test!(cross_width_ord_u8_u128, u8, u128);
  cross_width_ord_unsigned_test!(cross_width_ord_u16_u32, u16, u32);
  cross_width_ord_unsigned_test!(cross_width_ord_u16_u64, u16, u64);
  cross_width_ord_unsigned_test!(cross_width_ord_u16_u128, u16, u128);
  cross_width_ord_unsigned_test!(cross_width_ord_u32_u64, u32, u64);
  cross_width_ord_unsigned_test!(cross_width_ord_u32_u128, u32, u128);
  cross_width_ord_unsigned_test!(cross_width_ord_u64_u128, u64, u128);

  cross_width_ord_signed_test!(cross_width_ord_i8_i16, i8, i16);
  cross_width_ord_signed_test!(cross_width_ord_i8_i32, i8, i32);
  cross_width_ord_signed_test!(cross_width_ord_i8_i64, i8, i64);
  cross_width_ord_signed_test!(cross_width_ord_i8_i128, i8, i128);
  cross_width_ord_signed_test!(cross_width_ord_i16_i32, i16, i32);
  cross_width_ord_signed_test!(cross_width_ord_i16_i64, i16, i64);
  cross_width_ord_signed_test!(cross_width_ord_i16_i128, i16, i128);
  cross_width_ord_signed_test!(cross_width_ord_i32_i64, i32, i64);
  cross_width_ord_signed_test!(cross_width_ord_i32_i128, i32, i128);
  cross_width_ord_signed_test!(cross_width_ord_i64_i128, i64, i128);

  // FixedBytes tests
  use lexcode::FixedBytes;

  proptest! {
      #[test]
      fn prop_fixed_bytes_32(data in any::<[u8; 32]>()) {
          let fb = FixedBytes(data);
          let bytes = lexcode::to_bytes(&fb).unwrap();
          prop_assert_eq!(bytes.len(), 32, "FixedBytes<32> should encode to exactly 32 bytes");
          let decoded: FixedBytes<32> = lexcode::from_bytes(&bytes).unwrap();
          prop_assert_eq!(fb, decoded);
      }

      #[test]
      fn prop_ord_fixed_bytes_32(
          a in any::<[u8; 32]>(),
          b in any::<[u8; 32]>()
      ) {
          let fa = FixedBytes(a);
          let fb = FixedBytes(b);
          let bytes_a = lexcode::to_bytes(&fa).unwrap();
          let bytes_b = lexcode::to_bytes(&fb).unwrap();
          if fa < fb {
              prop_assert!(bytes_a < bytes_b);
          } else if fa > fb {
              prop_assert!(bytes_a > bytes_b);
          } else {
              prop_assert_eq!(bytes_a, bytes_b);
          }
      }
  }

  #[test]
  fn fixed_bytes_zero_overhead() {
      let fb = FixedBytes([0xDE, 0xAD, 0xBE, 0xEF]);
      let bytes = lexcode::to_bytes(&fb).unwrap();
      assert_eq!(bytes, vec![0xDE, 0xAD, 0xBE, 0xEF]);
  }

  #[test]
  fn fixed_bytes_identity_encoding() {
      // Every byte value 0x00..=0xFF should be stored as-is
      let mut arr = [0u8; 256];
      for i in 0..256 {
          arr[i] = i as u8;
      }
      let fb = FixedBytes(arr);
      let bytes = lexcode::to_bytes(&fb).unwrap();
      assert_eq!(bytes.len(), 256);
      assert_eq!(&bytes[..], &arr[..]);
  }
}