use std::collections::HashMap;
use std::num::NonZeroU16;

use maplit::hashmap;

use bitbuffer::{BigEndian, BitError, BitRead, BitReadBuffer, BitReadStream, LittleEndian};

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
    let buffer = BitReadBuffer::new(BYTES, LittleEndian);

    assert_eq!(buffer.read_int::<u8>(0, 1).unwrap(), 0b1);
    assert_eq!(buffer.read_bool(0).unwrap(), true);
    assert_eq!(buffer.read_int::<u8>(1, 1).unwrap(), 0b0);
    assert_eq!(buffer.read_bool(1).unwrap(), false);
    assert_eq!(buffer.read_int::<u8>(2, 2).unwrap(), 0b01);
    assert_eq!(buffer.read_int::<u8>(0, 3).unwrap(), 0b101);
    assert_eq!(buffer.read_int::<u8>(7, 5).unwrap(), 0b1010_1);
    assert_eq!(buffer.read_int::<u8>(6, 5).unwrap(), 0b010_10);
    assert_eq!(buffer.read_int::<u8>(12, 5).unwrap(), 0b0_0110);
}

#[test]
fn read_u8_be() {
    let buffer = BitReadBuffer::new(BYTES, BigEndian);

    assert_eq!(buffer.read_int::<u8>(0, 1).unwrap(), 0b1);
    assert_eq!(buffer.read_int::<u8>(1, 1).unwrap(), 0b0);
    assert_eq!(buffer.read_int::<u8>(2, 2).unwrap(), 0b11);
    assert_eq!(buffer.read_int::<u8>(0, 3).unwrap(), 0b101);
    assert_eq!(buffer.read_int::<u8>(7, 5).unwrap(), 0b1011_0);
    assert_eq!(buffer.read_int::<u8>(6, 5).unwrap(), 0b01_011);

    assert_eq!(buffer.read_bool(0).unwrap(), true);
    assert_eq!(buffer.read_bool(8).unwrap(), false);
}

#[test]
fn read_u16_le() {
    let buffer = BitReadBuffer::new(BYTES, LittleEndian);

    assert_eq!(buffer.read_int::<u16>(6, 12).unwrap(), 0b00_0110_1010_10);
}

#[test]
fn read_u16_be() {
    let buffer = BitReadBuffer::new(BYTES, BigEndian);

    assert_eq!(buffer.read_int::<u16>(6, 12).unwrap(), 0b01_0110_1010_10);
}

#[test]
fn read_u32_le() {
    let buffer = BitReadBuffer::new(BYTES, LittleEndian);

    assert_eq!(
        buffer.read_int::<u32>(6, 24).unwrap(),
        0b01_1001_1010_1100_0110_1010_10
    );
}

#[test]
fn read_u32_be() {
    let buffer = BitReadBuffer::new(BYTES, BigEndian);

    assert_eq!(
        buffer.read_int::<u32>(6, 24).unwrap(),
        0b01_0110_1010_1010_1100_1001_10
    );
}

#[test]
fn read_u64_le() {
    let buffer = BitReadBuffer::new(BYTES, LittleEndian);

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
    let buffer = BitReadBuffer::new(BYTES, BigEndian);

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
    let buffer = BitReadBuffer::new(BYTES, LittleEndian);

    assert_eq!(buffer.read_int::<i8>(0, 3).unwrap(), -0b11);
    assert_eq!(buffer.read_int::<i8>(0, 8).unwrap(), -0b100_1011);
}

#[test]
fn read_i8_be() {
    let buffer = BitReadBuffer::new(BYTES, BigEndian);

    assert_eq!(buffer.read_int::<i8>(1, 2).unwrap(), 0b1);
    assert_eq!(buffer.read_int::<i8>(0, 3).unwrap(), -0b11);
    assert_eq!(buffer.read_int::<i8>(0, 8).unwrap(), -0b100_1011);
}

#[test]
fn read_i16_le() {
    let buffer = BitReadBuffer::new(BYTES, LittleEndian);

    assert_eq!(buffer.read_int::<i16>(6, 12).unwrap(), 0b0_0110_1010_10);
    assert_eq!(buffer.read_int::<i16>(6, 13).unwrap(), -0b11_1001_0101_10);
}

