use bitbuffer::{BigEndian, BitRead, BitReadBuffer, BitReadStream, Endianness, LittleEndian};
use iai::black_box;

fn read_perf<E: Endianness>(buffer: &BitReadBuffer<E>) -> u16 {
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

const ONES: &[u8; 1024 * 1024 * 10] = &[1u8; 1024 * 1024 * 10];

fn perf_le() {
    let buffer = BitReadBuffer::new(black_box(ONES), BigEndian);
    let data = read_perf(&buffer);
    assert_eq!(data, 0);
    black_box(data);
}

fn perf_be() {
    let buffer = BitReadBuffer::new(black_box(ONES), BigEndian);
    let data = read_perf(&buffer);
    assert_eq!(data, 0);
    black_box(data);
}

fn perf_f32_be() {
    let buffer = BitReadBuffer::new(black_box(ONES), BigEndian);
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
    black_box(result);
}

fn perf_f32_le() {
    let buffer = BitReadBuffer::new(black_box(ONES), BigEndian);
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
    black_box(result);
}

const F64_RESULT: f64 = 0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010156250477904244;

fn perf_f64() {
    let buffer = BitReadBuffer::new(black_box(ONES), BigEndian);
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
    black_box(result);
}

fn perf_bool() {
    let buffer = BitReadBuffer::new(black_box(ONES), BigEndian);
    let mut pos = 0;
    let len = buffer.bit_len();
    loop {
        if pos >= len {
            break;
        }
        let num = buffer.read_bool(pos).unwrap();
        black_box(num);
        pos += 1;
    }
}

const fn build_string_data<const N: usize>(inputs: &[&str]) -> [u8; N] {
    let mut data = [0; N];
    let mut i = 0;
    loop {
        let mut y = 0;
        while y < inputs.len() {
            let mut z = 0;
            let input = inputs[y].as_bytes();
            while z < input.len() {
                i += 1;
                if i >= N {
                    return data;
                }

                data[i] = input[z];
                z += 1;
            }
            y += 1;
        }
    }
}

const fn get_string_buffer<const N: usize>() -> [u8; N] {
    let inputs = [
        "foo\0",
        "bar\0",
        "something a little bit longer for extra testing\0",
        "a\0",
        "\0",
    ];
    build_string_data::<N>(&inputs)
}

const STRING_DATA: [u8; 10 * 1024] = get_string_buffer();

fn perf_string_be() {
    let buffer = BitReadBuffer::new(black_box(&STRING_DATA), BigEndian);

    let mut pos = 0;
    let len = buffer.bit_len();
    loop {
        if pos + (128 * 8) > len {
            break;
        }
        let result = buffer.read_string(pos, None).unwrap();
        pos += (result.len() + 1) * 8;
        black_box(result);
    }
}

fn perf_string_le() {
    let buffer = BitReadBuffer::new(black_box(&STRING_DATA), LittleEndian);

    let mut pos = 0;
    let len = buffer.bit_len();
    loop {
        if pos + (128 * 8) > len {
            break;
        }
        let result = buffer.read_string(pos, None).unwrap();
        pos += (result.len() + 1) * 8;
        black_box(result);
    }
}

fn perf_bytes_be() {
    let buffer = BitReadBuffer::new(black_box(&STRING_DATA), BigEndian);

    let mut pos = 0;
    let len = buffer.bit_len();
    loop {
        if pos + (128 * 8) > len {
            break;
        }
        let result = buffer.read_bytes(pos, 128).unwrap();
        pos += (result.len() + 1) * 8;
        black_box(result);
    }
}

fn perf_bytes_le() {
    let buffer = BitReadBuffer::new(black_box(&STRING_DATA), LittleEndian);

    let mut pos = 0;
    let len = buffer.bit_len();
    loop {
        if pos + (128 * 8) > len {
            break;
        }
        let result = buffer.read_bytes(pos, 128).unwrap();
        pos += (result.len() + 1) * 8;
        black_box(result);
    }
}

fn perf_bytes_be_unaligned() {
    let buffer = BitReadBuffer::new(black_box(&STRING_DATA), BigEndian);

    let mut pos = 3;
    let len = buffer.bit_len();
    loop {
        if pos + (128 * 8) > len {
            break;
        }
        let result = buffer.read_bytes(pos, 128).unwrap();
        pos += (result.len() + 1) * 8;
        black_box(result);
    }
}

fn perf_bytes_le_unaligned() {
    let buffer = BitReadBuffer::new(black_box(&STRING_DATA), LittleEndian);

    let mut pos = 3;
    let len = buffer.bit_len();
    loop {
        if pos + (128 * 8) > len {
            break;
        }
        let result = buffer.read_bytes(pos, 128).unwrap();
        pos += (result.len() + 1) * 8;
        black_box(result);
    }
}

#[allow(dead_code)]
#[derive(BitRead)]
struct BasicStruct {
    a: f32,
    b: bool,
    #[size = 7]
    c: u32,
}

fn perf_struct() {
    let buffer = BitReadBuffer::new(black_box(&STRING_DATA), LittleEndian);

    let mut stream: BitReadStream<LittleEndian> = buffer.clone().into();
    while stream.bits_left() > 40 {
        let result = stream.read::<BasicStruct>().unwrap();
        black_box(result);
    }
}

iai::main!(
    perf_be,
    perf_bool,
    perf_bytes_be,
    perf_bytes_be_unaligned,
    perf_bytes_le,
    perf_bytes_le_unaligned,
    perf_f32_be,
    perf_f32_le,
    perf_f64,
    perf_le,
    perf_string_be,
    perf_string_le,
    perf_struct
);
