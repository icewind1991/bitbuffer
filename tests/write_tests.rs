use bitbuffer::{BigEndian, BitReadBuffer, BitReadStream, BitWriteStream, LittleEndian};
use std::rc::Rc;
use std::sync::Arc;

#[test]
fn test_write_bool_le() {
    let mut data = Vec::new();
    {
        let mut stream = BitWriteStream::new(&mut data, LittleEndian);

        stream.write_bool(true).unwrap();
        stream.write_bool(true).unwrap();
        stream.write_bool(false).unwrap();
        stream.write_bool(true).unwrap();
    }

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
    let mut data = Vec::new();
    {
        let mut stream = BitWriteStream::new(&mut data, BigEndian);

        stream.write_bool(true).unwrap();
        stream.write_bool(true).unwrap();
        stream.write_bool(false).unwrap();
        stream.write_bool(true).unwrap();
    }

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
    let mut data = Vec::new();
    {
        let mut stream = BitWriteStream::new(&mut data, LittleEndian);

        stream.write_bool(true).unwrap();
        stream.write_int(3253u16, 16).unwrap();
        stream.write_int(13253u64, 64).unwrap();
    }

    let mut read = BitReadStream::from(BitReadBuffer::new(&data, LittleEndian));

    assert_eq!(true, read.read_bool().unwrap());
    assert_eq!(3253u16, read.read::<u16>().unwrap());
    assert_eq!(13253u64, read.read::<u64>().unwrap());

    // 0 padded
    assert_eq!(false, read.read_bool().unwrap());
}

#[test]
fn test_write_bool_number_be() {
    let mut data = Vec::new();
    {
        let mut stream = BitWriteStream::new(&mut data, BigEndian);

        stream.write_bool(true).unwrap();
        stream.write_int(3253u16, 16).unwrap();
        stream.write_int(13253u64, 64).unwrap();
    }

    let mut read = BitReadStream::from(BitReadBuffer::new(&data, BigEndian));

    assert_eq!(1u8, read.read_int::<u8>(1).unwrap());
    assert_eq!(3253u16, read.read::<u16>().unwrap());
    assert_eq!(13253u64, read.read::<u64>().unwrap());

    // 0 padded
    assert_eq!(false, read.read_bool().unwrap());
}

#[test]
fn test_write_float_le() {
    let mut data = Vec::new();
    {
        let mut stream = BitWriteStream::new(&mut data, LittleEndian);

        stream.write_bool(true).unwrap();
        stream.write_float(3253.12f32).unwrap();
    }

    let mut read = BitReadStream::from(BitReadBuffer::new(&data, LittleEndian));

    assert_eq!(true, read.read_bool().unwrap());
    assert_eq!(3253.12f32, read.read::<f32>().unwrap());

    // 0 padded
    assert_eq!(false, read.read_bool().unwrap());
}

#[test]
fn test_write_float_be() {
    let mut data = Vec::new();
    {
        let mut stream = BitWriteStream::new(&mut data, BigEndian);

        stream.write_bool(true).unwrap();
        stream.write_float(3253.12f32).unwrap();
    }

    let mut read = BitReadStream::from(BitReadBuffer::new(&data, BigEndian));

    assert_eq!(1u8, read.read_int::<u8>(1).unwrap());
    assert_eq!(3253.12f32, read.read::<f32>().unwrap());

    // 0 padded
    assert_eq!(false, read.read_bool().unwrap());
}

#[test]
fn test_write_string_le() {
    let mut data = Vec::new();
    {
        let mut stream = BitWriteStream::new(&mut data, LittleEndian);

        stream.write_string("null terminated", None).unwrap();
        stream.write_string("fixed length1", Some(16)).unwrap();
        stream.write_string("fixed length2", Some(16)).unwrap();
    }

    let mut read = BitReadStream::from(BitReadBuffer::new(&data, LittleEndian));

    assert_eq!("null terminated", read.read_string(None).unwrap());
    assert_eq!("fixed length1", read.read_string(Some(16)).unwrap());
    assert_eq!("fixed length2", read.read_string(Some(16)).unwrap());
}

#[test]
fn test_write_string_le_unaligned() {
    let mut data = Vec::new();
    {
        let mut stream = BitWriteStream::new(&mut data, LittleEndian);

        stream.write_bool(true).unwrap();
        stream.write_string("null terminated", None).unwrap();
        stream.write_string("fixed length1", Some(16)).unwrap();
        stream.write_string("fixed length2", Some(16)).unwrap();
    }

    let mut read = BitReadStream::from(BitReadBuffer::new(&data, LittleEndian));

    assert_eq!(true, read.read_bool().unwrap());
    assert_eq!("null terminated", read.read_string(None).unwrap());
    assert_eq!("fixed length1", read.read_string(Some(16)).unwrap());
    assert_eq!("fixed length2", read.read_string(Some(16)).unwrap());

    // 0 padded
    assert_eq!(false, read.read_bool().unwrap());
}

#[test]
fn test_write_signed() {
    let mut data = Vec::new();
    {
        let mut stream = BitWriteStream::new(&mut data, LittleEndian);

        stream.write_bool(true).unwrap();
        stream.write_int(-17i32, 32).unwrap();
        stream.write_int(-9i32, 8).unwrap();
    }

    let mut read = BitReadStream::from(BitReadBuffer::new(&data, LittleEndian));

    assert_eq!(true, read.read_bool().unwrap());
    assert_eq!(-17i32, read.read_int::<i32>(32).unwrap());
    assert_eq!(-9i32, read.read_int::<i32>(8).unwrap());
}

#[test]
fn test_write_container() {
    let mut data = Vec::new();
    {
        let mut stream = BitWriteStream::new(&mut data, LittleEndian);

        stream.write(&Box::new(true)).unwrap();
        stream.write(&Rc::new(true)).unwrap();
        stream.write(&Arc::new(true)).unwrap();
    }

    let mut read = BitReadStream::from(BitReadBuffer::new(&data, LittleEndian));

    assert_eq!(Box::new(true), read.read().unwrap());
    assert_eq!(Rc::new(true), read.read().unwrap());
    assert_eq!(Arc::new(true), read.read().unwrap());
}

#[test]
fn test_write_to_slice() {
    let mut data = [0; 32];
    {
        let mut stream = unsafe { BitWriteStream::from_slice(&mut data[..], LittleEndian) };

        stream.write_bool(true).unwrap();
        stream.write_int(3253u16, 16).unwrap();
        stream.write_int(13253u64, 64).unwrap();
    }

    let mut read = BitReadStream::from(BitReadBuffer::new(&data[..], LittleEndian));

    assert_eq!(true, read.read_bool().unwrap());
    assert_eq!(3253u16, read.read::<u16>().unwrap());
    assert_eq!(13253u64, read.read::<u64>().unwrap());

    // 0 padded
    assert_eq!(false, read.read_bool().unwrap());
}
