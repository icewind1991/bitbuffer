#![allow(dead_code)]
#![allow(unreachable_patterns)]

use bitbuffer::{
    BigEndian, BitReadBuffer, BitReadSized, BitReadStream, BitWriteStream, Endianness, LittleEndian,
};
use bitbuffer_derive::{BitRead, BitWrite, BitWriteSized};

#[derive(BitWrite, PartialEq, Debug)]
struct TestStruct {
    foo: u8,
    str: String,
    #[size = 2]
    truncated: String,
    bar: u16,
    float: f32,
    #[size = 3]
    asd: u8,
    #[size = "asd"]
    previous_field: u8,
}

#[test]
fn test_read_struct() {
    let float: [u8; 4] = 12.5f32.to_bits().to_le_bytes();
    let bytes = vec![
        12,
        'h' as u8,
        'e' as u8,
        'l' as u8,
        'l' as u8,
        'o' as u8,
        0,
        'f' as u8,
        'o' as u8,
        'o' as u8,
        0,
        float[0],
        float[1],
        float[2],
        float[3],
        0b1010_0101,
    ];
    let val = TestStruct {
        foo: 12,
        str: "hello".to_owned(),
        truncated: "fo".to_owned(),
        bar: 'o' as u16,
        float: 12.5,
        asd: 0b101,
        previous_field: 0b1010_0,
    };
    let mut data = Vec::new();
    let mut stream = BitWriteStream::new(&mut data, LittleEndian);
    stream.write(&val).unwrap();
    assert_eq!(bytes, data);
}

#[derive(BitWrite, PartialEq, Debug)]
#[discriminant_bits = 2]
enum TestBareEnum {
    Foo,
    Bar,
    Asd = 3,
}

#[test]
fn test_read_bare_enum() {
    let bytes = vec![0b1100_0100];
    let mut data = Vec::new();
    let mut stream = BitWriteStream::new(&mut data, BigEndian);
    stream.write(&TestBareEnum::Asd).unwrap();
    stream.write(&TestBareEnum::Foo).unwrap();
    stream.write(&TestBareEnum::Bar).unwrap();

    assert_eq!(bytes, data);
}

#[derive(BitWrite, BitRead, PartialEq, Debug)]
#[discriminant_bits = 2]
enum TestUnnamedFieldEnum {
    #[size = 5]
    Foo(i8),
    Bar(bool),
    #[discriminant = 3]
    Asd(u8),
}

#[test]
fn test_read_unnamed_field_enum() {
    let bytes = vec![0b1100_0110, 0b1000_0110, 0b1011_0000];
    let mut data = Vec::new();
    let mut stream = BitWriteStream::new(&mut data, BigEndian);
    stream
        .write(&TestUnnamedFieldEnum::Asd(0b_00_0110_10))
        .unwrap();
    assert_eq!(10, stream.bit_len());

    stream.write(&TestUnnamedFieldEnum::Foo(0b0110_1)).unwrap();
    assert_eq!(17, stream.bit_len());

    stream.write(&TestUnnamedFieldEnum::Bar(true)).unwrap();

    let mut read = BitReadStream::<BigEndian>::from(data.as_slice());

    assert_eq!(
        TestUnnamedFieldEnum::Asd(0b_00_0110_10),
        read.read().unwrap()
    );
    assert_eq!(TestUnnamedFieldEnum::Foo(0b11_0_1), read.read().unwrap());
    assert_eq!(TestUnnamedFieldEnum::Bar(true), read.read().unwrap());

    assert_eq!(bytes, data);
}

#[derive(BitWriteSized, BitReadSized, PartialEq, Debug)]
struct TestStructSized {
    foo: u8,
    #[size = "input_size"]
    string: String,
    #[size = "input_size"]
    int: u8,
}

#[test]
fn test_read_struct_sized() {
    let bytes = vec![12, 'h' as u8, 'e' as u8, 'l' as u8, 0b1000_0000];
    let mut data = Vec::new();
    let mut stream = BitWriteStream::new(&mut data, BigEndian);
    let val = TestStructSized {
        foo: 12,
        string: "hel".to_owned(),
        int: 4,
    };
    stream.write_sized(&val, 3).unwrap();
    let mut read = BitReadStream::<BigEndian>::from(data.as_slice());

    assert_eq!(val, read.read_sized(3).unwrap());

    assert_eq!(bytes, data);
}

#[derive(BitWriteSized, PartialEq, Debug)]
#[discriminant_bits = 2]
enum TestUnnamedFieldEnumSized {
    #[size = 5]
    Foo(i8),
    Bar(bool),
    #[discriminant = 3]
    #[size = "input_size"]
    Asd(u8),
}

#[test]
fn test_read_unnamed_field_enum_sized() {
    let bytes = vec![0b1100_0110];
    let mut data = Vec::new();
    let mut stream = BitWriteStream::new(&mut data, BigEndian);
    stream
        .write_sized(&TestUnnamedFieldEnumSized::Asd(0b_00_0110), 6)
        .unwrap();
    assert_eq!(bytes, data);
}

#[derive(BitWrite, PartialEq, Debug)]
struct TestStruct2 {
    size: u8,
    #[size = "size * 2"]
    str: String,
}

