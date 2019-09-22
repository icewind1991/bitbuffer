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

const F64_RESULT: f64 = 0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010156250477904244;

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
        assert_eq!(result, F64_RESULT);
        test::black_box(result);
    });
}

#[bench]
fn perf_bool(b: &mut Bencher) {
    let data = vec![1u8; 1024 * 1024 * 1];
    let buffer = BitBuffer::new(data, BigEndian);
    b.iter(|| {
        let mut pos = 0;
        let len = buffer.bit_len();
        loop {
            if pos >= len {
                break;
            }
            let num = buffer.read_bool(pos).unwrap();
            test::black_box(num);
            pos += 1;
        }
    });
}

fn build_string_data(size: usize, inputs: &Vec<&str>) -> Vec<u8> {
    let mut data = Vec::with_capacity(size);
    loop {
        for input in inputs.iter() {
            if data.len() + input.len() + 1 >= size {
                return data;
            }
            data.extend_from_slice(input.as_bytes());
            data.push(0);
        }
    }
}

fn get_string_buffer() -> Vec<u8> {
    let inputs = vec![
        "foo",
        "bar",
        "something a little bit longer for extra testing",
        "a",
        "",
    ];
    build_string_data(10 * 1024 * 1024, &inputs)
}

#[bench]
fn perf_string_be(b: &mut Bencher) {
    let buffer = BitBuffer::new(get_string_buffer(), BigEndian);

    b.iter(|| {
        let mut pos = 0;
        let len = buffer.bit_len();
        loop {
            if pos + (128 * 8) > len {
                break;
            }
            let result = buffer.read_string(pos, None).unwrap();
            pos += (result.len() + 1) * 8;
            test::black_box(result);
        }
    });
}

#[bench]
fn perf_string_le(b: &mut Bencher) {
    let buffer = BitBuffer::new(get_string_buffer(), LittleEndian);

    b.iter(|| {
        let mut pos = 0;
        let len = buffer.bit_len();
        loop {
            if pos + (128 * 8) > len {
                break;
            }
            let result = buffer.read_string(pos, None).unwrap();
            pos += (result.len() + 1) * 8;
            test::black_box(result);
        }
    });
}

#[bench]
fn perf_bytes_be(b: &mut Bencher) {
    let buffer = BitBuffer::new(get_string_buffer(), BigEndian);

    b.iter(|| {
        let mut pos = 0;
        let len = buffer.bit_len();
        loop {
            if pos + (128 * 8) > len {
                break;
            }
            let result = buffer.read_bytes(pos, 128).unwrap();
            pos += (result.len() + 1) * 8;
            test::black_box(result);
        }
    });
}

#[bench]
fn perf_bytes_le(b: &mut Bencher) {
    let buffer = BitBuffer::new(get_string_buffer(), LittleEndian);

    b.iter(|| {
        let mut pos = 0;
        let len = buffer.bit_len();
        loop {
            if pos + (128 * 8) > len {
                break;
            }
            let result = buffer.read_bytes(pos, 128).unwrap();
            pos += (result.len() + 1) * 8;
            test::black_box(result);
        }
    });
}

#[bench]
fn perf_bytes_be_unaligned(b: &mut Bencher) {
    let buffer = BitBuffer::new(get_string_buffer(), BigEndian);

    b.iter(|| {
        let mut pos = 3;
        let len = buffer.bit_len();
        loop {
            if pos + (128 * 8) > len {
                break;
            }
            let result = buffer.read_bytes(pos, 128).unwrap();
            pos += (result.len() + 1) * 8;
            test::black_box(result);
        }
    });
}

#[bench]
fn perf_bytes_le_unaligned(b: &mut Bencher) {
    let buffer = BitBuffer::new(get_string_buffer(), LittleEndian);

    b.iter(|| {
        let mut pos = 3;
        let len = buffer.bit_len();
        loop {
            if pos + (128 * 8) > len {
                break;
            }
            let result = buffer.read_bytes(pos, 128).unwrap();
            pos += (result.len() + 1) * 8;
            test::black_box(result);
        }
    });
}
