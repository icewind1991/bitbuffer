#![allow(dead_code)]
#![allow(unreachable_patterns)]

use bitbuffer::{
    bit_size_of, bit_size_of_sized, BigEndian, BitReadBuffer, BitReadStream, Endianness,
    LittleEndian,
};
use bitbuffer_derive::{BitRead, BitReadSized};

#[derive(BitRead, PartialEq, Debug)]
struct TestStruct {
    foo: u8,
    str: String,
    #[size = 2]
    truncated: String,
    bar: u16,
    float: f32,
    #[size = 3]
    asd: u8,
    #[size_bits = 2]
    dynamic: u8,
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
        0b0101_0101,
        0b1010_1010,
    ];
    let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    let mut stream = BitReadStream::from(buffer);
    assert_eq!(
        TestStruct {
            foo: 12,
            str: "hello".to_owned(),
            truncated: "fo".to_owned(),
            bar: 'o' as u16,
            float: 12.5,
            asd: 0b101,
            dynamic: 0b10,
            previous_field: 0b1010_0,
        },
        stream.read().unwrap()
    );
    assert_eq!(None, bit_size_of::<TestStruct>());
}

#[derive(BitRead, PartialEq, Debug)]
#[discriminant_bits = 2]
enum TestBareEnum {
    Foo,
    Bar,
    Asd = 3,
}

#[test]
fn test_read_bare_enum() {
    let bytes = vec![
        0b1100_0110,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
    ];
    let buffer = BitReadBuffer::new(&bytes, BigEndian);
    let mut stream = BitReadStream::from(buffer);
    assert_eq!(TestBareEnum::Asd, stream.read().unwrap());
    assert_eq!(TestBareEnum::Foo, stream.read().unwrap());
    assert_eq!(TestBareEnum::Bar, stream.read().unwrap());
    assert_eq!(true, stream.read::<TestBareEnum>().is_err());
    assert_eq!(Some(2), bit_size_of::<TestBareEnum>());
}

#[derive(BitRead, PartialEq, Debug)]
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
    let bytes = vec![
        0b1100_0110,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
    ];
    let buffer = BitReadBuffer::new(&bytes, BigEndian);
    let mut stream = BitReadStream::from(buffer);
    assert_eq!(
        TestUnnamedFieldEnum::Asd(0b_00_0110_10),
        stream.read().unwrap()
    );
    assert_eq!(10, stream.pos());
    stream.set_pos(2).unwrap();
    assert_eq!(TestUnnamedFieldEnum::Foo(0b11_0_1), stream.read().unwrap());
    assert_eq!(9, stream.pos());
    stream.set_pos(4).unwrap();
    assert_eq!(TestUnnamedFieldEnum::Bar(true), stream.read().unwrap());
    assert_eq!(7, stream.pos());
    assert_eq!(None, bit_size_of::<TestUnnamedFieldEnum>());
}

#[derive(BitReadSized, PartialEq, Debug)]
struct TestStructSized {
    foo: u8,
    #[size = "input_size"]
    string: String,
    #[size = "input_size"]
    int: u8,
}

#[test]
fn test_read_struct_sized() {
    let bytes = vec![
        12, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0, 0, 0, 0, 0, 0,
    ];
    let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    let mut stream = BitReadStream::from(buffer);
    assert_eq!(
        TestStructSized {
            foo: 12,
            string: "hel".to_owned(),
            int: 4,
        },
        stream.read_sized(3).unwrap()
    );
    assert_eq!(Some(8 + 2 * 8 + 2), bit_size_of_sized::<TestStructSized>(2));
}

#[derive(BitReadSized, PartialEq, Debug)]
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
    let bytes = vec![
        0b1100_0110,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
    ];
    let buffer = BitReadBuffer::new(&bytes, BigEndian);
    let mut stream = BitReadStream::from(buffer);
    assert_eq!(
        TestUnnamedFieldEnumSized::Asd(0b_00_0110),
        stream.read_sized(6).unwrap()
    );
    assert_eq!(8, stream.pos());
    assert_eq!(None, bit_size_of_sized::<TestUnnamedFieldEnumSized>(6));
}

