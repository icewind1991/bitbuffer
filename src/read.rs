use crate::endianness::{BigEndian, LittleEndian};
use crate::{BitReadStream, Endianness, Result};
use std::borrow::Cow;
use std::cell::RefCell;
use std::cmp::min;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::{size_of, MaybeUninit};
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
/// # use bitbuffer::BitRead;
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
/// # use bitbuffer::BitRead;
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
/// # use bitbuffer::BitRead;
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
/// [read_sized]: BitReadStream::read_sized
/// [read]: BitReadStream::read
pub trait BitRead<'a, E: Endianness>: Sized {
    /// Read the type from stream
    fn read(stream: &mut BitReadStream<'a, E>) -> Result<Self>;

    /// Note: only the bounds are unchecked
    ///
    /// any other validations (e.g. checking for valid utf8) still needs to be done
    #[doc(hidden)]
    #[inline]
    unsafe fn read_unchecked(stream: &mut BitReadStream<'a, E>, _end: bool) -> Result<Self> {
        Self::read(stream)
    }

    /// Skip the type
    ///
    /// This might be faster than reading it if the size is known beforehand
    #[inline]
    fn skip(stream: &mut BitReadStream<'a, E>) -> Result<()> {
        match Self::bit_size() {
            Some(size) => stream.skip_bits(size),
            None => Self::read(stream).map(|_| ()),
        }
    }

    /// The number of bits that will be read or None if the number of bits will change depending
    /// on the bit stream
    #[inline]
    fn bit_size() -> Option<usize> {
        None
    }
}

macro_rules! impl_read_int {
    ($type:ty) => {
        impl<E: Endianness> BitRead<'_, E> for $type {
            #[inline]
            fn read(stream: &mut BitReadStream<E>) -> Result<$type> {
                stream.read_int::<$type>(<$type>::BITS as usize)
            }

            #[inline]
            unsafe fn read_unchecked(stream: &mut BitReadStream<E>, end: bool) -> Result<$type> {
                Ok(stream.read_int_unchecked::<$type>(<$type>::BITS as usize, end))
            }

            #[inline]
            fn bit_size() -> Option<usize> {
                Some(<$type>::BITS as usize)
            }
        }
    };
}

macro_rules! impl_read_int_nonzero {
    ($type:ty) => {
        impl BitRead<'_, LittleEndian> for Option<$type> {
            #[inline]
            fn read(stream: &mut BitReadStream<LittleEndian>) -> Result<Self> {
                Ok(<$type>::new(stream.read()?))
            }

            #[inline]
            unsafe fn read_unchecked(
                stream: &mut BitReadStream<LittleEndian>,
                end: bool,
            ) -> Result<Self> {
                Ok(<$type>::new(
                    stream.read_int_unchecked(size_of::<$type>() * 8, end),
                ))
            }

            #[inline]
            fn bit_size() -> Option<usize> {
                Some(size_of::<$type>() * 8)
            }
        }

        impl BitRead<'_, BigEndian> for Option<$type> {
            #[inline]
            fn read(stream: &mut BitReadStream<BigEndian>) -> Result<Self> {
                Ok(<$type>::new(stream.read()?))
            }

            #[inline]
            unsafe fn read_unchecked(
                stream: &mut BitReadStream<BigEndian>,
                end: bool,
            ) -> Result<Self> {
                Ok(<$type>::new(
                    stream.read_int_unchecked(size_of::<$type>() * 8, end),
                ))
            }

            #[inline]
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

impl<E: Endianness> BitRead<'_, E> for f32 {
    #[inline]
    fn read(stream: &mut BitReadStream<E>) -> Result<f32> {
        stream.read_float::<f32>()
    }

    #[inline]
    unsafe fn read_unchecked(stream: &mut BitReadStream<E>, end: bool) -> Result<f32> {
        Ok(stream.read_float_unchecked::<f32>(end))
    }

    #[inline]
    fn bit_size() -> Option<usize> {
        Some(32)
    }
}

impl<E: Endianness> BitRead<'_, E> for f64 {
    #[inline]
    fn read(stream: &mut BitReadStream<E>) -> Result<f64> {
        stream.read_float::<f64>()
    }

    #[inline]
    unsafe fn read_unchecked(stream: &mut BitReadStream<E>, end: bool) -> Result<f64> {
        Ok(stream.read_float_unchecked::<f64>(end))
    }

    #[inline]
    fn bit_size() -> Option<usize> {
        Some(64)
    }
}

