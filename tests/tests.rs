use std::collections::HashMap;
use std::num::NonZeroU16;

use maplit::hashmap;

use bitstream_reader::{BigEndian, BitBuffer, BitRead, BitStream, LittleEndian};

const BYTES: &'static [u8] = &[
    0b1011_0101,
    0b0110_1010,
    0b1010_1100,
    0b1001_1001,
    0b1001_1001,
    0b1001_1001,
    0b1001_1001,
    0b1110_0111,
    0b1001_1001,
    0b1001_1001,
    0b1001_1001,
    0b1110_0111,
];

#[test]
fn read_u8_le() {
    let buffer = BitBuffer::new(BYTES.to_vec(), LittleEndian);

    assert_eq!(buffer.read_int::<u8>(0, 1).unwrap(), 0b1);
    assert_eq!(buffer.read_int::<u8>(1, 1).unwrap(), 0b0);
    assert_eq!(buffer.read_int::<u8>(2, 2).unwrap(), 0b01);
    assert_eq!(buffer.read_int::<u8>(0, 3).unwrap(), 0b101);
    assert_eq!(buffer.read_int::<u8>(7, 5).unwrap(), 0b1010_1);
    assert_eq!(buffer.read_int::<u8>(6, 5).unwrap(), 0b010_10);
    assert_eq!(buffer.read_int::<u8>(12, 5).unwrap(), 0b0_0110);
}

#[test]
fn read_u8_be() {
    let buffer = BitBuffer::new(BYTES.to_vec(), BigEndian);

    assert_eq!(buffer.read_int::<u8>(0, 1).unwrap(), 0b1);
    assert_eq!(buffer.read_int::<u8>(1, 1).unwrap(), 0b0);
    assert_eq!(buffer.read_int::<u8>(2, 2).unwrap(), 0b11);
    assert_eq!(buffer.read_int::<u8>(0, 3).unwrap(), 0b101);
    assert_eq!(buffer.read_int::<u8>(7, 5).unwrap(), 0b1011_0);
    assert_eq!(buffer.read_int::<u8>(6, 5).unwrap(), 0b01_011);
}

#[test]
fn read_u16_le() {
    let buffer = BitBuffer::new(BYTES.to_vec(), LittleEndian);

    assert_eq!(buffer.read_int::<u16>(6, 12).unwrap(), 0b00_0110_1010_10);
}

#[test]
fn read_u16_be() {
    let buffer = BitBuffer::new(BYTES.to_vec(), BigEndian);

    assert_eq!(buffer.read_int::<u16>(6, 12).unwrap(), 0b01_0110_1010_10);
}

#[test]
fn read_u32_le() {
    let buffer = BitBuffer::new(BYTES.to_vec(), LittleEndian);

    assert_eq!(
        buffer.read_int::<u32>(6, 24).unwrap(),
        0b01_1001_1010_1100_0110_1010_10
    );
}

#[test]
fn read_u32_be() {
    let buffer = BitBuffer::new(BYTES.to_vec(), BigEndian);

    assert_eq!(
        buffer.read_int::<u32>(6, 24).unwrap(),
        0b01_0110_1010_1010_1100_1001_10
    );
}

#[test]
fn read_u64_le() {
    let buffer = BitBuffer::new(BYTES.to_vec(), LittleEndian);

    assert_eq!(
        buffer.read_int::<u64>(6, 34).unwrap(),
        0b1001_1001_1001_1001_1010_1100_0110_1010_10
    );
    assert_eq!(
        buffer.read_int::<u64>(6, 60).unwrap(),
        0b01_1110_0111_1001_1001_1001_1001_1001_1001_1001_1001_1010_1100_0110_1010_10
    );
    assert_eq!(
        buffer.read_int::<u64>(6, 64).unwrap(),
        0b01_1001_1110_0111_1001_1001_1001_1001_1001_1001_1001_1001_1010_1100_0110_1010_10
    );
    assert_eq!(
        buffer.read_int::<u64>(8, 62).unwrap(),
        0b01_1001_1110_0111_1001_1001_1001_1001_1001_1001_1001_1001_1010_1100_0110_1010
    );
}