#[derive(BitRead, PartialEq, Debug)]
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
        'e' as u8,
    ];
    let buffer = BitReadBuffer::new(&bytes, BigEndian);
    let mut stream = BitReadStream::from(buffer);
    assert_eq!(
        TestStruct2 {
            size: 5,
            str: "hello worl".to_owned(),
        },
        stream.read().unwrap()
    );
    assert_eq!(None, bit_size_of::<TestStruct2>());
}

#[derive(BitRead)]
#[endianness = "E"]
struct TestStruct3<'a, E: Endianness> {
    size: u8,
    #[size = "size"]
    stream: BitReadStream<'a, E>,
}

#[test]
fn test_read_struct3() {
    let bytes = vec![0b0000_0101, 0, 0, 0, 0, 0, 0, 0];
    let buffer = BitReadBuffer::new(&bytes, BigEndian);
    let mut stream = BitReadStream::from(buffer);
    let result: TestStruct3<BigEndian> = stream.read().unwrap();
    assert_eq!(5, result.size);
    assert_eq!(5, result.stream.bit_len());
    assert_eq!(None, bit_size_of::<TestStruct3<LittleEndian>>());
}

#[derive(BitRead, PartialEq, Debug)]
#[discriminant_bits = 2]
enum TestEnumRest {
    Foo,
    Bar,
    #[discriminant = "_"]
    Asd,
}

#[test]
fn test_read_rest_enum() {
    let bytes = vec![
        0b1100_0110,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
        0b1000_0100,
    ];
    let buffer = BitReadBuffer::new(&bytes, BigEndian);
    let mut stream = BitReadStream::from(buffer);
    assert_eq!(TestEnumRest::Asd, stream.read().unwrap());
    assert_eq!(TestEnumRest::Foo, stream.read().unwrap());
    assert_eq!(TestEnumRest::Bar, stream.read().unwrap());
    assert_eq!(TestEnumRest::Asd, stream.read().unwrap());
    assert_eq!(Some(2), bit_size_of::<TestEnumRest>());
}

#[derive(BitRead, PartialEq, Debug)]
struct UnnamedSize(u8, #[size = 5] String, bool);

fn test_unnamed_struct() {
    let bytes = vec![
        12, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0, 0, 0, 0, 0, 0,
    ];
    let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    let mut stream = BitReadStream::from(buffer);

    assert_eq!(
        UnnamedSize(12, "hello".to_string(), false),
        stream.read().unwrap()
    );
}

#[derive(BitRead, PartialEq, Debug)]
struct EmptyStruct;

fn test_empty_struct() {
    let bytes = vec![0, 0, 0, 0];
    let buffer = BitReadBuffer::new(&bytes, BigEndian);
    let mut stream = BitReadStream::from(buffer);
    assert_eq!(EmptyStruct, stream.read().unwrap());
    assert_eq!(0, stream.pos());
    assert_eq!(Some(0), bit_size_of::<EmptyStruct>());
}

#[derive(BitRead)]
struct SizeStruct {
    foo: u8,
    #[size = 6]
    str: String,
    bar: bool,
}

#[derive(BitRead)]
struct UnnamedSizeStruct(u8, #[size = 6] String, bool);

#[test]
fn test_bit_size() {
    assert_eq!(bit_size_of::<SizeStruct>(), Some(8 + 8 * 6 + 1));
    assert_eq!(bit_size_of::<UnnamedSizeStruct>(), Some(8 + 8 * 6 + 1));
}

#[derive(BitReadSized)]
struct SizeStructSized {
    foo: u8,
    #[size = "input_size"]
    str: String,
    bar: bool,
}

#[test]
fn test_bit_size_sized() {
    assert_eq!(bit_size_of_sized::<SizeStructSized>(6), Some(8 + 8 * 6 + 1));
    assert_eq!(
        bit_size_of_sized::<SizeStructSized>(16),
        Some(8 + 8 * 16 + 1)
    );
}
