use crate::endianness::{BigEndian, LittleEndian};
use crate::{BitStream, Endianness, Result};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::size_of;
use std::rc::Rc;
use std::sync::Arc;

/// Trait for types that can be read from a stream without requiring the size to be configured
///
/// The `BitRead` trait can be used with `#[derive]` on structs and enums
///
/// # Structs
///
/// The implementation can be derived for a struct as long as every field in the struct implements `BitRead` or [`BitReadSized`]
///
/// The struct is read field by field in the order they are defined in, if the size for a field is set [`stream.read_sized()`][read_sized]
/// will be used, otherwise [`stream_read()`][read] will be used.
///
/// The size for a field can be set using 3 different methods
///  - set the size as an integer using the `size` attribute,
///  - use a previously defined field as the size using the `size` attribute
///  - read a set number of bits as an integer, using the resulting value as size using the `size_bits` attribute
///
/// ## Examples
///
/// ```
/// # use bitstream_reader_derive::BitRead;
/// #
/// #[derive(BitRead)]
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
/// The implementation can be derived for an enum as long as every variant of the enum either has no field, or an unnamed field that implements `BitRead` or [`BitReadSized`]
///
/// The enum is read by first reading a set number of bits as the discriminant of the enum, then the variant for the read discriminant is read.
///
/// For details about setting the input size for fields implementing [`BitReadSized`] see the block about size in the `Structs` section above.
///
/// The discriminant for the variants defaults to incrementing by one for every field, starting with `0`.
/// You can overwrite the discriminant for a field, which will also change the discriminant for every following field.
///
/// ## Examples
///
/// ```
/// # use bitstream_reader_derive::BitRead;
/// #
/// #[derive(BitRead)]
/// #[discriminant_bits = 2]
/// enum TestBareEnum {
///     Foo,
///     Bar,
///     Asd = 3, // manually set the discriminant value for a field
/// }
/// ```
///
/// ```
/// # use bitstream_reader_derive::BitRead;
/// #
/// #[derive(BitRead)]
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
/// [`BitReadSized`]: trait.BitReadSized.html
/// [read_sized]: struct.BitStream.html#method.read_sized
/// [read]: struct.BitStream.html#method.read
pub trait BitRead<E: Endianness>: Sized {
    /// Read the type from stream
    fn read(stream: &mut BitStream<E>) -> Result<Self>;

    /// The number of bits that will be read or None if the number of bits will change depending
    /// on the bit stream
    #[inline(always)]
    fn bit_size() -> Option<usize> {
        return None;
    }
}

/// Trait to allow skipping a type
///
/// This might be faster than trying to read it
pub trait BitSkip<E: Endianness>: BitRead<E> {
    /// Skip the type
    fn skip(stream: &mut BitStream<E>) -> Result<()> {
        match Self::bit_size() {
            Some(size) => stream.skip_bits(size),
            None => Self::read(stream).map(|_| ()),
        }
    }
}

macro_rules! impl_read_int {
    ($type:ty) => {
        impl<E: Endianness> BitRead<E> for $type {
            #[inline]
            fn read(stream: &mut BitStream<E>) -> Result<$type> {
                stream.read_int::<$type>(size_of::<$type>() * 8)
            }

            #[inline(always)]
            fn bit_size() -> Option<usize> {
                Some(size_of::<$type>() * 8)
            }
        }
    };
}

macro_rules! impl_read_int_nonzero {
    ($type:ty) => {
        impl BitRead<LittleEndian> for Option<$type> {
            #[inline]
            fn read(stream: &mut BitStream<LittleEndian>) -> Result<Self> {
                Ok(<$type>::new(stream.read()?))
            }

            #[inline(always)]
            fn bit_size() -> Option<usize> {
                Some(size_of::<$type>() * 8)
            }
        }

        impl BitRead<BigEndian> for Option<$type> {
            #[inline]
            fn read(stream: &mut BitStream<BigEndian>) -> Result<Self> {
                Ok(<$type>::new(stream.read()?))
            }

            #[inline(always)]
            fn bit_size() -> Option<usize> {
                Some(size_of::<$type>() * 8)
            }
        }
    };
}

impl_read_int!(u8);
impl_read_int!(u16);
impl_read_int!(u32);
impl_read_int!(u64);
impl_read_int!(u128);
impl_read_int!(i8);
impl_read_int!(i16);
impl_read_int!(i32);
impl_read_int!(i64);
impl_read_int!(i128);

impl_read_int_nonzero!(std::num::NonZeroU8);
impl_read_int_nonzero!(std::num::NonZeroU16);
impl_read_int_nonzero!(std::num::NonZeroU32);
impl_read_int_nonzero!(std::num::NonZeroU64);
impl_read_int_nonzero!(std::num::NonZeroU128);