impl<E: Endianness> BitRead<'_, E> for bool {
    #[inline]
    fn read(stream: &mut BitReadStream<E>) -> Result<bool> {
        stream.read_bool()
    }

    #[inline]
    unsafe fn read_unchecked(stream: &mut BitReadStream<E>, _end: bool) -> Result<bool> {
        Ok(stream.read_bool_unchecked())
    }

    #[inline]
    fn bit_size() -> Option<usize> {
        Some(1)
    }
}

impl<E: Endianness> BitRead<'_, E> for String {
    #[inline]
    fn read(stream: &mut BitReadStream<E>) -> Result<String> {
        Ok(stream.read_string(None)?.into_owned())
    }
}

impl<'a, E: Endianness> BitRead<'a, E> for Cow<'a, str> {
    #[inline]
    fn read(stream: &mut BitReadStream<'a, E>) -> Result<Cow<'a, str>> {
        stream.read_string(None)
    }
}

impl<'a, E: Endianness, T: BitRead<'a, E>> BitRead<'a, E> for Rc<T> {
    #[inline]
    fn read(stream: &mut BitReadStream<'a, E>) -> Result<Self> {
        Ok(Rc::new(T::read(stream)?))
    }

    #[inline]
    unsafe fn read_unchecked(stream: &mut BitReadStream<'a, E>, end: bool) -> Result<Self> {
        Ok(Rc::new(T::read_unchecked(stream, end)?))
    }

    #[inline]
    fn bit_size() -> Option<usize> {
        T::bit_size()
    }
}

impl<'a, E: Endianness, T: BitRead<'a, E>> BitRead<'a, E> for Arc<T> {
    #[inline]
    fn read(stream: &mut BitReadStream<'a, E>) -> Result<Self> {
        Ok(Arc::new(T::read(stream)?))
    }

    #[inline]
    unsafe fn read_unchecked(stream: &mut BitReadStream<'a, E>, end: bool) -> Result<Self> {
        Ok(Arc::new(T::read_unchecked(stream, end)?))
    }

    #[inline]
    fn bit_size() -> Option<usize> {
        T::bit_size()
    }
}

impl<'a, E: Endianness, T: BitRead<'a, E>> BitRead<'a, E> for Box<T> {
    #[inline]
    fn read(stream: &mut BitReadStream<'a, E>) -> Result<Self> {
        Ok(Box::new(T::read(stream)?))
    }

    #[inline]
    unsafe fn read_unchecked(stream: &mut BitReadStream<'a, E>, end: bool) -> Result<Self> {
        Ok(Box::new(T::read_unchecked(stream, end)?))
    }

    #[inline]
    fn bit_size() -> Option<usize> {
        T::bit_size()
    }
}

macro_rules! impl_read_tuple {
    ($($type:ident),*) => {
        impl<'a, E: Endianness, $($type: BitRead<'a, E>),*> BitRead<'a, E> for ($($type),*) {
            #[inline]
            fn read(stream: &mut BitReadStream<'a, E>) -> Result<Self> {
                Ok(($(<$type>::read(stream)?),*))
            }

            #[inline]
            unsafe fn read_unchecked(stream: &mut BitReadStream<'a, E>, end: bool) -> Result<Self> {
                Ok(($(<$type>::read_unchecked(stream, end)?),*))
            }

            #[inline]
            fn bit_size() -> Option<usize> {
                Some(0)$(.and_then(|sum| <$type>::bit_size().map(|size| sum + size)))*
            }
        }
    };
}

impl_read_tuple!(T1, T2);
impl_read_tuple!(T1, T2, T3);
impl_read_tuple!(T1, T2, T3, T4);

