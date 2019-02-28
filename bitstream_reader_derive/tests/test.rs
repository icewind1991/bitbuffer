use bitstream_reader::{BigEndian, BitBuffer, BitStream, Endianness, LittleEndian};
use bitstream_reader_derive::{BitRead, BitReadSized};

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
    let buffer = BitBuffer::new(bytes, LittleEndian);
    let mut stream = BitStream::from(buffer);
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
    let buffer = BitBuffer::new(bytes, BigEndian);
    let mut stream = BitStream::from(buffer);
    assert_eq!(TestBareEnum::Asd, stream.read().unwrap());
    assert_eq!(TestBareEnum::Foo, stream.read().unwrap());
    assert_eq!(TestBareEnum::Bar, stream.read().unwrap());
    assert_eq!(true, stream.read::<TestBareEnum>().is_err());
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
    let buffer = BitBuffer::new(bytes, BigEndian);
    let mut stream = BitStream::from(buffer);
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
    let buffer = BitBuffer::new(bytes, LittleEndian);
    let mut stream = BitStream::from(buffer);
    assert_eq!(
        TestStructSized {
            foo: 12,
            string: "hel".to_owned(),
            int: 4,
        },
        stream.read_sized(3).unwrap()
    );
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
    let buffer = BitBuffer::new(bytes, BigEndian);
    let mut stream = BitStream::from(buffer);
    assert_eq!(
        TestUnnamedFieldEnumSized::Asd(0b_00_0110),
        stream.read_sized(6).unwrap()
    );
    assert_eq!(8, stream.pos());
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
    let buffer = BitBuffer::new(bytes, BigEndian);
    let mut stream = BitStream::from(buffer);
    assert_eq!(
        TestStruct2 {
            size: 5,
            str: "hello worl".to_owned(),
        },
        stream.read().unwrap()
    );
}

#[derive(BitRead)]
#[endianness = "E"]
struct TestStruct3<E: Endianness> {
    size: u8,
    #[size = "size"]
    stream: BitStream<E>,
}

#[test]
fn test_read_struct3() {
    let bytes = vec![0b0000_0101, 0, 0, 0, 0, 0, 0, 0];
    let buffer = BitBuffer::new(bytes, BigEndian);
    let mut stream = BitStream::from(buffer);
    let result: TestStruct3<BigEndian> = stream.read().unwrap();
    assert_eq!(5, result.size);
    assert_eq!(5, result.stream.bit_len());
}