impl<E: Endianness> BitRead<E> for f32 {
    #[inline]
    fn read(stream: &mut BitStream<E>) -> Result<f32> {
        stream.read_float::<f32>()
    }

    #[inline(always)]
    fn bit_size() -> Option<usize> {
        Some(32)
    }
}

impl<E: Endianness> BitRead<E> for f64 {
    #[inline]
    fn read(stream: &mut BitStream<E>) -> Result<f64> {
        stream.read_float::<f64>()
    }

    #[inline(always)]
    fn bit_size() -> Option<usize> {
        Some(64)
    }
}

impl<E: Endianness> BitRead<E> for bool {
    #[inline]
    fn read(stream: &mut BitStream<E>) -> Result<bool> {
        stream.read_bool()
    }

    #[inline(always)]
    fn bit_size() -> Option<usize> {
        Some(1)
    }
}

impl<E: Endianness> BitRead<E> for String {
    #[inline]
    fn read(stream: &mut BitStream<E>) -> Result<String> {
        stream.read_string(None)
    }
}

impl<E: Endianness, T: BitRead<E>> BitRead<E> for Rc<T> {
    #[inline]
    fn read(stream: &mut BitStream<E>) -> Result<Self> {
        Ok(Rc::new(T::read(stream)?))
    }

    #[inline(always)]
    fn bit_size() -> Option<usize> {
        T::bit_size()
    }
}

impl<E: Endianness, T: BitRead<E>> BitRead<E> for Arc<T> {
    #[inline]
    fn read(stream: &mut BitStream<E>) -> Result<Self> {
        Ok(Arc::new(T::read(stream)?))
    }

    #[inline(always)]
    fn bit_size() -> Option<usize> {
        T::bit_size()
    }
}

impl<E: Endianness, T: BitRead<E>> BitRead<E> for Box<T> {
    #[inline]
    fn read(stream: &mut BitStream<E>) -> Result<Self> {
        Ok(Box::new(T::read(stream)?))
    }

    #[inline(always)]
    fn bit_size() -> Option<usize> {
        T::bit_size().and_then(|sum| T::bit_size().map(|size| sum + size))
    }
}

macro_rules! impl_read_tuple {
    ($($type:ident),*) => {
        impl<E: Endianness, $($type: BitRead<E>),*> BitRead<E> for ($($type),*) {
            #[inline]
            fn read(stream: &mut BitStream<E>) -> Result<Self> {
                Ok(($(<$type>::read(stream)?),*))
            }

            #[inline(always)]
            fn bit_size() -> Option<usize> {
                Some(0)$(.and_then(|sum| <$type>::bit_size().map(|size| sum + size)))*
            }
        }
    };
}

impl_read_tuple!(T1, T2);
impl_read_tuple!(T1, T2, T3);
impl_read_tuple!(T1, T2, T3, T4);

/// Trait for types that can be read from a stream, requiring the size to be configured
///
/// The meaning of the set sized depends on the type being read (e.g, number of bits for integers,
/// number of bytes for strings, number of items for Vec's, etc)
///
/// The `BitReadSized` trait can be used with `#[derive]` on structs
///
/// The implementation can be derived for a struct as long as every field in the struct implements [`BitRead`] or `BitReadSized`
///
/// The struct is read field by field in the order they are defined in, if the size for a field is set [`stream.read_sized()`][read_sized]
/// will be used, otherwise [`stream_read()`][read] will be used.
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
/// # use bitstream_reader_derive::BitReadSized;
/// #
/// #[derive(BitReadSized, PartialEq, Debug)]
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
/// The implementation can be derived for an enum as long as every variant of the enum either has no field, or an unnamed field that implements [`BitRead`] or `BitReadSized`
///
/// The enum is read by first reading a set number of bits as the discriminant of the enum, then the variant for the read discriminant is read.
///
/// For details about setting the input size for fields implementing `BitReadSized` see the block about size in the `Structs` section above.
///
/// The discriminant for the variants defaults to incrementing by one for every field, starting with `0`.
/// You can overwrite the discriminant for a field, which will also change the discriminant for every following field.
///
/// ## Examples
///
/// ```
/// # use bitstream_reader_derive::BitReadSized;
/// #
/// #[derive(BitReadSized)]
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
/// [`BitRead`]: trait.BitRead.html
/// [read_sized]: struct.BitStream.html#method.read_sized
/// [read]: struct.BitStream.html#method.read
pub trait BitReadSized<E: Endianness>: Sized {
    /// Read the type from stream
    fn read(stream: &mut BitStream<E>, size: usize) -> Result<Self>;

    /// The number of bits that will be read or None if the number of bits will change depending
    /// on the bit stream
    #[inline(always)]
    fn bit_size_sized(_size: usize) -> Option<usize> {
        return None;
    }
}