#[test]
fn read_i16_be() {
    let buffer = BitReadBuffer::new(BYTES, BigEndian);

    assert_eq!(buffer.read_int::<i16>(6, 12).unwrap(), 0b1_0110_1010_10);
    assert_eq!(buffer.read_int::<i16>(7, 12).unwrap(), -0b1001_0101_011);
}

#[test]
fn read_i32_le() {
    let buffer = BitReadBuffer::new(BYTES, LittleEndian);

    assert_eq!(
        buffer.read_int::<i32>(6, 24).unwrap(),
        0b1_1001_1010_1100_0110_1010_10
    );
    assert_eq!(buffer.read_int::<i32>(6, 26).unwrap(), -26824278);
}

#[test]
fn read_i32_be() {
    let buffer = BitReadBuffer::new(BYTES, BigEndian);

    assert_eq!(buffer.read_int::<i32>(7, 24).unwrap(), -4893108);
}

#[test]
fn read_i64_le() {
    let buffer = BitReadBuffer::new(BYTES, LittleEndian);

    assert_eq!(buffer.read_int::<i64>(6, 34).unwrap(), -6871928406);
    assert_eq!(buffer.read_int::<i64>(6, 59).unwrap(), -27471957726940758);
    assert_eq!(buffer.read_int::<i64>(1, 64).unwrap(), -879102647262104230);
}

#[test]
fn read_i64_be() {
    let buffer = BitReadBuffer::new(BYTES, BigEndian);

    assert_eq!(buffer.read_int::<i64>(7, 34).unwrap(), -5010541773);
    assert_eq!(buffer.read_int::<i64>(7, 60).unwrap(), -336251766397153476);
    assert_eq!(buffer.read_int::<i64>(7, 64).unwrap(), -5380028262354455604);
}

#[test]
fn read_f32_le() {
    let buffer = BitReadBuffer::new(BYTES, LittleEndian);

    assert_eq!(buffer.read_float::<f64>(6).unwrap(), 135447455835963910000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0);
}

#[test]
fn read_f64_le() {
    let buffer = BitReadBuffer::new(BYTES, LittleEndian);

    assert_eq!(buffer.read_float::<f64>(6).unwrap(), 135447455835963910000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0);
}

#[test]
fn test_from() {
    let buffer: BitReadBuffer<LittleEndian> = BitReadBuffer::from(BYTES);
    let _: BitReadStream<LittleEndian> = BitReadStream::from(buffer);
    let _: BitReadStream<LittleEndian> = BitReadStream::from(BYTES);
}