impl<'a, E: Endianness, T: BitRead<'a, E>, const N: usize> BitRead<'a, E> for [T; N] {
    #[inline]
    fn read(stream: &mut BitReadStream<'a, E>) -> Result<Self> {
        match T::bit_size() {
            Some(bit_size) => {
                let end = stream.check_read(bit_size * N)?;
                unsafe { Self::read_unchecked(stream, end) }
            }
            None => {
                // SAFETY: An uninitialized `[MaybeUninit<_>; LEN]` is valid.
                let mut array =
                    unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() };
                for item in array.iter_mut() {
                    unsafe {
                        // length is already checked
                        let val = stream.read()?;
                        item.as_mut_ptr().write(val)
                    }
                }
                unsafe { Ok((&array as *const _ as *const [T; N]).read()) }
            }
        }
    }

    #[inline]
    unsafe fn read_unchecked(stream: &mut BitReadStream<'a, E>, end: bool) -> Result<Self> {
        // SAFETY: An uninitialized `[MaybeUninit<_>; LEN]` is valid.
        let mut array = MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init();

        for item in array.iter_mut() {
            // length is already checked
            let val = stream.read_unchecked(end)?;
            item.as_mut_ptr().write(val);
        }

        Ok((&array as *const _ as *const [T; N]).read())
    }

    #[inline]
    fn bit_size() -> Option<usize> {
        T::bit_size().map(|size| size * N)
    }
}

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
/// # use bitbuffer::BitReadSized;
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
/// # use bitbuffer::BitReadSized;
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
/// [read_sized]: BitReadStream::read_sized
/// [read]: BitReadStream::read
pub trait BitReadSized<'a, E: Endianness>: Sized {
    /// Read the type from stream
    fn read(stream: &mut BitReadStream<'a, E>, size: usize) -> Result<Self>;

    #[doc(hidden)]
    #[inline]
    unsafe fn read_unchecked(
        stream: &mut BitReadStream<'a, E>,
        size: usize,
        _end: bool,
    ) -> Result<Self> {
        Self::read(stream, size)
    }

    /// Skip the type
    ///
    /// This might be faster than reading it if the size is known beforehand
    #[inline]
    fn skip(stream: &mut BitReadStream<'a, E>, size: usize) -> Result<()> {
        match Self::bit_size_sized(size) {
            Some(size) => stream.skip_bits(size),
            None => Self::read(stream, size).map(|_| ()),
        }
    }

    /// The number of bits that will be read or None if the number of bits will change depending
    /// on the bit stream
    #[inline]
    fn bit_size_sized(_size: usize) -> Option<usize> {
        None
    }
}

macro_rules! impl_read_int_sized {
    ( $ type: ty) => {
        impl<E: Endianness> BitReadSized<'_, E> for $type {
            #[inline]
            fn read(stream: &mut BitReadStream<E>, size: usize) -> Result<$type> {
                stream.read_int::<$type>(size)
            }

            #[inline]
            unsafe fn read_unchecked(
                stream: &mut BitReadStream<E>,
                size: usize,
                end: bool,
            ) -> Result<$type> {
                Ok(stream.read_int_unchecked::<$type>(size, end))
            }

            #[inline]
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

impl<E: Endianness> BitReadSized<'_, E> for String {
    #[inline]
    fn read(stream: &mut BitReadStream<E>, size: usize) -> Result<String> {
        Ok(stream.read_string(Some(size))?.into_owned())
    }

    #[inline]
    fn bit_size_sized(size: usize) -> Option<usize> {
        Some(8 * size)
    }
}

impl<'a, E: Endianness> BitReadSized<'a, E> for Cow<'a, str> {
    #[inline]
    fn read(stream: &mut BitReadStream<'a, E>, size: usize) -> Result<Cow<'a, str>> {
        stream.read_string(Some(size))
    }

    #[inline]
    fn bit_size_sized(size: usize) -> Option<usize> {
        Some(8 * size)
    }
}

impl<'a, E: Endianness> BitReadSized<'a, E> for Cow<'a, [u8]> {
    #[inline]
    fn read(stream: &mut BitReadStream<'a, E>, size: usize) -> Result<Cow<'a, [u8]>> {
        stream.read_bytes(size)
    }

    #[inline]
    fn bit_size_sized(size: usize) -> Option<usize> {
        Some(8 * size)
    }
}

/// Read a boolean, if true, read `T`, else return `None`
impl<'a, E: Endianness, T: BitRead<'a, E>> BitRead<'a, E> for Option<T> {
    fn read(stream: &mut BitReadStream<'a, E>) -> Result<Self> {
        if stream.read()? {
            Ok(Some(stream.read()?))
        } else {
            Ok(None)
        }
    }
}

impl<'a, E: Endianness, T: BitReadSized<'a, E>> BitReadSized<'a, E> for Option<T> {
    fn read(stream: &mut BitReadStream<'a, E>, size: usize) -> Result<Self> {
        if stream.read()? {
            Ok(Some(stream.read_sized(size)?))
        } else {
            Ok(None)
        }
    }
}

