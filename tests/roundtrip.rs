use bitbuffer::{
    BigEndian, BitRead, BitReadBuffer, BitReadStream, BitWrite, BitWriteStream, LittleEndian,
};
use std::fmt::Debug;

#[track_caller]
fn roundtrip<
    T: BitRead<'static, BigEndian>
        + BitWrite<BigEndian>
        + BitRead<'static, LittleEndian>
        + BitWrite<LittleEndian>
        + Debug
        + PartialEq,
>(
    val: T,
) {
    {
        let mut data = Vec::new();
        let size = {
            let mut stream = BitWriteStream::new(&mut data, LittleEndian);
            stream.write(&val).unwrap();
            stream.bit_len()
        };
        let mut read = BitReadStream::new(BitReadBuffer::new_owned(data, LittleEndian));
        assert_eq!(val, read.read().unwrap());
        assert_eq!(size, read.pos());
    }
    {
        let mut data = Vec::new();
        let size = {
            let mut stream = BitWriteStream::new(&mut data, BigEndian);
            stream.write(&val).unwrap();
            stream.bit_len()
        };
        let mut read = BitReadStream::new(BitReadBuffer::new_owned(data, BigEndian));
        assert_eq!(val, read.read().unwrap());
        assert_eq!(size, read.pos());
    }
}

#[test]
fn test_basic_struct() {
    #[derive(Debug, PartialEq, BitRead, BitWrite)]
    struct Foo {
        int: u32,
        float: f64,
        #[size = 2]
        smaller_int: u8,
        signed: i32,
        #[size = 3]
        smaller_signed: i32,
        dynamic_string: String,
        #[size = 3]
        fixed_string: String,
    }
    roundtrip(Foo {
        int: 1234,
        float: 10.2,
        smaller_int: 3,
        signed: -3,
        smaller_signed: -1,
        dynamic_string: "Foobar".to_string(),
        fixed_string: "asd".to_string(),
    });
}

#[test]
fn test_bare_enum() {
    #[derive(Debug, PartialEq, BitRead, BitWrite)]
    #[discriminant_bits = 4]
    enum Enum {
        A,
        B,
        C,
        D,
    }
    roundtrip(Enum::A);
    roundtrip(Enum::B);
    roundtrip(Enum::C);
    roundtrip(Enum::D);
}

#[test]
fn test_field_enum() {
    #[derive(Debug, PartialEq, BitRead, BitWrite)]
    struct CompoundVariant(
        #[size = 15]
        u16,
        bool,
    );

    #[derive(Debug, PartialEq, BitRead, BitWrite)]
    #[discriminant_bits = 4]
    enum Enum {
        A,
        B(String),
        C(f32),
        D(#[size = 15] i64),
        E(CompoundVariant),
    }
    roundtrip(Enum::A);
    roundtrip(Enum::B("foobar".into()));
    roundtrip(Enum::C(12.0));
    roundtrip(Enum::D(-12345));
    roundtrip(Enum::E(CompoundVariant(6789, true)));
}

#[test]
fn test_array() {
    roundtrip([1, 2, 3, 4, 5]);
    roundtrip([String::from("asd"), String::from("foobar")]);
}

#[test]
fn test_tuple() {
    roundtrip((1, false));
    roundtrip((1, 10.12, String::from("asd")));
}
