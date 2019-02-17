use std::fs;
use super::*;
use test::Bencher;

const BYTES: &'static[u8] = &[
    0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111,
    0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111,
    0, 0, 0, 0, 0, 0, 0, 0 ,0
];

#[test]
fn read_u8() {
    let buffer = BitBuffer::from_padded_slice(BYTES, 12);

    assert_eq!(buffer.read::<u8>(0, 1).unwrap(), 0b1);
    assert_eq!(buffer.read::<u8>(1, 1).unwrap(), 0b0);
    assert_eq!(buffer.read::<u8>(2, 2).unwrap(), 0b01);
    assert_eq!(buffer.read::<u8>(0, 3).unwrap(), 0b101);
    assert_eq!(buffer.read::<u8>(7, 5).unwrap(), 0b1010_1);
    assert_eq!(buffer.read::<u8>(6, 5).unwrap(), 0b010_10);
}

#[test]
fn read_u16() {
    let buffer = BitBuffer::from_padded_slice(BYTES, 12);

    assert_eq!(buffer.read::<u16>(6, 12).unwrap(), 0b00_0110_1010_10);
}

#[test]
fn read_u32() {
    let buffer = BitBuffer::from_padded_slice(BYTES, 12);

    assert_eq!(buffer.read::<u32>(6, 24).unwrap(), 0b01_1001_1010_1100_0110_1010_10);
}

#[test]
fn read_u64() {
    let buffer = BitBuffer::from_padded_slice(BYTES, 12);

    assert_eq!(buffer.read::<u64>(6, 34).unwrap(), 0b1001_1001_1001_1001_1010_1100_0110_1010_10);
    assert_eq!(buffer.read::<u64>(6, 60).unwrap(), 0b00_1110_01111001_1001_1001_1001_1001_1001_1001_1001_1010_1100_0110_1010_10);
}

#[test]
fn read_i8() {
    let buffer = BitBuffer::from_padded_slice(BYTES, 12);

    assert_eq!(buffer.read::<i8>(0, 3).unwrap(), -0b1);
    assert_eq!(buffer.read::<i8>(0, 8).unwrap(), -0b011_0101);
}

fn read_perf(buffer: BitBuffer) -> u16 {
    let size = 5;
    let mut pos = 0;
    let len = buffer.bit_len();
    let mut result: u16 = 0;
    loop {
        if pos + size > len {
            return result;
        }
        let data = buffer.read::<u16>(pos, size).unwrap();
        result = result.wrapping_add(data);
        pos += size;
    }
}

#[bench]
fn perf(b: &mut Bencher) {
    let mut file = fs::read("/bulk/tmp/test.dem").expect("Unable to read file");
    let length = file.len();
    file.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0]);
    let bytes = file.as_slice();
    b.iter(|| {
        let buffer = BitBuffer::from_padded_slice(&bytes, length);
        let data = read_perf(buffer);
        assert_eq!(data, 43943);
        test::black_box(data);
    });
}