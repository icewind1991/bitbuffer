use bitbuffer::{BigEndian, BitReadBuffer, BitReadStream, BitWriteStream, LittleEndian};

#[test]
fn test_write_bool_le() {
    let mut stream = BitWriteStream::new(LittleEndian);

    stream.write_bool(true).unwrap();
    stream.write_bool(true).unwrap();
    stream.write_bool(false).unwrap();
    stream.write_bool(true).unwrap();

    let data = stream.finish();
    let mut read = BitReadStream::from(BitReadBuffer::new(&data, LittleEndian));

    assert_eq!(true, read.read_bool().unwrap());
    assert_eq!(true, read.read_bool().unwrap());
    assert_eq!(false, read.read_bool().unwrap());
    assert_eq!(true, read.read_bool().unwrap());

    // 0 padded
    assert_eq!(false, read.read_bool().unwrap());
}

#[test]
fn test_write_bool_be() {
    let mut stream = BitWriteStream::new(BigEndian);

    stream.write_bool(true).unwrap();
    stream.write_bool(true).unwrap();
    stream.write_bool(false).unwrap();
    stream.write_bool(true).unwrap();

    let data = stream.finish();
    let mut read = BitReadStream::from(BitReadBuffer::new(&data, BigEndian));

    assert_eq!(true, read.read_bool().unwrap());
    assert_eq!(true, read.read_bool().unwrap());
    assert_eq!(false, read.read_bool().unwrap());
    assert_eq!(true, read.read_bool().unwrap());

    // 0 padded
    assert_eq!(false, read.read_bool().unwrap());
}

#[test]
fn test_write_bool_number_le() {
    let mut stream = BitWriteStream::new(LittleEndian);

    stream.write_bool(true).unwrap();
    stream.write_int(3253u16, 16).unwrap();
    stream.write_int(13253u64, 64).unwrap();

    let data = stream.finish();
    let mut read = BitReadStream::from(BitReadBuffer::new(&data, LittleEndian));

    assert_eq!(true, read.read_bool().unwrap());
    assert_eq!(3253u16, read.read().unwrap());
    assert_eq!(13253u64, read.read().unwrap());

    // 0 padded
    assert_eq!(false, read.read_bool().unwrap());
}

#[test]
fn test_write_bool_number_be() {
    let mut stream = BitWriteStream::new(BigEndian);

    stream.write_bool(true).unwrap();
    stream.write_int(3253u16, 16).unwrap();
    stream.write_int(13253u64, 64).unwrap();

    let data = stream.finish();
    let mut read = BitReadStream::from(BitReadBuffer::new(&data, BigEndian));

    assert_eq!(1u8, read.read_int(1).unwrap());
    assert_eq!(3253u16, read.read().unwrap());
    assert_eq!(13253u64, read.read().unwrap());

    // 0 padded
    assert_eq!(false, read.read_bool().unwrap());
}

#[test]
fn test_write_float_le() {
    let mut stream = BitWriteStream::new(LittleEndian);

    stream.write_bool(true).unwrap();
    stream.write_float(3253.12f32).unwrap();

    let data = stream.finish();
    let mut read = BitReadStream::from(BitReadBuffer::new(&data, LittleEndian));

    assert_eq!(true, read.read_bool().unwrap());
    assert_eq!(3253.12f32, read.read().unwrap());

    // 0 padded
    assert_eq!(false, read.read_bool().unwrap());
}

#[test]
fn test_write_float_be() {
    let mut stream = BitWriteStream::new(BigEndian);

    stream.write_bool(true).unwrap();
    stream.write_float(3253.12f32).unwrap();

    let data = stream.finish();
    let mut read = BitReadStream::from(BitReadBuffer::new(&data, BigEndian));

    assert_eq!(1u8, read.read_int(1).unwrap());
    assert_eq!(3253.12f32, read.read().unwrap());

    // 0 padded
    assert_eq!(false, read.read_bool().unwrap());
}

#[test]
fn test_write_string_le() {
    let mut stream = BitWriteStream::new(LittleEndian);

    stream.write_bool(true).unwrap();
    stream.write_string("null terminated", None).unwrap();
    stream.write_string("fixed length1", Some(16)).unwrap();
    stream.write_string("fixed length2", Some(16)).unwrap();

    let data = stream.finish();
    let mut read = BitReadStream::from(BitReadBuffer::new(&data, LittleEndian));

    assert_eq!(true, read.read_bool().unwrap());
    assert_eq!("null terminated", read.read_string(None).unwrap());
    assert_eq!("fixed length1", read.read_string(Some(16)).unwrap());
    assert_eq!("fixed length2", read.read_string(Some(16)).unwrap());

    // 0 padded
    assert_eq!(false, read.read_bool().unwrap());
}
