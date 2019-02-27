use bitstream_reader::{BigEndian, BitBuffer, BitStream, LittleEndian, Read};

// for bench on nightly
//use std::fs;
//use test::Bencher;

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
}

#[derive(Read, PartialEq, Debug)]
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

//0b1011_0101,
//    0b0110_1010,
//    0b1010_1100,
//    0b1001_1001,
//    0b1001_1001,
//    0b1001_1001,
//    0b1001_1001,
//    0b1110_0111,
//    0b1001_1001,
//    0b1001_1001,
//    0b1001_1001,
//    0b1110_0111,

// for bench on nightly
//fn read_perf<P: IsPadded>(buffer: BitBuffer<LittleEndian, P>) -> u16 {
//    let size = 5;
//    let mut pos = 0;
//    let len = buffer.bit_len();
//    let mut result: u16 = 0;
//    loop {
//        if pos + size > len {
//            return result;
//        }
//        let data = buffer.read::<u16>(pos, size).unwrap();
//        result = result.wrapping_add(data);
//        pos += size;
//    }
//}
//
//#[bench]
//fn perf_padded(b: &mut Bencher) {
//    let mut file = fs::read("/bulk/tmp/test.dem").expect("Unable to read file");
//    let len = file.len();
//    file.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0]);
//    let bytes = file.as_slice();
//    b.iter(|| {
//        let buffer = BitBuffer::from_padded_slice(&bytes, len, LittleEndian);
//        let data = read_perf(buffer);
//        assert_eq!(data, 43943);
//        test::black_box(data);
//    });
//}
//
//#[bench]
//fn perf_non_padded(b: &mut Bencher) {
//    let file = fs::read("/bulk/tmp/test.dem").expect("Unable to read file");
//    let bytes = file.as_slice();
//    b.iter(|| {
//        let buffer = BitBuffer::new(&bytes, LittleEndian);
//        let data = read_perf(buffer);
//        assert_eq!(data, 43943);
//        test::black_box(data);
//    });
//}