macro_rules! impl_read_int_sized {
    ( $ type: ty) => {
        impl<E: Endianness> BitReadSized<E> for $type {
            #[inline]
            fn read(stream: &mut BitStream<E>, size: usize) -> Result<$type> {
                stream.read_int::<$type>(size)
            }

            #[inline(always)]
            fn bit_size_sized(size: usize) -> Option<usize> {
                Some(size)
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
    #[inline]
    fn read(stream: &mut BitStream<E>, size: usize) -> Result<String> {
        stream.read_string(Some(size))
    }

    #[inline(always)]
    fn bit_size_sized(size: usize) -> Option<usize> {
        Some(8 * size)
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

impl<E: Endianness, T: BitReadSized<E>> BitReadSized<E> for Option<T> {
    fn read(stream: &mut BitStream<E>, size: usize) -> Result<Self> {
        if stream.read()? {
            Ok(Some(stream.read_sized(size)?))
        } else {
            Ok(None)
        }
    }
}

impl<E: Endianness> BitReadSized<E> for BitStream<E> {
    #[inline]
    fn read(stream: &mut BitStream<E>, size: usize) -> Result<Self> {
        stream.read_bits(size)
    }

    #[inline(always)]
    fn bit_size_sized(size: usize) -> Option<usize> {
        Some(size)
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

    #[inline(always)]
    fn bit_size_sized(size: usize) -> Option<usize> {
        T::bit_size().map(|element_size| size * element_size)
    }
}

// Once we have something like https://github.com/rust-lang/rfcs/issues/1053 we can do this optimization
//impl<E: Endianness> ReadSized<E> for Vec<u8> {
//    #[inline]
//    fn read(stream: &mut BitStream<E>, size: usize) -> Result<Self> {
//        stream.read_bytes(size)
//    }
//}

/// Read `K` and `T` `size` times and return as `HashMap<K, T>`
impl<E: Endianness, K: BitRead<E> + Eq + Hash, T: BitRead<E>> BitReadSized<E> for HashMap<K, T> {
    fn read(stream: &mut BitStream<E>, size: usize) -> Result<Self> {
        let mut map = HashMap::with_capacity(size);
        for _ in 0..size {
            let key = stream.read()?;
            let value = stream.read()?;
            map.insert(key, value);
        }
        Ok(map)
    }

    #[inline(always)]
    fn bit_size_sized(size: usize) -> Option<usize> {
        if let (Some(key_size), Some(value_size)) = (K::bit_size(), T::bit_size()) {
            Some(size * (key_size + value_size))
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
/// Struct that lazily reads it's contents from the stream
///
/// Requires [`BitSize`] to be implemented for it's contents so it can grab the correct number of bytes
///
/// [`BitSize`]: trait.BitSize.html
pub struct LazyBitRead<T: BitRead<E>, E: Endianness> {
    source: BitStream<E>,
    inner_type: PhantomData<T>,
}

impl<T: BitRead<E>, E: Endianness> LazyBitRead<T, E> {
    #[inline]
    /// Get the contents of the lazy struct
    pub fn read(mut self) -> Result<T> {
        self.source.read::<T>()
    }
}

impl<T: BitRead<E>, E: Endianness> BitRead<E> for LazyBitRead<T, E> {
    #[inline]
    fn read(stream: &mut BitStream<E>) -> Result<Self> {
        match T::bit_size() {
            Some(bit_size) => Ok(LazyBitRead {
                source: stream.read_bits(bit_size)?,
                inner_type: PhantomData,
            }),
            None => panic!(),
        }
    }

    #[inline(always)]
    fn bit_size() -> Option<usize> {
        T::bit_size()
    }
}

#[derive(Clone, Debug)]
/// Struct that lazily reads it's contents from the stream
///
/// [`BitReadSized`]: trait.BitReadSized.html
pub struct LazyBitReadSized<T: BitReadSized<E>, E: Endianness> {
    source: RefCell<BitStream<E>>,
    size: usize,
    inner_type: PhantomData<T>,
}

impl<T: BitReadSized<E>, E: Endianness> LazyBitReadSized<T, E> {
    #[inline]
    /// Get the contents of the lazy struct
    pub fn value(self) -> Result<T> {
        self.source.borrow_mut().read_sized::<T>(self.size)
    }
}

impl<T: BitReadSized<E>, E: Endianness> BitReadSized<E> for LazyBitReadSized<T, E> {
    #[inline]
    fn read(stream: &mut BitStream<E>, size: usize) -> Result<Self> {
        match T::bit_size_sized(size) {
            Some(bit_size) => Ok(LazyBitReadSized {
                source: RefCell::new(stream.read_bits(bit_size)?),
                inner_type: PhantomData,
                size,
            }),
            None => panic!(),
        }
    }

    #[inline(always)]
    fn bit_size_sized(size: usize) -> Option<usize> {
        T::bit_size_sized(size)
    }
}