#[test]
fn test_read_struct2() {
    let bytes = vec![
        0b0000_0101,
        'h' as u8,
        'e' as u8,
        'l' as u8,
        'l' as u8,
        'o' as u8,
        ' ' as u8,
        'w' as u8,
        'o' as u8,
        'r' as u8,
        'l' as u8,
    ];
    let mut data = Vec::new();
    let mut stream = BitWriteStream::new(&mut data, BigEndian);
    stream
        .write(&TestStruct2 {
            size: 5,
            str: "hello worl".to_owned(),
        })
        .unwrap();
    assert_eq!(bytes, data);
}

#[derive(BitWrite)]
#[endianness = "E"]
struct TestStruct3<'a, E: Endianness> {
    size: u8,
    #[size = "size"]
    stream: BitReadStream<'a, E>,
}

#[test]
fn test_read_struct3() {
    let bytes = vec![0b0000_0101, 0b1010_1000];
    let mut data = Vec::new();
    let mut stream = BitWriteStream::new(&mut data, BigEndian);
    let mut inner = BitReadStream::from(BitReadBuffer::new(&[0b1010_1010], BigEndian));

    let inner = inner.read_bits(5).unwrap();

    let val: TestStruct3<BigEndian> = TestStruct3 {
        size: 5,
        stream: inner,
    };
    stream.write(&val).unwrap();
    assert_eq!(bytes, data);
}

#[derive(BitWrite, PartialEq, Debug)]
#[discriminant_bits = 2]
enum TestEnumRest {
    Foo,
    Bar,
    #[discriminant = "_"]
    Asd,
}

#[test]
fn test_read_rest_enum() {
    let bytes = vec![0b1000_0110];
    let mut data = Vec::new();
    let mut stream = BitWriteStream::new(&mut data, BigEndian);

    stream.write(&TestEnumRest::Asd).unwrap();
    stream.write(&TestEnumRest::Foo).unwrap();
    stream.write(&TestEnumRest::Bar).unwrap();
    stream.write(&TestEnumRest::Asd).unwrap();

    assert_eq!(bytes, data);
}

#[derive(BitWrite, PartialEq, Debug)]
struct UnnamedSize(u8, #[size = 5] String, bool);

fn test_unnamed_struct() {
    let bytes = vec![
        12, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0, 0, 0, 0, 0, 0,
    ];
    let mut data = Vec::new();
    let mut stream = BitWriteStream::new(&mut data, LittleEndian);
    stream
        .write(&UnnamedSize(12, "hello".to_string(), false))
        .unwrap();

    assert_eq!(bytes, data);
}

#[derive(BitWrite, PartialEq, Debug)]
struct EmptyStruct;

fn test_empty_struct() {
    let mut data = Vec::new();
    let mut stream = BitWriteStream::new(&mut data, LittleEndian);
    stream.write(&EmptyStruct).unwrap();
    assert_eq!(Vec::<u8>::new(), data);
}

#[derive(BitWrite)]
struct TestSizeExpression {
    size: u8,
    #[size = "size + 2"]
    str: String,
}

#[test]
fn test_read_size_expression() {
    let bytes = vec![0b0000_0011, b'a', b'b', b'c', b'd', b'e'];
    let mut data = Vec::new();
    let mut stream = BitWriteStream::new(&mut data, BigEndian);

    let val = TestSizeExpression {
        size: 3,
        str: String::from("abcde"),
    };
    stream.write(&val).unwrap();
    assert_eq!(bytes, data);
}

#[derive(BitWrite, PartialEq, Debug)]
#[align]
struct AlignStruct(u8);

#[test]
fn test_align() {
    let bytes = vec![0, 0x80];
    let mut data = Vec::new();
    let mut stream = BitWriteStream::new(&mut data, BigEndian);
    stream.write_bool(false).unwrap();
    let val = AlignStruct(0x80);
    stream.write(&val).unwrap();
    assert_eq!(bytes, data);
}

#[derive(BitWrite, PartialEq, Debug)]
#[align]
struct AlignFieldStruct {
    #[size = 1]
    foo: u8,
    #[align]
    bar: u8,
}

#[test]
fn test_align_field() {
    let bytes = vec![0, 0x80];
    let mut data = Vec::new();
    let mut stream = BitWriteStream::new(&mut data, BigEndian);
    let val = AlignFieldStruct { foo: 0, bar: 0x80 };
    stream.write(&val).unwrap();
    assert_eq!(bytes, data);
}

#[derive(BitWrite, PartialEq, Debug)]
#[discriminant_bits = 4]
#[align]
enum AlignEnum {
    Foo,
    Bar(u8),
}

#[test]
fn test_align_enum() {
    let bytes = vec![0x00, 0x18, 0];
    let mut data = Vec::new();
    let mut stream = BitWriteStream::new(&mut data, BigEndian);
    stream.write_bool(false).unwrap();
    let val = AlignEnum::Bar(0x80);
    stream.write(&val).unwrap();
    assert_eq!(bytes, data);
}

#[derive(BitWrite, PartialEq, Debug)]
#[discriminant_bits = 4]
#[align]
enum AlignEnumField {
    Foo,
    #[align]
    Bar(u8),
}

#[test]
fn test_align_enum_field() {
    let bytes = vec![0x00, 0x10, 0x80];
    let mut data = Vec::new();
    let mut stream = BitWriteStream::new(&mut data, BigEndian);
    stream.write_bool(false).unwrap();
    let val = AlignEnumField::Bar(0x80);
    stream.write(&val).unwrap();
    assert_eq!(bytes, data);
}
