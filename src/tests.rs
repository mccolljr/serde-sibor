fn arbitrary_value<T>() -> T
where
    T: for<'x> ::arbitrary::Arbitrary<'x>,
{
    <T as ::arbitrary::Arbitrary>::arbitrary_take_rest(::arbitrary::Unstructured::new(
        &Vec::from_iter(std::iter::repeat_with(::rand::random::<u8>).take(1024)),
    ))
    .expect(&format!(
        "failed to generate arbitrary valueof type {}",
        std::any::type_name::<T>()
    ))
}

macro_rules! assert_round_trip {
    ($t:ty) => {
        assert_round_trip!(@DO_FUZZY $t);
    };

    ($t:ty, $($val:expr),*) => {
        $({
            let specific: $t = $val;
            assert_round_trip!(@DO_ASSERT $t, specific);
        })*
    };

    (@DO_ASSERT $t:ty, $given:ident) => {{
        let expected_size = crate::encoded_size(&$given).unwrap();
        let encoded_bytes = crate::to_bytes(&$given).unwrap();
        assert_eq!(expected_size, encoded_bytes.len());
        let decoded = crate::from_bytes::<$t>(&encoded_bytes).unwrap();
        assert_eq!($given, decoded);
    }};

    (@DO_FUZZY $t:ty) => {
        for _ in 0..1000 {
            let original: $t = arbitrary_value::<$t>();
            assert_round_trip!(@DO_ASSERT $t, original);
        }
    };
}

#[test]
fn test_primitive_round_trips() {
    assert_round_trip!(u8, u8::MIN, u8::MAX);
    assert_round_trip!(u8);

    assert_round_trip!(u16, u16::MIN, u16::MAX);
    assert_round_trip!(u16);

    assert_round_trip!(u32, u32::MIN, u32::MAX);
    assert_round_trip!(u32);

    assert_round_trip!(u64, u64::MIN, u64::MAX);
    assert_round_trip!(u64);

    assert_round_trip!(i8, i8::MIN, 0, i8::MAX);
    assert_round_trip!(i8);

    assert_round_trip!(i16, i16::MIN, 0, i16::MAX);
    assert_round_trip!(i16);

    assert_round_trip!(i32, i32::MIN, 0, i32::MAX);
    assert_round_trip!(i32);

    assert_round_trip!(i64, i64::MIN, 0, i64::MAX);
    assert_round_trip!(i64);

    assert_round_trip!(f32, f32::MIN, 0.0, f32::MAX);
    assert_round_trip!(f64, f64::MIN, 0.0, f64::MAX);
    assert_round_trip!(bool, true, false);

    assert_round_trip!(String, "".into());
    assert_round_trip!(String);
}

#[test]
fn test_primitive_vector_round_trips() {
    assert_round_trip!(Vec<u8>, vec![], vec![u8::MIN, u8::MAX]);
    assert_round_trip!(Vec<u16>, vec![], vec![u16::MIN, u16::MAX]);
    assert_round_trip!(Vec<u32>, vec![], vec![u32::MIN, u32::MAX]);
    assert_round_trip!(Vec<u64>, vec![], vec![u64::MIN, u64::MAX]);

    assert_round_trip!(Vec<i8>, vec![], vec![i8::MIN, 0, i8::MAX]);
    assert_round_trip!(Vec<i16>, vec![], vec![i16::MIN, 0, i16::MAX]);
    assert_round_trip!(Vec<i32>, vec![], vec![i32::MIN, 0, i32::MAX]);
    assert_round_trip!(Vec<i64>, vec![], vec![i64::MIN, 0, i64::MAX]);

    assert_round_trip!(Vec<f32>, vec![], vec![f32::MIN, 0.0, f32::MAX]);
    assert_round_trip!(Vec<f64>, vec![], vec![f64::MIN, 0.0, f64::MAX]);
    assert_round_trip!(Vec<bool>, vec![], vec![true, false]);
}

#[test]
fn test_unit_struct_round_trip() {
    #[derive(Debug, Clone, PartialEq, ::serde::Deserialize, ::serde::Serialize)]
    struct TestUnitStruct;
    assert_round_trip!(TestUnitStruct, TestUnitStruct);
}

#[test]
fn test_newtype_struct_round_trip() {
    #[derive(
        Debug, Clone, PartialEq, ::serde::Deserialize, ::serde::Serialize, ::arbitrary::Arbitrary,
    )]
    struct TestNewtypeStruct(i32);

    assert_round_trip!(TestNewtypeStruct);
}