#[test]
fn read_u64_be() {
    let buffer = BitBuffer::new(BYTES.to_vec(), BigEndian);

    assert_eq!(
        buffer.read_int::<u64>(6, 34).unwrap(),
        0b01_0110_1010_1010_1100_1001_1001_1001_1001
    );
    assert_eq!(
        buffer.read_int::<u64>(6, 60).unwrap(),
        0b01_0110_1010_1010_1100_1001_1001_1001_1001_1001_1001_1001_1001_1110_0111_10
    );
    assert_eq!(
        buffer.read_int::<u64>(6, 64).unwrap(),
        0b01_0110_1010_1010_1100_1001_1001_1001_1001_1001_1001_1001_1001_1110_0111_1001_10
    );
}

#[test]
fn read_i8_le() {
    let buffer = BitBuffer::new(BYTES.to_vec(), LittleEndian);

    assert_eq!(buffer.read_int::<i8>(0, 3).unwrap(), -0b01);
    assert_eq!(buffer.read_int::<i8>(0, 8).unwrap(), -0b011_0101);
}

#[test]
fn read_i8_be() {
    let buffer = BitBuffer::new(BYTES.to_vec(), BigEndian);

    assert_eq!(buffer.read_int::<i8>(1, 2).unwrap(), 0b1);
    assert_eq!(buffer.read_int::<i8>(0, 3).unwrap(), -0b01);
    assert_eq!(buffer.read_int::<i8>(0, 8).unwrap(), -0b011_0101);
}

#[test]
fn read_i16_le() {
    let buffer = BitBuffer::new(BYTES.to_vec(), LittleEndian);

    assert_eq!(buffer.read_int::<i16>(6, 12).unwrap(), 0b0_0110_1010_10);
    assert_eq!(buffer.read_int::<i16>(6, 13).unwrap(), -0b00_0110_1010_10);
}

#[test]
fn read_i16_be() {
    let buffer = BitBuffer::new(BYTES.to_vec(), BigEndian);

    assert_eq!(buffer.read_int::<i16>(6, 12).unwrap(), 0b1_0110_1010_10);
    assert_eq!(buffer.read_int::<i16>(7, 12).unwrap(), -0b0110_1010_101);
}

#[test]
fn read_i32_le() {
    let buffer = BitBuffer::new(BYTES.to_vec(), LittleEndian);

    assert_eq!(
        buffer.read_int::<i32>(6, 24).unwrap(),
        0b1_1001_1010_1100_0110_1010_10
    );
    assert_eq!(
        buffer.read_int::<i32>(6, 26).unwrap(),
        -0b001_1001_1010_1100_0110_1010_10
    );
}

#[test]
fn read_i32_be() {
    let buffer = BitBuffer::new(BYTES.to_vec(), BigEndian);

    assert_eq!(
        buffer.read_int::<i32>(7, 24).unwrap(),
        -0b0110_1010_1010_1100_1001_100
    );
}

#[test]
fn read_i64_le() {
    let buffer = BitBuffer::new(BYTES.to_vec(), LittleEndian);

    assert_eq!(
        buffer.read_int::<i64>(6, 34).unwrap(),
        -0b001_1001_1001_1001_1010_1100_0110_1010_10
    );
    assert_eq!(
        buffer.read_int::<i64>(6, 59).unwrap(),
        -0b1110_01111001_1001_1001_1001_1001_1001_1001_1001_1010_1100_0110_1010_10
    );
    assert_eq!(
        buffer.read_int::<i64>(1, 64).unwrap(),
        -0b1110_01111001_1001_1001_1001_1001_1001_1001_1001_1010_1100_0110_1010_1011_010
    );
}

#[test]
fn read_i64_be() {
    let buffer = BitBuffer::new(BYTES.to_vec(), BigEndian);

    assert_eq!(
        buffer.read_int::<i64>(7, 34).unwrap(),
        -0b0110_1010_1010_1100_1001_1001_1001_1001_1
    );
    assert_eq!(
        buffer.read_int::<i64>(7, 60).unwrap(),
        -0b0110_1010_1010_1100_1001_1001_1001_1001_1001_1001_1001_1001_1110_0111_100
    );
    assert_eq!(
        buffer.read_int::<i64>(7, 64).unwrap(),
        -0b0110_1010_1010_1100_1001_1001_1001_1001_1001_1001_1001_1001_1110_0111_1001_100
    );
}

#[test]
fn read_f32_le() {
    let buffer = BitBuffer::new(BYTES.to_vec(), LittleEndian);

    assert_eq!(buffer.read_float::<f64>(6).unwrap(), 135447455835963910000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0);
}

