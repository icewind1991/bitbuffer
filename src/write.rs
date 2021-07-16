use crate::{BitReadStream, BitWriteStream, Endianness, Result};
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
/// will be used, otherwise [`write_read()`][write] will be used.
///
/// The size for a field can be set using 3 different methods
///  - set the size as an integer using the `size` attribute,
///  - use a previously defined field as the size using the `size` attribute
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
///     #[size = "asd"] // use a previously defined field as size
///     previous_field: u8,
/// }
/// ```
///
/// # Enums
///
/// The implementation can be derived for an enum as long as every variant of the enum either has no field, or an unnamed field that implements `BitWrite` or [`BitWriteSized`]
///
/// The enum is written by first writing a set number of bits as the discriminant of the enum, then the variant written.
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
/// [write_sized]: BitWriteStream::write_sized
/// [write]: BitWriteStream::write
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

impl<E: Endianness, T: BitWrite<E>, const N: usize> BitWrite<E> for [T; N] {
    #[inline]
    fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
        for element in self.iter() {
            stream.write(element)?;
        }
        Ok(())
    }
}

impl<T: BitWrite<E>, E: Endianness> BitWrite<E> for Box<T> {
    #[inline]
    fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
        stream.write(self.as_ref())
    }
}

impl<T: BitWrite<E>, E: Endianness> BitWrite<E> for Rc<T> {
    #[inline]
    fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
        stream.write(self.as_ref())
    }
}

impl<T: BitWrite<E>, E: Endianness> BitWrite<E> for Arc<T> {
    #[inline]
    fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
        stream.write(self.as_ref())
    }
}

macro_rules! impl_write_tuple {
    ($($i:tt: $type:ident),*) => {
        impl<'a, E: Endianness, $($type: BitWrite<E>),*> BitWrite<E> for ($($type),*) {
            #[inline]
            fn write(&self, stream: &mut BitWriteStream<E>) -> Result<()> {
                $(self.$i.write(stream)?;)*
                Ok(())
            }
        }
    };
}

impl_write_tuple!(0: T1, 1: T2);
impl_write_tuple!(0: T1, 1: T2, 2: T3);
impl_write_tuple!(0: T1, 1: T2, 2: T3, 3: T4);

/// Trait for types that can be written to a stream, requiring the size to be configured
///
/// The meaning of the set sized depends on the type being written (e.g, number of bits for integers,
/// number of bytes for strings, number of items for Vec's, etc)
///
/// The `BitReadSized` trait can be used with `#[derive]` on structs
///
/// The implementation can be derived for a struct as long as every field in the struct implements [`BitWrite`] or `BitWriteSized`
///
/// The struct is written field by field in the order they are defined in, if the size for a field is set [`stream.write_sized()`][write_sized]
/// will be used, otherwise [`stream.write()`][write] will be used.
///
/// The size for a field can be set using 4 different methods
///  - set the size as an integer using the `size` attribute,
///  - use a previously defined field as the size using the `size` attribute
///  - based on the input size by setting `size` attribute to `"input_size"`
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
/// The enum is written by first writing a set number of bits as the discriminant of the enum, then the variant is written.
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
/// [write_sized]: BitReadStream::write_sized
/// [write]: BitReadStream::write
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
impl_write_sized_int!(usize);
impl_write_sized_int!(i8);
impl_write_sized_int!(i16);
impl_write_sized_int!(i32);
impl_write_sized_int!(i64);
impl_write_sized_int!(i128);
impl_write_sized_int!(isize);

impl<E: Endianness> BitWriteSized<E> for BitReadStream<'_, E> {
    #[inline]
    fn write_sized(&self, stream: &mut BitWriteStream<E>, len: usize) -> Result<()> {
        let bits = self.clone().read_bits(len)?;
        stream.write_bits(&bits)
    }
}

impl<E: Endianness, T: BitWriteSized<E>, const N: usize> BitWriteSized<E> for [T; N] {
    #[inline]
    fn write_sized(&self, stream: &mut BitWriteStream<E>, len: usize) -> Result<()> {
        for element in self.iter() {
            stream.write_sized(element, len)?;
        }
        Ok(())
    }
}

impl<T: BitWriteSized<E>, E: Endianness> BitWriteSized<E> for Box<T> {
    #[inline]
    fn write_sized(&self, stream: &mut BitWriteStream<E>, len: usize) -> Result<()> {
        stream.write_sized(self.as_ref(), len)
    }
}

impl<T: BitWriteSized<E>, E: Endianness> BitWriteSized<E> for Rc<T> {
    #[inline]
    fn write_sized(&self, stream: &mut BitWriteStream<E>, len: usize) -> Result<()> {
        stream.write_sized(self.as_ref(), len)
    }
}

impl<T: BitWriteSized<E>, E: Endianness> BitWriteSized<E> for Arc<T> {
    #[inline]
    fn write_sized(&self, stream: &mut BitWriteStream<E>, len: usize) -> Result<()> {
        stream.write_sized(self.as_ref(), len)
    }
}