#[test]
fn test_read_str_be() {
    let bytes = vec![
        0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0, 0, 0, 0, 0,
    ];
    let buffer = BitReadBuffer::new(&bytes, BigEndian);
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
fn test_read_str_no_null_termination_le() {
    let bytes = vec![
        0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
    ];
    let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    assert_eq!(
        buffer.read_string(0, None).unwrap(),
        "Hello world".to_owned()
    );
}

#[test]
fn test_read_str_no_null_termination_be() {
    let bytes = vec![
        0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
    ];
    let buffer = BitReadBuffer::new(&bytes, BigEndian);
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
    let buffer = BitReadBuffer::new(&bytes, LittleEndian);
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
    let buffer = BitReadBuffer::new(BYTES, BigEndian);
    let mut stream = BitReadStream::new(buffer);
    let a: u8 = stream.read().unwrap();
    assert_eq!(0b1011_0101, a);
    let b: i8 = stream.read().unwrap();
    assert_eq!(0b110_1010, b);
    let c: i16 = stream.read().unwrap();
    assert_eq!(-0b101_0011_0110_0111, c);
    let d: bool = stream.read().unwrap();
    assert_eq!(true, d);
    let e: Option<u8> = stream.read().unwrap();
    assert_eq!(None, e);
    stream.set_pos(0).unwrap();
    let f: Option<u8> = stream.read().unwrap();
    assert_eq!(Some(0b011_0101_0), f);
}

#[test]
fn read_trait_unchecked() {
    unsafe {
        let buffer = BitReadBuffer::new(BYTES, BigEndian);
        let mut stream = BitReadStream::new(buffer);
        let a: u8 = stream.read_unchecked(true).unwrap();
        assert_eq!(0b1011_0101, a);
        let b: i8 = stream.read_unchecked(true).unwrap();
        assert_eq!(0b110_1010, b);
        let c: i16 = stream.read_unchecked(true).unwrap();
        assert_eq!(-0b101_0011_0110_0111, c);
        let d: bool = stream.read_unchecked(true).unwrap();
        assert_eq!(true, d);
        let e: Option<u8> = stream.read_unchecked(true).unwrap();
        assert_eq!(None, e);
        stream.set_pos(0).unwrap();
        let f: Option<u8> = stream.read_unchecked(true).unwrap();
        assert_eq!(Some(0b011_0101_0), f);
    }
}

#[test]
fn read_sized_trait() {
    let buffer = BitReadBuffer::new(BYTES, BigEndian);
    let mut stream = BitReadStream::new(buffer);
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
    let mut result: BitReadStream<BigEndian> = stream.read_sized(4).unwrap();
    assert_eq!(0b10u8, result.read_int::<u8>(2).unwrap());
}

#[test]
fn read_sized_trait_unchecked() {
    unsafe {
        let buffer = BitReadBuffer::new(BYTES, BigEndian);
        let mut stream = BitReadStream::new(buffer);
        let a: u8 = stream.read_sized_unchecked(4, true).unwrap();
        assert_eq!(0b1011, a);
        stream.set_pos(0).unwrap();
        let vec: Vec<u16> = stream.read_sized_unchecked(3, true).unwrap();
        assert_eq!(
            vec![
                0b1011_0101_0110_1010,
                0b1010_1100_1001_1001,
                0b1001_1001_1001_1001
            ],
            vec
        );
        stream.set_pos(0).unwrap();
        let vec: Vec<u8> = stream.read_sized_unchecked(3, true).unwrap();
        assert_eq!(vec![0b1011_0101, 0b0110_1010, 0b1010_1100], vec);
        stream.set_pos(0).unwrap();
        let result: HashMap<u8, u8> = stream.read_sized_unchecked(2, true).unwrap();
        assert_eq!(
            hashmap!(0b1011_0101 => 0b0110_1010, 0b1010_1100 => 0b1001_1001),
            result
        );
        stream.set_pos(0).unwrap();
        let mut result: BitReadStream<BigEndian> = stream.read_sized_unchecked(4, true).unwrap();
        assert_eq!(0b10u8, result.read_int::<u8>(2).unwrap());
    }
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
}

#[test]
fn test_read_nonzero() {
    let bytes = vec![12, 0, 0, 0];
    let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    let mut stream = BitReadStream::from(buffer);
    assert_eq!(NonZeroU16::new(12), stream.read().unwrap());
    assert_eq!(None, stream.read::<Option<NonZeroU16>>().unwrap());
}

#[test]
fn read_read_signed() {
    let bytes = vec![255, 255, 255, 255, 255, 255, 255, 255];
    let buffer = BitReadBuffer::new(&bytes, LittleEndian);

    assert_eq!(buffer.read_int::<i32>(0, 32).unwrap(), -1);

    let bytes = (-10i32).to_le_bytes();
    let mut byte_vec = Vec::with_capacity(4);
    byte_vec.extend_from_slice(&bytes);
    let buffer = BitReadBuffer::new(&byte_vec, LittleEndian);
    assert_eq!(buffer.read_int::<i32>(0, 32).unwrap(), -10);
}

#[test]
fn test_to_owned_stream() {
    let bytes = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    let mut stream = BitReadStream::new(buffer);
    let mut stream = stream.read_bits(15 * 7).unwrap();
    stream.skip_bits(25).unwrap();

    let mut owned = stream.to_owned();

    assert_eq!(stream.read::<u8>().unwrap(), owned.read::<u8>().unwrap());
    assert_eq!(stream.read::<u16>().unwrap(), owned.read::<u16>().unwrap());
    assert_eq!(stream.read::<u8>().unwrap(), owned.read::<u8>().unwrap());

    assert_eq!(stream.bit_len(), owned.bit_len());
    assert_eq!(stream.bits_left(), owned.bits_left());
}

#[test]
fn test_invalid_utf8() {
    let bytes = vec![b'b', b'a', 129, b'c', 0, 0, 0];
    let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    let mut stream = BitReadStream::new(buffer.clone());

    assert!(matches!(
        stream.read_string(None),
        Err(BitError::Utf8Error(_, 4))
    ));

    assert_eq!(stream.pos(), 5 * 8);

    let mut stream = BitReadStream::new(buffer);

    assert!(matches!(
        stream.read_string(Some(6)),
        Err(BitError::Utf8Error(_, 6))
    ));

    assert_eq!(stream.pos(), 6 * 8);
}
