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
  ordpreserving_test!(prop_ord_u8,  u8, true);
  ordpreserving_test!(prop_ord_u16, u16, true);
  ordpreserving_test!(prop_ord_u32, u32, true);
  ordpreserving_test!(prop_ord_u64, u64, true);
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
}