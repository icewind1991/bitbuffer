use crate::{BitStream, Endianness, Result};

/// Trait for types that can be read from a stream without requiring the size to be configured
///
/// The `BitRead` trait can be used with `#[derive]` if all fields implement `BitRead` or [`BitReadSized`],
/// when `derive`d for structs, it will read all fields in the struct in the order they are defined in.
/// If a field only implements [`BitReadSized`] then the size needs to be defined using a field attribute.
///
/// # Examples
///
/// ```
/// use bitstream_reader::BitRead;
///
/// #[derive(BitRead)]
/// struct TestStruct {
///    foo: u8,
///    str: String,
///    #[size = 2] // when `size` is set, the attributed will be read using `read_sized`
///    truncated: String,
///    bar: u16,
///    float: f32,
///    #[size = 3]
///    asd: u8,
///    #[size_bits = 2] // first read 2 bits as unsigned integer, then use the resulting value as size for the read
///    dynamic_length: u8,
///    #[size = "asd"] // use a previously defined field as size
///    previous_field: u8,
/// }
/// ```
///
/// [`BitReadSized`]: trait.BitReadSized.html
pub trait BitRead<E: Endianness>: Sized {
    /// Read the type from stream
    fn read(stream: &mut BitStream<E>) -> Result<Self>;
}

macro_rules! impl_read_int {
    ($type:ty, $len:expr) => {
        impl<E: Endianness> BitRead<E> for $type {
            #[inline(always)]
            fn read(stream: &mut BitStream<E>) -> Result<$type> {
                stream.read_int::<$type>($len)
            }
        }
    };
}

impl_read_int!(u8, 8);
impl_read_int!(u16, 16);
impl_read_int!(u32, 32);
impl_read_int!(u64, 64);
impl_read_int!(u128, 128);
impl_read_int!(i8, 8);
impl_read_int!(i16, 16);
impl_read_int!(i32, 32);
impl_read_int!(i64, 64);
impl_read_int!(i128, 128);

impl<E: Endianness> BitRead<E> for f32 {
    #[inline(always)]
    fn read(stream: &mut BitStream<E>) -> Result<f32> {
        stream.read_float::<f32>()
    }
}

impl<E: Endianness> BitRead<E> for f64 {
    #[inline(always)]
    fn read(stream: &mut BitStream<E>) -> Result<f64> {
        stream.read_float::<f64>()
    }
}

impl<E: Endianness> BitRead<E> for bool {
    #[inline(always)]
    fn read(stream: &mut BitStream<E>) -> Result<bool> {
        stream.read_bool()
    }
}

impl<E: Endianness> BitRead<E> for String {
    #[inline(always)]
    fn read(stream: &mut BitStream<E>) -> Result<String> {
        stream.read_string(None)
    }
}

/// Trait for types that can be read from a stream, requiring the size to be configured
///
/// The meaning of the set sized depends on the type being read (e.g, number of bits for integers,
/// number of bytes for strings, number of items for Vec's, etc)
pub trait BitReadSized<E: Endianness>: Sized {
    /// Read the type from stream
    fn read(stream: &mut BitStream<E>, size: usize) -> Result<Self>;
}

macro_rules! impl_read_int_sized {
    ($type:ty) => {
        impl<E: Endianness> BitReadSized<E> for $type {
            #[inline(always)]
            fn read(stream: &mut BitStream<E>, size: usize) -> Result<$type> {
                stream.read_int::<$type>(size)
            }
        }
    };
}

impl_read_int_sized!(u8);
impl_read_int_sized!(u16);
impl_read_int_sized!(u32);
impl_read_int_sized!(u64);
impl_read_int_sized!(u128);
impl_read_int_sized!(i8);
impl_read_int_sized!(i16);
impl_read_int_sized!(i32);
impl_read_int_sized!(i64);
impl_read_int_sized!(i128);

impl<E: Endianness> BitReadSized<E> for String {
    #[inline(always)]
    fn read(stream: &mut BitStream<E>, size: usize) -> Result<String> {
        stream.read_string(Some(size))
    }
}

/// Read a boolean, if true, read `T`, else return `None`
impl<E: Endianness, T: BitRead<E>> BitRead<E> for Option<T> {
    fn read(stream: &mut BitStream<E>) -> Result<Self> {
        if stream.read()? {
            Ok(Some(stream.read()?))
        } else {
            Ok(None)
        }
    }
}

impl<E: Endianness> BitReadSized<E> for BitStream<E> {
    #[inline(always)]
    fn read(stream: &mut BitStream<E>, size: usize) -> Result<Self> {
        stream.read_bits(size)
    }
}

/// Read `T` `size` times and return as `Vec<T>`
impl<E: Endianness, T: BitRead<E>> BitReadSized<E> for Vec<T> {
    fn read(stream: &mut BitStream<E>, size: usize) -> Result<Self> {
        let mut vec = Vec::with_capacity(size);
        for _ in 0..size {
            vec.push(stream.read()?)
        }
        Ok(vec)
    }
}

// Once we have something like https://github.com/rust-lang/rfcs/issues/1053 we can do this optimization
//impl<E: Endianness> ReadSized<E> for Vec<u8> {
//    #[inline(always)]
//    fn read(stream: &mut BitStream<E>, size: usize) -> Result<Self> {
//        stream.read_bytes(size)
//    }
//}