impl<'a, E: Endianness> BitReadSized<'a, E> for BitReadStream<'a, E> {
    #[inline]
    fn read(stream: &mut BitReadStream<'a, E>, size: usize) -> Result<Self> {
        stream.read_bits(size)
    }

    #[inline]
    fn bit_size_sized(size: usize) -> Option<usize> {
        Some(size)
    }
}

/// Read `T` `size` times and return as `Vec<T>`
impl<'a, E: Endianness, T: BitRead<'a, E>> BitReadSized<'a, E> for Vec<T> {
    fn read(stream: &mut BitReadStream<'a, E>, size: usize) -> Result<Self> {
        let mut vec = Vec::with_capacity(min(size, 128));
        match T::bit_size() {
            Some(bit_size) => {
                if stream.check_read(bit_size * size)? {
                    for _ in 0..size {
                        vec.push(unsafe { stream.read_unchecked(true) }?)
                    }
                } else {
                    for _ in 0..size {
                        vec.push(unsafe { stream.read_unchecked(false) }?)
                    }
                }
            }
            _ => {
                for _ in 0..size {
                    vec.push(stream.read()?)
                }
            }
        }
        Ok(vec)
    }

    #[inline]
    unsafe fn read_unchecked(
        stream: &mut BitReadStream<'a, E>,
        size: usize,
        end: bool,
    ) -> Result<Self> {
        let mut vec = Vec::with_capacity(min(size, 128));
        for _ in 0..size {
            vec.push(stream.read_unchecked(end)?)
        }
        Ok(vec)
    }

    #[inline]
    fn bit_size_sized(size: usize) -> Option<usize> {
        T::bit_size().map(|element_size| size * element_size)
    }
}

// Once we have something like https://github.com/rust-lang/rfcs/issues/1053 we can do this optimization
//impl<E: Endianness> ReadSized<E> for Vec<u8> {
//    #[inline]
//    fn read(stream: &mut BitReadStream<E>, size: usize) -> Result<Self> {
//        stream.read_bytes(size)
//    }
//}

/// Read `K` and `T` `size` times and return as `HashMap<K, T>`
#[allow(clippy::implicit_hasher)]
impl<'a, E: Endianness, K: BitRead<'a, E> + Eq + Hash, T: BitRead<'a, E>> BitReadSized<'a, E>
    for HashMap<K, T>
{
    fn read(stream: &mut BitReadStream<'a, E>, size: usize) -> Result<Self> {
        let mut map = HashMap::with_capacity(min(size, 128));
        for _ in 0..size {
            let key = stream.read()?;
            let value = stream.read()?;
            map.insert(key, value);
        }
        Ok(map)
    }

    #[inline]
    unsafe fn read_unchecked(
        stream: &mut BitReadStream<'a, E>,
        size: usize,
        end: bool,
    ) -> Result<Self> {
        let mut map = HashMap::with_capacity(min(size, 128));
        for _ in 0..size {
            let key = stream.read_unchecked(end)?;
            let value = stream.read_unchecked(end)?;
            map.insert(key, value);
        }
        Ok(map)
    }

    #[inline]
    fn bit_size_sized(size: usize) -> Option<usize> {
        if let (Some(key_size), Some(value_size)) = (K::bit_size(), T::bit_size()) {
            Some(size * (key_size + value_size))
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Struct that lazily reads it's contents from the stream
pub struct LazyBitRead<'a, T: BitRead<'a, E>, E: Endianness> {
    source: BitReadStream<'a, E>,
    inner_type: PhantomData<T>,
}

impl<'a, T: BitRead<'a, E>, E: Endianness> LazyBitRead<'a, T, E> {
    #[inline]
    /// Get the contents of the lazy struct
    pub fn read(&self) -> Result<T> {
        self.source.clone().read::<T>()
    }
}

impl<'a, T: BitRead<'a, E>, E: Endianness> BitRead<'a, E> for LazyBitRead<'a, T, E> {
    #[inline]
    fn read(stream: &mut BitReadStream<'a, E>) -> Result<Self> {
        match T::bit_size() {
            Some(bit_size) => Ok(LazyBitRead {
                source: stream.read_bits(bit_size)?,
                inner_type: PhantomData,
            }),
            None => panic!(),
        }
    }

    #[inline]
    fn bit_size() -> Option<usize> {
        T::bit_size()
    }
}

#[derive(Clone, Debug)]
/// Struct that lazily reads it's contents from the stream
pub struct LazyBitReadSized<'a, T: BitReadSized<'a, E>, E: Endianness> {
    source: RefCell<BitReadStream<'a, E>>,
    size: usize,
    inner_type: PhantomData<T>,
}

impl<'a, T: BitReadSized<'a, E>, E: Endianness> LazyBitReadSized<'a, T, E> {
    #[inline]
    /// Get the contents of the lazy struct
    pub fn value(self) -> Result<T> {
        self.source.borrow_mut().read_sized::<T>(self.size)
    }
}

impl<'a, T: BitReadSized<'a, E>, E: Endianness> BitReadSized<'a, E> for LazyBitReadSized<'a, T, E> {
    #[inline]
    fn read(stream: &mut BitReadStream<'a, E>, size: usize) -> Result<Self> {
        match T::bit_size_sized(size) {
            Some(bit_size) => Ok(LazyBitReadSized {
                source: RefCell::new(stream.read_bits(bit_size)?),
                inner_type: PhantomData,
                size,
            }),
            None => panic!(),
        }
    }

    #[inline]
    fn bit_size_sized(size: usize) -> Option<usize> {
        T::bit_size_sized(size)
    }
}

impl<'a, E: Endianness, T: BitReadSized<'a, E>> BitReadSized<'a, E> for Arc<T> {
    #[inline]
    fn read(stream: &mut BitReadStream<'a, E>, size: usize) -> Result<Self> {
        Ok(Arc::new(T::read(stream, size)?))
    }

    #[inline]
    unsafe fn read_unchecked(
        stream: &mut BitReadStream<'a, E>,
        size: usize,
        end: bool,
    ) -> Result<Self> {
        Ok(Arc::new(T::read_unchecked(stream, size, end)?))
    }

    #[inline]
    fn bit_size_sized(size: usize) -> Option<usize> {
        T::bit_size_sized(size)
    }
}

