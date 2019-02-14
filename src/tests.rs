use std::fs;
use super::*;
use test::Bencher;

#[test]
fn read_le() {
    let bytes: &[u8] = &[
        0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
        0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111,
        0, 0, 0, 0, 0, 0, 0, 0
    ];

    let buffer = BitBuffer::from_padded_slice(&bytes, 8);

    assert_eq!(buffer.read_u8(0, 1), 0b1);
    assert_eq!(buffer.read_u8(1, 1), 0b0);
    assert_eq!(buffer.read_u8(2, 2), 0b01);
    assert_eq!(buffer.read_u8(7, 5), 0b10101);
    assert_eq!(buffer.read_u8(6, 5), 0b01010);
    assert_eq!(buffer.read_u16(6, 12), 0b000110101010);
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
        let data = buffer.read_u16(pos, size);
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
//        assert_eq!(data, 43943);
        test::black_box(data);
    });
}