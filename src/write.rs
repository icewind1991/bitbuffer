use crate::endianness::{BigEndian, LittleEndian};
use crate::{BitWriteStream, Endianness, Result};
use std::mem::size_of;
use std::rc::Rc;
use std::sync::Arc;

/// Trait for types that can be written to a stream without requiring the size to be configured
///
/// The `BitWrite` trait can be used with `#[derive]` on structs and enums
///
/// # Structs
///
/// The implementation can be derived for a struct as long as every field in the struct implements `BitWrite` or [`BitWriteSized`]
///
/// The struct is written field by field in the order they are defined in, if the size for a field is set [`stream.write_sized()`][write_sized]
/// will be used, otherwise [`stream_write()`][write] will be used.
///
/// The size for a field can be set using 3 different methods
///  - set the size as an integer using the `size` attribute,
///  - use a previously defined field as the size using the `size` attribute
///  - read a set number of bits as an integer, using the resulting value as size using the `size_bits` attribute
///
/// ## Examples
///
/// ```
/// # use bitbuffer::BitWrite;
/// #
/// #[derive(BitWrite)]
/// struct TestStruct {
///     foo: u8,
///     str: String,
///     #[size = 2] // when `size` is set, the attributed will be read using `read_sized`
///     truncated: String,
///     bar: u16,
///     float: f32,
///     #[size = 3]
///     asd: u8,
///     #[size_bits = 2] // first read 2 bits as unsigned integer, then use the resulting value as size for the read
///     dynamic_length: u8,
///     #[size = "asd"] // use a previously defined field as size
///     previous_field: u8,
/// }
/// ```
///
/// # Enums
///
/// The implementation can be derived for an enum as long as every variant of the enum either has no field, or an unnamed field that implements `BitWrite` or [`BitWriteSized`]
///
/// The enum is written by first writing a set number of bits as the discriminant of the enum, then the variant for the written discriminant is read.
///
/// For details about setting the input size for fields implementing [`BitWriteSized`] see the block about size in the `Structs` section above.
///
/// The discriminant for the variants defaults to incrementing by one for every field, starting with `0`.
/// You can overwrite the discriminant for a field, which will also change the discriminant for every following field.
///
/// ## Examples
///
/// ```
/// # use bitbuffer::BitWrite;
/// #
/// #[derive(BitWrite)]
/// #[discriminant_bits = 2]
/// enum TestBareEnum {
///     Foo,
///     Bar,
///     Asd = 3, // manually set the discriminant value for a field
/// }
/// ```
///
/// ```
/// # use bitbuffer::BitWrite;
/// #
/// #[derive(BitWrite)]
/// #[discriminant_bits = 2]
/// enum TestUnnamedFieldEnum {
///     #[size = 5]
///     Foo(i8),
///     Bar(bool),
///     #[discriminant = 3] // since rust only allows setting the discriminant on field-less enums, you can use an attribute instead
///     Asd(u8),
/// }
/// ```
///
/// [`BitWriteSized`]: trait.BitWriteSized.html
/// [write_sized]: struct.BitWriteStream.html#method.write_sized
/// [write]: struct.BitWriteStream.html#method.write
pub trait BitWrite<E: Endianness> {
    /// Write the type to the stream
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

macro_rules! impl_write_int_nonzero {
    ($type:ty) => {
        impl BitWrite<LittleEndian> for Option<$type> {
            #[inline]
            fn write(&self, stream: &mut BitWriteStream<LittleEndian>) -> Result<()> {
                BitWrite::write(&self.map(<$type>::get).unwrap_or(0), stream)
            }
        }

        impl BitWrite<BigEndian> for Option<$type> {
            #[inline]
            fn write(&self, stream: &mut BitWriteStream<BigEndian>) -> Result<()> {
                BitWrite::write(&self.map(<$type>::get).unwrap_or(0), stream)
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

impl_write_int_nonzero!(std::num::NonZeroU8);
impl_write_int_nonzero!(std::num::NonZeroU16);
impl_write_int_nonzero!(std::num::NonZeroU32);
impl_write_int_nonzero!(std::num::NonZeroU64);
impl_write_int_nonzero!(std::num::NonZeroU128);

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

impl<E: Endianness> BitWrite<E> for String {
    #[inline]
    fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
        stream.write_string(self, None)
    }
}

impl<E: Endianness> BitWrite<E> for str {
    #[inline]
    fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
        stream.write_string(self, None)
    }
}

impl<E: Endianness, T: BitWrite<E>> BitWrite<E> for Rc<T> {
    #[inline]
    fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
        T::write(self, stream)
    }
}

impl<E: Endianness, T: BitWrite<E>> BitWrite<E> for Arc<T> {
    #[inline]
    fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
        T::write(self, stream)
    }
}

impl<E: Endianness, T: BitWrite<E>> BitWrite<E> for Box<T> {
    #[inline]
    fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
        T::write(self, stream)
    }
}