impl<'a, E: Endianness, T: BitReadSized<'a, E>> BitReadSized<'a, E> for Rc<T> {
    #[inline]
    fn read(stream: &mut BitReadStream<'a, E>, size: usize) -> Result<Self> {
        Ok(Rc::new(T::read(stream, size)?))
    }

    #[inline]
    unsafe fn read_unchecked(
        stream: &mut BitReadStream<'a, E>,
        size: usize,
        end: bool,
    ) -> Result<Self> {
        Ok(Rc::new(T::read_unchecked(stream, size, end)?))
    }

    #[inline]
    fn bit_size_sized(size: usize) -> Option<usize> {
        T::bit_size_sized(size)
    }
}

impl<'a, E: Endianness, T: BitReadSized<'a, E>> BitReadSized<'a, E> for Box<T> {
    #[inline]
    fn read(stream: &mut BitReadStream<'a, E>, size: usize) -> Result<Self> {
        Ok(Box::new(T::read(stream, size)?))
    }

    #[inline]
    unsafe fn read_unchecked(
        stream: &mut BitReadStream<'a, E>,
        size: usize,
        end: bool,
    ) -> Result<Self> {
        Ok(Box::new(T::read_unchecked(stream, size, end)?))
    }

    #[inline]
    fn bit_size_sized(size: usize) -> Option<usize> {
        T::bit_size_sized(size)
    }
}

impl<'a, E: Endianness, T: BitReadSized<'a, E>, const N: usize> BitReadSized<'a, E> for [T; N] {
    #[inline]
    fn read(stream: &mut BitReadStream<'a, E>, size: usize) -> Result<Self> {
        match T::bit_size_sized(size) {
            Some(bit_size) => {
                let end = stream.check_read(bit_size * N)?;
                unsafe { Self::read_unchecked(stream, size, end) }
            }
            None => {
                // SAFETY: An uninitialized `[MaybeUninit<_>; LEN]` is valid.
                let mut array =
                    unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() };
                for item in array.iter_mut() {
                    unsafe {
                        // length is already checked
                        let val = stream.read_sized(size)?;
                        item.as_mut_ptr().write(val)
                    }
                }
                unsafe { Ok((&array as *const _ as *const [T; N]).read()) }
            }
        }
    }

    #[inline]
    unsafe fn read_unchecked(
        stream: &mut BitReadStream<'a, E>,
        size: usize,
        end: bool,
    ) -> Result<Self> {
        // SAFETY: An uninitialized `[MaybeUninit<_>; LEN]` is valid.
        let mut array = MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init();

        for item in array.iter_mut() {
            // length is already checked
            let val = stream.read_sized_unchecked(size, end)?;
            item.as_mut_ptr().write(val);
        }

        Ok((&array as *const _ as *const [T; N]).read())
    }

    #[inline]
    fn bit_size_sized(size: usize) -> Option<usize> {
        T::bit_size_sized(size).map(|size| size * N)
    }
}

