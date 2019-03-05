#![feature(test)]

extern crate test;

use bitstream_reader::{BitBuffer, LittleEndian};
use test::Bencher;

fn read_perf(buffer: &BitBuffer<LittleEndian>) -> u16 {
    let size = 5;
    let mut pos = 0;
    let len = buffer.bit_len();
    let mut result: u16 = 0;
    loop {
        if pos + size > len {
            return result;
        }
        let data = buffer.read_int::<u16>(pos, size).unwrap();
        result = result.wrapping_add(data);
        pos += size;
    }
}

#[bench]
fn perf_non_padded(b: &mut Bencher) {
    let data = vec![1u8; 1024 * 1024 * 10];
    let buffer = BitBuffer::new(data, LittleEndian);
    b.iter(|| {
        let data = read_perf(&buffer);
        assert_eq!(data, 0);
        test::black_box(data);
    });
}