macro_rules! impl_write_tuple {
    ($($type:ident),*) => {
        impl<E: Endianness, $($type: BitWrite<E>),*> BitWrite<E> for ($($type),*) {
            #[inline]
            fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
                #[allow(non_snake_case)]
                let ($($type),*) = self;

                ($($type.write(stream)?),*);
                Ok(())
            }
        }
    };
}

impl_write_tuple!(T1, T2);
impl_write_tuple!(T1, T2, T3);
impl_write_tuple!(T1, T2, T3, T4);

/// Trait for types that can be written from a stream, requiring the size to be configured
///
/// The meaning of the set sized depends on the type being written (e.g, number of bits for integers,
/// number of bytes for strings, etc)
///
/// The `BitWriteSized` trait can be used with `#[derive]` on structs
///
/// The implementation can be derived for a struct as long as every field in the struct implements [`BitWrite`] or `BitWriteSized`
///
/// The struct is written field by field in the order they are defined in, if the size for a field is set [`stream.write_sized()`][write_sized]
/// will be used, otherwise [`stream_write()`][write] will be used.
///
/// The size for a field can be set using 4 different methods
///  - set the size as an integer using the `size` attribute,
///  - use a previously defined field as the size using the `size` attribute
///  - based on the input size by setting `size` attribute to `"input_size"`
///  - read a set number of bits as an integer, using the resulting value as size using the `size_bits` attribute
///
/// ## Examples
///
/// ```
/// # use bitbuffer::BitWriteSized;
/// #
/// #[derive(BitWriteSized, PartialEq, Debug)]
/// struct TestStructSized {
///     foo: u8,
///     #[size = "input_size"]
///     string: String,
///     #[size = "input_size"]
///     int: u8,
/// }
/// ```
///
/// # Enums
///
/// The implementation can be derived for an enum as long as every variant of the enum either has no field, or an unnamed field that implements [`BitWrite`] or `BitWriteSized`
///
/// The enum is written by first reading a set number of bits as the discriminant of the enum, then the variant for the written discriminant is read.
///
/// For details about setting the input size for fields implementing `BitWriteSized` see the block about size in the `Structs` section above.
///
/// The discriminant for the variants defaults to incrementing by one for every field, starting with `0`.
/// You can overwrite the discriminant for a field, which will also change the discriminant for every following field.
///
/// ## Examples
///
/// ```
/// # use bitbuffer::BitWriteSized;
/// #
/// #[derive(BitWriteSized)]
/// #[discriminant_bits = 2]
/// enum TestUnnamedFieldEnum {
///     #[size = 5]
///     Foo(i8),
///     Bar(bool),
///     #[discriminant = 3] // since rust only allows setting the discriminant on field-less enums, you can use an attribute instead
///     #[size = "input_size"]
///     Asd(u8),
/// }
/// ```
///
/// [`BitWrite`]: trait.BitWrite.html
/// [read_sized]: struct.BitStream.html#method.read_sized
/// [read]: struct.BitStream.html#method.read
pub trait BitWriteSized<E: Endianness> {
    /// Write the type from stream
    fn write(&self, stream: &mut BitWriteStream<E>, size: usize) -> Result<()>;
}

macro_rules! impl_write_int_sized {
    ( $ type: ty) => {
        impl<E: Endianness> BitWriteSized<E> for $type {
            #[inline]
            fn write(&self, stream: &mut BitWriteStream<E>, size: usize) -> Result<()> {
                stream.write_int::<$type>(*self, size)
            }
        }
    };
}

impl_write_int_sized!(u8);
impl_write_int_sized!(u16);
impl_write_int_sized!(u32);
impl_write_int_sized!(u64);
impl_write_int_sized!(u128);
impl_write_int_sized!(i8);
impl_write_int_sized!(i16);
impl_write_int_sized!(i32);
impl_write_int_sized!(i64);
impl_write_int_sized!(i128);

impl<E: Endianness> BitWriteSized<E> for String {
    #[inline]
    fn write(&self, stream: &mut BitWriteStream<E>, size: usize) -> Result<()> {
        stream.write_string(self, Some(size))
    }
}

impl<E: Endianness> BitWriteSized<E> for str {
    #[inline]
    fn write(&self, stream: &mut BitWriteStream<E>, size: usize) -> Result<()> {
        stream.write_string(self, Some(size))
    }
}

impl<E: Endianness, T: BitWrite<E>> BitWrite<E> for Option<T> {
    fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
        match self.as_ref() {
            Some(inner) => {
                stream.write_bool(true)?;
                T::write(inner, stream)
            }
            None => stream.write_bool(false),
        }
    }
}

impl<E: Endianness, T: BitWriteSized<E>> BitWriteSized<E> for Option<T> {
    fn write(&self, stream: &mut BitWriteStream<E>, size: usize) -> Result<()> {
        match self.as_ref() {
            Some(inner) => {
                stream.write_bool(true)?;
                T::write(inner, stream, size)
            }
            None => stream.write_bool(false),
        }
    }
}