#[test]
fn test_array_sizes() {
    assert_eq!(None, <[String; 16] as BitRead<LittleEndian>>::bit_size());
    assert_eq!(
        Some(3 * 8 * 16),
        <[String; 16] as BitReadSized<LittleEndian>>::bit_size_sized(3)
    );

    assert_eq!(
        Some(16 * 8),
        <[u8; 16] as BitRead<LittleEndian>>::bit_size()
    );

    assert_eq!(
        Some(8 * 16),
        <Cow<[u8]> as BitReadSized<LittleEndian>>::bit_size_sized(16)
    );
    assert_eq!(
        Some(8 * 16),
        <Cow<str> as BitReadSized<LittleEndian>>::bit_size_sized(16)
    );
    assert_eq!(
        Some(16),
        <BitReadStream<LittleEndian> as BitReadSized<LittleEndian>>::bit_size_sized(16)
    );

    assert_eq!(
        Some(8 * 16),
        <Vec<u8> as BitReadSized<LittleEndian>>::bit_size_sized(16)
    );
    assert_eq!(
        Some(8 * 16 + 16 * 16),
        <HashMap<u8, u16> as BitReadSized<LittleEndian>>::bit_size_sized(16)
    );
}

#[test]
fn test_wrapper_sizes() {
    fn test_bit_size_le<'a, T: BitRead<'a, LittleEndian>, U: BitRead<'a, LittleEndian>>() {
        assert_eq!(T::bit_size(), U::bit_size());
    }

    fn test_bit_size_sized_le<
        'a,
        T: BitReadSized<'a, LittleEndian>,
        U: BitReadSized<'a, LittleEndian>,
    >() {
        assert_eq!(T::bit_size_sized(3), U::bit_size_sized(3));
    }
    test_bit_size_le::<String, Arc<String>>();

    test_bit_size_sized_le::<String, Arc<String>>();
    test_bit_size_sized_le::<String, Rc<String>>();
    test_bit_size_sized_le::<String, Box<String>>();
    test_bit_size_sized_le::<String, LazyBitReadSized<String, LittleEndian>>();

    test_bit_size_le::<u8, Arc<u8>>();
    test_bit_size_le::<u8, Rc<u8>>();
    test_bit_size_le::<u8, Box<u8>>();
    test_bit_size_le::<u8, LazyBitRead<u8, LittleEndian>>();
}

#[test]
fn test_unsized_sizes() {
    fn test_bit_size_none<'a, T: BitRead<'a, LittleEndian>>() {
        assert_eq!(None, T::bit_size());
    }
    fn test_bit_size_sized_none<'a, T: BitReadSized<'a, LittleEndian>>() {
        assert_eq!(None, T::bit_size_sized(3));
    }
    fn test_bit_size_sized_some<'a, T: BitReadSized<'a, LittleEndian>>() {
        assert!(T::bit_size_sized(3).is_some());
    }
    test_bit_size_none::<String>();
    test_bit_size_none::<Cow<str>>();
    test_bit_size_sized_none::<Option<String>>();

    test_bit_size_none::<Option<u8>>();

    test_bit_size_sized_some::<Cow<[u8]>>();
    test_bit_size_sized_some::<String>();
    test_bit_size_sized_some::<String>();
}

#[test]
fn test_primitive_sizes() {
    fn test_bit_size<'a, T: BitRead<'a, LittleEndian>>() {
        assert_eq!(Some(size_of::<T>() * 8), T::bit_size());
    }
    test_bit_size::<u8>();
    test_bit_size::<u16>();
    test_bit_size::<u32>();
    test_bit_size::<u64>();
    test_bit_size::<u128>();
    test_bit_size::<i8>();
    test_bit_size::<i16>();
    test_bit_size::<i32>();
    test_bit_size::<i64>();
    test_bit_size::<i128>();
    test_bit_size::<f32>();
    test_bit_size::<f64>();

    assert_eq!(Some(1), <bool as BitRead<LittleEndian>>::bit_size());
}
