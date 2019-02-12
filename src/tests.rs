use std::fs;
use super::*;
use test::Bencher;

#[test]
fn read_be() {
    let bytes = &[
        0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
        0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111,
    ];

    let buffer = BitBuffer::new(bytes, ByteOrder::BigEndian);

    assert_eq!(buffer.read_u8(0, 1).unwrap(), 0b1);
    assert_eq!(buffer.read_u8(1, 1).unwrap(), 0b0);
    assert_eq!(buffer.read_u8(2, 2).unwrap(), 0b11);
    assert_eq!(buffer.read_u8(7, 4).unwrap(), 0b1011);
    assert_eq!(buffer.read_u8(6, 5).unwrap(), 0b01011);
}

#[test]
fn read_le() {
    let bytes = &[
        0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
        0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111,
    ];

    let buffer = BitBuffer::new(bytes, ByteOrder::LittleEndian);

    assert_eq!(buffer.read_u8(0, 1).unwrap(), 0b1);
    assert_eq!(buffer.read_u8(1, 1).unwrap(), 0b0);
    assert_eq!(buffer.read_u8(2, 2).unwrap(), 0b01);
    assert_eq!(buffer.read_u8(7, 5).unwrap(), 0b10101);
    assert_eq!(buffer.read_u8(6, 5).unwrap(), 0b01010);
    assert_eq!(buffer.read_u16(6, 12).unwrap(), 0b000110101010);
}

#[test]
fn signed_values() {
    let from = -2048;
    let to = 2048;
    for x in from..to {
        let bytes = &[
            (x >> 8) as u8,
            x as u8,
        ];
        let buffer = BitBuffer::new(bytes, ByteOrder::BigEndian);
        assert_eq!(buffer.read_u8(0, 4).unwrap(), if x < 0 { 0b1111 } else { 0 });
        assert_eq!(buffer.read_i16(4, 12).unwrap(), x);
    }
}

fn read_perf(buffer: BitBuffer) -> u16 {
    let size = 5;
    let mut pos = 0;
    let len = buffer.bit_len();
    let mut result: u16 = 0;
    //while pos < len {
    loop {
        if pos + size > len {
            return result;
        }
        let data = buffer.read_u16(pos, size).unwrap();
        result = result.wrapping_add(data);
        pos += size;
    }
    return result;
}

#[bench]
fn perf(b: &mut Bencher) {
    let data = fs::read("/bulk/tmp/test.dem").expect("Unable to read file");
    b.iter(|| {
        let buffer = BitBuffer::new(data.as_slice(), ByteOrder::LittleEndian);
        let data = read_perf(buffer);
        test::black_box(data);
    });
}