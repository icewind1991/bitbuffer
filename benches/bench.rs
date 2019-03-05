#![feature(test)]

extern crate test;

use bitstream_reader::{BigEndian, BitBuffer, Endianness, LittleEndian};
use test::Bencher;

fn read_perf<E: Endianness>(buffer: &BitBuffer<E>) -> u16 {
    let size = 5;
    let mut pos = 0;
    let len = buffer.bit_len();
    let mut result: u16 = 0;
    loop {
        if pos + size > len {
            return result;
        }
        let data = buffer.read_int::<u64>(pos, size).unwrap() as u16;
        result = result.wrapping_add(data);
        pos += size;
    }
}
#[bench]
fn perf_le(b: &mut Bencher) {
    let data = vec![1u8; 1024 * 1024 * 10];
    let buffer = BitBuffer::new(data, LittleEndian);
    b.iter(|| {
        let data = read_perf(&buffer);
        assert_eq!(data, 0);
        test::black_box(data);
    });
}

#[bench]
fn perf_be(b: &mut Bencher) {
    let data = vec![1u8; 1024 * 1024 * 10];
    let buffer = BitBuffer::new(data, BigEndian);
    b.iter(|| {
        let data = read_perf(&buffer);
        assert_eq!(data, 0);
        test::black_box(data);
    });
}

#[bench]
fn perf_f32(b: &mut Bencher) {
    let data = vec![1u8; 1024 * 1024 * 10];
    let buffer = BitBuffer::new(data, BigEndian);
    b.iter(|| {
        let mut pos = 0;
        let len = buffer.bit_len();
        let mut result: f32 = 0.0;
        loop {
            if pos + 32 > len {
                break;
            }
            let num = buffer.read_float::<f32>(pos).unwrap();
            result += num;
            pos += 32;
        }
        assert_eq!(result, 0.00000000000000000000000000000006170106);
        test::black_box(result);
    });
}

#[bench]
fn perf_f64(b: &mut Bencher) {
    let data = vec![1u8; 1024 * 1024 * 10];
    let buffer = BitBuffer::new(data, BigEndian);
    b.iter(|| {
        let mut pos = 0;
        let len = buffer.bit_len();
        let mut result: f64 = 0.0;
        loop {
            if pos + 64 > len {
                break;
            }
            let num = buffer.read_float::<f64>(pos).unwrap();
            result += num;
            pos += 64;
        }
        assert_eq!(result, 0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010156250477904244);
        test::black_box(result);
    });
}