#[test]
fn test_tuple_struct_round_trip() {
    #[derive(
        Debug, Clone, PartialEq, ::serde::Deserialize, ::serde::Serialize, ::arbitrary::Arbitrary,
    )]
    struct TestTupleStruct(i32, bool);

    assert_round_trip!(TestTupleStruct);
}

#[test]
fn test_normal_struct_round_trip() {
    #[derive(
        Debug, Clone, PartialEq, ::serde::Deserialize, ::serde::Serialize, ::arbitrary::Arbitrary,
    )]
    struct TestNormalStruct {
        x: i32,
        y: bool,
    }

    assert_round_trip!(TestNormalStruct);
}

#[test]
fn test_unit_enum_round_trip() {
    #[derive(
        Debug, Clone, PartialEq, ::serde::Deserialize, ::serde::Serialize, ::arbitrary::Arbitrary,
    )]
    enum TestUnitEnum {
        A,
        B,
        C,
    }

    assert_round_trip!(
        TestUnitEnum,
        TestUnitEnum::A,
        TestUnitEnum::B,
        TestUnitEnum::C
    );
    assert_round_trip!(TestUnitEnum);
}

#[test]
fn test_newtype_enum_round_trip() {
    #[derive(
        Debug, Clone, PartialEq, ::serde::Deserialize, ::serde::Serialize, ::arbitrary::Arbitrary,
    )]
    enum TestNewtypeEnum {
        A(String),
        B(bool),
        C(Vec<u32>),
    }

    assert_round_trip!(
        TestNewtypeEnum,
        TestNewtypeEnum::A("".into()),
        TestNewtypeEnum::B(false),
        TestNewtypeEnum::C(vec![])
    );
    assert_round_trip!(TestNewtypeEnum);
}

#[test]
fn test_tuple_enum_round_trip() {
    #[derive(
        Debug, Clone, PartialEq, ::serde::Deserialize, ::serde::Serialize, ::arbitrary::Arbitrary,
    )]
    enum TestTupleEnum {
        A(String, bool),
        B(bool, String),
        C(Vec<u32>, Vec<u8>),
    }

    assert_round_trip!(
        TestTupleEnum,
        TestTupleEnum::A("x".into(), false),
        TestTupleEnum::B(true, "y".into()),
        TestTupleEnum::C(vec![1, 2, 3], vec![1, 2, 3])
    );
    assert_round_trip!(TestTupleEnum);
}

#[test]
fn test_mixed_enum_round_trip() {
    #[derive(
        Debug, Clone, PartialEq, ::serde::Deserialize, ::serde::Serialize, ::arbitrary::Arbitrary,
    )]
    enum TestMixedEnum {
        A,
        B(i32),
        C(i32, u64),
        D { x: i8 },
    }

    assert_round_trip!(
        TestMixedEnum,
        TestMixedEnum::A,
        TestMixedEnum::B(-1),
        TestMixedEnum::C(-100, 100),
        TestMixedEnum::D { x: 120 }
    );
    assert_round_trip!(TestMixedEnum);
}

#[test]
fn test_bytes_round_trip() {
    #[derive(
        Debug, Clone, PartialEq, ::serde::Deserialize, ::serde::Serialize, ::arbitrary::Arbitrary,
    )]
    struct TestHasBytes {
        #[serde(with = "serde_bytes")]
        b: Vec<u8>,
    }

    assert_round_trip!(
        TestHasBytes,
        TestHasBytes { b: vec![] },
        TestHasBytes { b: vec![1, 2, 3] }
    );
    assert_round_trip!(TestHasBytes);
}

#[test]
fn test_deeply_nested_round_trips() {
    #[derive(
        Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::arbitrary::Arbitrary,
    )]
    enum TestEnum {
        A,
        B(TestObjX),
        C { y: TestObjY },
    }
    #[derive(
        Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::arbitrary::Arbitrary,
    )]
    pub struct TestObjX {
        a: (i8, i16, i32, i64),
        b: (u8, u16, u32, u64),
        d: bool,
    }

    #[derive(
        Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::arbitrary::Arbitrary,
    )]
    pub struct TestObjY {
        e: String,
        #[serde(with = "serde_bytes")]
        f: Vec<u8>,
        g: Vec<TestEnum>,
    }

    for _ in 0..10 {
        assert_round_trip!(TestEnum);
    }
}
