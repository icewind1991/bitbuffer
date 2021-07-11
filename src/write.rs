use crate::{BitReadStream, BitWriteStream, Endianness, Result};
use std::mem::size_of;

/// Trait for types that can be written to a stream without requiring the size to be configured
pub trait BitWrite<E: Endianness> {
    /// Write the type to stream
    fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()>;
}

macro_rules! impl_write_int {
    ($type:ty) => {
        impl<E: Endianness> BitWrite<E> for $type {
            #[inline]
            fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
                stream.write_int::<$type>(*self, size_of::<$type>() * 8)
            }
        }
    };
}

impl_write_int!(u8);
impl_write_int!(u16);
impl_write_int!(u32);
impl_write_int!(u64);
impl_write_int!(u128);
impl_write_int!(i8);
impl_write_int!(i16);
impl_write_int!(i32);
impl_write_int!(i64);
impl_write_int!(i128);

impl<E: Endianness> BitWrite<E> for f32 {
    #[inline]
    fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
        stream.write_float::<f32>(*self)
    }
}

impl<E: Endianness> BitWrite<E> for f64 {
    #[inline]
    fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
        stream.write_float::<f64>(*self)
    }
}

impl<E: Endianness> BitWrite<E> for bool {
    #[inline]
    fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
        stream.write_bool(*self)
    }
}

impl<E: Endianness> BitWrite<E> for str {
    #[inline]
    fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
        stream.write_string(self, None)
    }
}

impl<E: Endianness> BitWrite<E> for String {
    #[inline]
    fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
        stream.write_string(self, None)
    }
}

impl<E: Endianness> BitWrite<E> for BitReadStream<'_, E> {
    #[inline]
    fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
        stream.write_bits(self)
    }
}

/// Trait for types that can be written to a stream, requiring the size to be configured
pub trait BitWriteSized<E: Endianness> {
    /// Write the type to stream
    fn write_sized(&self, stream: &mut BitWriteStream<E>, len: usize) -> Result<()>;
}

impl<E: Endianness> BitWriteSized<E> for str {
    #[inline]
    fn write_sized(&self, stream: &mut BitWriteStream<E>, len: usize) -> Result<()> {
        stream.write_string(self, Some(len))
    }
}

impl<E: Endianness> BitWriteSized<E> for String {
    #[inline]
    fn write_sized(&self, stream: &mut BitWriteStream<E>, len: usize) -> Result<()> {
        stream.write_string(self, Some(len))
    }
}

macro_rules! impl_write_sized_int {
    ($type:ty) => {
        impl<E: Endianness> BitWriteSized<E> for $type {
            #[inline]
            fn write_sized(&self, stream: &mut BitWriteStream<E>, len: usize) -> Result<()> {
                stream.write_int::<$type>(*self, len)
            }
        }
    };
}

impl_write_sized_int!(u8);
impl_write_sized_int!(u16);
impl_write_sized_int!(u32);
impl_write_sized_int!(u64);
impl_write_sized_int!(u128);
impl_write_sized_int!(i8);
impl_write_sized_int!(i16);
impl_write_sized_int!(i32);
impl_write_sized_int!(i64);
impl_write_sized_int!(i128);

impl<E: Endianness> BitWriteSized<E> for BitReadStream<'_, E> {
    #[inline]
    fn write_sized(&self, stream: &mut BitWriteStream<E>, len: usize) -> Result<()> {
        let bits = self.clone().read_bits(len)?;
        stream.write_bits(&bits)
    }
}
