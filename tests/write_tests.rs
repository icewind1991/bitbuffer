use bitbuffer::num_traits::{IsSigned, SplitFitUsize, UncheckedPrimitiveInt};
use bitbuffer::{BigEndian, BitReadBuffer, BitReadStream, BitWriteStream, LittleEndian};
use num_traits::{PrimInt, WrappingSub};
use std::any::type_name;
use std::fmt::Debug;
use std::mem::size_of;
use std::ops::BitOrAssign;
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

    assert!(read.read_bool().unwrap());
    assert!(read.read_bool().unwrap());
    assert!(!read.read_bool().unwrap());
    assert!(read.read_bool().unwrap());

    // 0 padded
    assert!(!read.read_bool().unwrap());
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

    assert!(read.read_bool().unwrap());
    assert!(read.read_bool().unwrap());
    assert!(!read.read_bool().unwrap());
    assert!(read.read_bool().unwrap());

    // 0 padded
    assert!(!read.read_bool().unwrap());
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

    assert!(read.read_bool().unwrap());
    assert_eq!(3253u16, read.read::<u16>().unwrap());
    assert_eq!(13253u64, read.read::<u64>().unwrap());

    // 0 padded
    assert!(!read.read_bool().unwrap());
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
    assert!(!read.read_bool().unwrap());
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

    assert!(read.read_bool().unwrap());
    assert_eq!(3253.12f32, read.read::<f32>().unwrap());

    // 0 padded
    assert!(!read.read_bool().unwrap());
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
    assert!(!read.read_bool().unwrap());
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

    assert!(read.read_bool().unwrap());
    assert_eq!("null terminated", read.read_string(None).unwrap());
    assert_eq!("fixed length1", read.read_string(Some(16)).unwrap());
    assert_eq!("fixed length2", read.read_string(Some(16)).unwrap());

    // 0 padded
    assert!(!read.read_bool().unwrap());
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

    assert!(read.read_bool().unwrap());
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
        let mut stream = BitWriteStream::from_slice(&mut data[..], LittleEndian);

        stream.write_bool(true).unwrap();
        stream.write_int(3253u16, 16).unwrap();
        stream.write_int(13253u64, 64).unwrap();
    }

    let mut read = BitReadStream::from(BitReadBuffer::new(&data[..], LittleEndian));

    assert!(read.read_bool().unwrap());
    assert_eq!(3253u16, read.read::<u16>().unwrap());
    assert_eq!(13253u64, read.read::<u64>().unwrap());

    // 0 padded
    assert!(!read.read_bool().unwrap());
}

#[test]
fn test_write_last_slice() {
    let mut data = [0; 1];
    {
        let mut stream = BitWriteStream::from_slice(&mut data[..], LittleEndian);

        stream.write_int::<u8>(0b1000, 4).unwrap();
        stream.write_bool(true).unwrap();
    }

    let mut read = BitReadStream::from(BitReadBuffer::new(&data[..], LittleEndian));

    assert_eq!(0b1000, read.read_int::<u8>(4).unwrap());
    assert!(read.read_bool().unwrap());
}

#[test]
fn test_write_be_long() {
    let mut bytes = vec![];
    let mut writer = BitWriteStream::new(&mut bytes, BigEndian);
    let num1 = 0b11000_00111110u64;
    let num2 = 0b1111111_11100100_00100100_11011101_00000011_11100000_01100111_11011011u64;
    writer.write_int(num1, 13).unwrap();
    writer.write_int(num2, 63).unwrap();

    let buffer = BitReadBuffer::new(&bytes, BigEndian);
    let mut reader = BitReadStream::new(buffer);
    let num1actual = reader.read_int::<u64>(13).unwrap();
    let num2actual = reader.read_int::<u64>(63).unwrap();

    assert_eq!(num1actual, num1);
    assert_eq!(num2actual, num2);
}

#[test]
fn test_write_all_lengths() {
    let pattern = 0b10101010u8;
    test_write_all_lengths_ty::<u8>(pattern);
    test_write_all_lengths_ty::<u16>(u16::from_le_bytes([pattern; 2]));
    test_write_all_lengths_ty::<u32>(u32::from_le_bytes([pattern; 4]));
    test_write_all_lengths_ty::<u64>(u64::from_le_bytes([pattern; 8]));
    test_write_all_lengths_ty::<u128>(u128::from_le_bytes([pattern; 16]));
    test_write_all_lengths_ty::<usize>(usize::from_le_bytes([pattern; size_of::<usize>()]));

    test_write_all_lengths_ty::<i8>(i8::from_le_bytes([pattern; 1]));
    test_write_all_lengths_ty::<i16>(i16::from_le_bytes([pattern; 2]));
    test_write_all_lengths_ty::<i32>(i32::from_le_bytes([pattern; 4]));
    test_write_all_lengths_ty::<i64>(i64::from_le_bytes([pattern; 8]));
    test_write_all_lengths_ty::<i128>(i128::from_le_bytes([pattern; 16]));
    test_write_all_lengths_ty::<isize>(isize::from_le_bytes([pattern; size_of::<isize>()]));
}

fn test_write_all_lengths_ty<
    T: PrimInt + BitOrAssign + IsSigned + UncheckedPrimitiveInt + Debug + SplitFitUsize + WrappingSub,
>(
    pattern: T,
) {
    let max_bits = size_of::<T>() * 8;
    let mut bytes = Vec::new();
    let mut writer = BitWriteStream::new(&mut bytes, BigEndian);

    let mut expected = Vec::<T>::new();

    for bits in 1..max_bits {
        let value = pattern >> (max_bits - bits);
        expected.push(value);
        writer.write_int(value, bits).unwrap();
    }

    let buffer = BitReadBuffer::new(&bytes, BigEndian);
    let mut reader = BitReadStream::new(buffer);

    for (bits, expected_value) in (1..max_bits).zip(expected.into_iter()) {
        let actual = reader.read_int::<T>(bits).unwrap();
        assert_eq!(
            expected_value,
            actual,
            "write + read for {} bits {}",
            bits,
            type_name::<T>()
        );
    }
}