#[test]
fn read_f64_le() {
    let buffer = BitBuffer::new(BYTES.to_vec(), LittleEndian);

    assert_eq!(buffer.read_float::<f64>(6).unwrap(), 135447455835963910000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0);
}

#[test]
fn test_from() {
    let buffer: BitBuffer<LittleEndian> = BitBuffer::from(BYTES.to_vec());
    let _: BitStream<LittleEndian> = BitStream::from(buffer);
    let _: BitStream<LittleEndian> = BitStream::from(BYTES.to_vec());
}

#[test]
fn test_read_str_be() {
    let bytes = vec![
        0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0, 0, 0, 0, 0,
    ];
    let buffer = BitBuffer::new(bytes, BigEndian);
    assert_eq!(
        buffer.read_string(0, Some(13)).unwrap(),
        "Hello world".to_owned()
    );
    assert_eq!(
        buffer.read_string(0, Some(16)).unwrap(),
        "Hello world".to_owned()
    );
    assert_eq!(
        buffer.read_string(0, None).unwrap(),
        "Hello world".to_owned()
    );
}

#[test]
fn test_read_str_le() {
    let bytes = vec![
        'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, ' ' as u8, 'w' as u8, 'o' as u8,
        'r' as u8, 'l' as u8, 'd' as u8, 0, 'f' as u8, 'o' as u8, 'o' as u8, 0, 0, 0, 0, 0,
    ];
    let buffer = BitBuffer::new(bytes, LittleEndian);
    assert_eq!(buffer.read_string(0, Some(3)).unwrap(), "hel".to_owned());
    assert_eq!(
        buffer.read_string(0, Some(11)).unwrap(),
        "hello world".to_owned()
    );
    assert_eq!(
        buffer.read_string(0, None).unwrap(),
        "hello world".to_owned()
    );
}

#[test]
fn read_trait() {
    let buffer = BitBuffer::new(BYTES.to_vec(), BigEndian);
    let mut stream = BitStream::new(buffer);
    let a: u8 = stream.read().unwrap();
    assert_eq!(0b1011_0101, a);
    let b: i8 = stream.read().unwrap();
    assert_eq!(0b110_1010, b);
    let c: i16 = stream.read().unwrap();
    assert_eq!(-0b0010_1100_1001_1001, c);
    let d: bool = stream.read().unwrap();
    assert_eq!(true, d);
    let e: Option<u8> = stream.read().unwrap();
    assert_eq!(None, e);
    stream.set_pos(0).unwrap();
    let f: Option<u8> = stream.read().unwrap();
    assert_eq!(Some(0b011_0101_0), f);
}

#[test]
fn read_sized_trait() {
    let buffer = BitBuffer::new(BYTES.to_vec(), BigEndian);
    let mut stream = BitStream::new(buffer);
    let a: u8 = stream.read_sized(4).unwrap();
    assert_eq!(0b1011, a);
    stream.set_pos(0).unwrap();
    let vec: Vec<u16> = stream.read_sized(3).unwrap();
    assert_eq!(
        vec![
            0b1011_0101_0110_1010,
            0b1010_1100_1001_1001,
            0b1001_1001_1001_1001
        ],
        vec
    );
    stream.set_pos(0).unwrap();
    let vec: Vec<u8> = stream.read_sized(3).unwrap();
    assert_eq!(vec![0b1011_0101, 0b0110_1010, 0b1010_1100], vec);
    stream.set_pos(0).unwrap();
    let result: HashMap<u8, u8> = stream.read_sized(2).unwrap();
    assert_eq!(
        hashmap!(0b1011_0101 => 0b0110_1010, 0b1010_1100 => 0b1001_1001),
        result
    );
    stream.set_pos(0).unwrap();
    let mut result: BitStream<BigEndian> = stream.read_sized(4).unwrap();
    assert_eq!(0b10u8, result.read_int(2).unwrap());
}

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

#[test]
fn test_read_nonzero() {
    let bytes = vec![12, 0, 0, 0];
    let buffer = BitBuffer::new(bytes, LittleEndian);
    let mut stream = BitStream::from(buffer);
    assert_eq!(NonZeroU16::new(12), stream.read().unwrap());
    assert_eq!(None, stream.read::<Option<NonZeroU16>>().unwrap());
}
