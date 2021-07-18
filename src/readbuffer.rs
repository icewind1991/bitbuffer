use std::cmp::min;
use std::fmt;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{BitOrAssign, BitXor, Index, Range, RangeFrom};

use num_traits::{Float, PrimInt};

use crate::endianness::Endianness;
use crate::num_traits::{IsSigned, UncheckedPrimitiveFloat, UncheckedPrimitiveInt};
use crate::{BitError, Result};
use std::borrow::{Borrow, Cow};
use std::convert::TryInto;
use std::rc::Rc;

const USIZE_SIZE: usize = size_of::<usize>();
const USIZE_BIT_SIZE: usize = USIZE_SIZE * 8;

// Cow<[u8]> but with cheap clones using Rc
pub(crate) enum Data<'a> {
    Borrowed(&'a [u8]),
    Owned(Rc<[u8]>),
}

impl<'a> Data<'a> {
    pub fn as_slice(&self) -> &[u8] {
        match self {
            Data::Borrowed(bytes) => *bytes,
            Data::Owned(bytes) => bytes.borrow(),
        }
    }

    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    pub fn to_owned(&self) -> Data<'static> {
        let bytes = match self {
            Data::Borrowed(bytes) => Rc::from(bytes.to_vec()),
            Data::Owned(bytes) => Rc::clone(bytes),
        };
        Data::Owned(bytes)
    }
}

impl<'a> Index<Range<usize>> for Data<'a> {
    type Output = [u8];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl<'a> Index<RangeFrom<usize>> for Data<'a> {
    type Output = [u8];

    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl<'a> Index<usize> for Data<'a> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl<'a> Clone for Data<'a> {
    fn clone(&self) -> Self {
        match self {
            Data::Borrowed(bytes) => Data::Borrowed(*bytes),
            Data::Owned(bytes) => Data::Owned(Rc::clone(bytes)),
        }
    }
}

/// Buffer that allows reading integers of arbitrary bit length and non byte-aligned integers
///
/// # Examples
///
/// ```
/// use bitbuffer::{BitReadBuffer, LittleEndian, Result};
///
/// # fn main() -> Result<()> {
/// let bytes = vec![
///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
/// ];
/// let buffer = BitReadBuffer::new(&bytes, LittleEndian);
/// // read 7 bits as u8, starting from bit 3
/// let result: u8 = buffer.read_int(3, 7)?;
/// #
/// #     Ok(())
/// # }
/// ```
pub struct BitReadBuffer<'a, E>
where
    E: Endianness,
{
    pub(crate) bytes: Data<'a>,
    bit_len: usize,
    endianness: PhantomData<E>,
    slice: &'a [u8],
}

impl<'a, E> BitReadBuffer<'a, E>
where
    E: Endianness,
{
    /// Create a new BitBuffer from a byte slice
    ///
    /// # Examples
    ///
    /// ```
    /// use bitbuffer::{BitReadBuffer, LittleEndian};
    ///
    /// let bytes = vec![
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// ```
    pub fn new(bytes: &'a [u8], _endianness: E) -> Self {
        let byte_len = bytes.len();

        BitReadBuffer {
            bytes: Data::Borrowed(bytes),
            bit_len: byte_len * 8,
            endianness: PhantomData,
            slice: bytes,
        }
    }

    /// Create a static version of this buffer
    ///
    /// If the current buffer is borrowed, this will copy the data
    pub fn to_owned(&self) -> BitReadBuffer<'static, E> {
        let bytes = self.bytes.to_owned();
        let byte_len = bytes.len();

        // this is safe because
        //  - the slice can only be access trough this struct
        //  - this struct keeps the vec the slice comes from alive
        //  - this struct doesn't allow mutation
        let slice = unsafe { std::slice::from_raw_parts(bytes.as_slice().as_ptr(), bytes.len()) };

        BitReadBuffer {
            bytes,
            bit_len: byte_len * 8,
            endianness: PhantomData,
            slice,
        }
    }
}

impl<E> BitReadBuffer<'static, E>
where
    E: Endianness,
{
    /// Create a new BitBuffer from a byte vector
    ///
    /// # Examples
    ///
    /// ```
    /// use bitbuffer::{BitReadBuffer, LittleEndian};
    ///
    /// let bytes = vec![
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer = BitReadBuffer::new_owned(bytes, LittleEndian);
    /// ```
    pub fn new_owned(bytes: Vec<u8>, _endianness: E) -> Self {
        let byte_len = bytes.len();
        let bytes = Data::Owned(Rc::from(bytes));

        // this is safe because
        //  - the slice can only be access trough this struct
        //  - this struct keeps the vec the slice comes from alive
        //  - this struct doesn't allow mutation
        let slice = unsafe { std::slice::from_raw_parts(bytes.as_slice().as_ptr(), bytes.len()) };

        BitReadBuffer {
            bytes,
            bit_len: byte_len * 8,
            endianness: PhantomData,
            slice,
        }
    }
}

pub(crate) fn get_bits_from_usize<E: Endianness>(
    val: usize,
    bit_offset: usize,
    count: usize,
) -> usize {
    let usize_bit_size = size_of::<usize>() * 8;

    let shifted = if E::is_le() {
        val >> bit_offset
    } else {
        val >> (usize_bit_size - bit_offset - count)
    };
    let mask = !(std::usize::MAX << count);
    shifted & mask
}

impl<'a, E> BitReadBuffer<'a, E>
where
    E: Endianness,
{
    /// The available number of bits in the buffer
    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    /// The available number of bytes in the buffer
    pub fn byte_len(&self) -> usize {
        self.slice.len()
    }

    unsafe fn read_usize_bytes(&self, byte_index: usize, end: bool) -> [u8; USIZE_SIZE] {
        if end {
            let mut bytes = [0; USIZE_SIZE];
            let count = min(USIZE_SIZE, self.slice.len() - byte_index);
            bytes[0..count]
                .copy_from_slice(self.slice.get_unchecked(byte_index..byte_index + count));
            bytes
        } else {
            debug_assert!(byte_index + USIZE_SIZE <= self.slice.len());
            // this is safe because all calling paths check that byte_index is less than the unpadded
            // length (because they check based on bit_len), so with padding byte_index + USIZE_SIZE is
            // always within bounds
            self.slice
                .get_unchecked(byte_index..byte_index + USIZE_SIZE)
                .try_into()
                .unwrap()
        }
    }

    /// note that only the bottom USIZE - 1 bytes are usable
    unsafe fn read_shifted_usize(&self, byte_index: usize, shift: usize, end: bool) -> usize {
        let raw_bytes: [u8; USIZE_SIZE] = self.read_usize_bytes(byte_index, end);
        let raw_usize: usize = usize::from_le_bytes(raw_bytes);
        raw_usize >> shift
    }

    unsafe fn read_usize(&self, position: usize, count: usize, end: bool) -> usize {
        let byte_index = position / 8;
        let bit_offset = position & 7;

        let bytes: [u8; USIZE_SIZE] = self.read_usize_bytes(byte_index, end);

        let container = if E::is_le() {
            usize::from_le_bytes(bytes)
        } else {
            usize::from_be_bytes(bytes)
        };

        get_bits_from_usize::<E>(container, bit_offset, count)
    }

    /// Read a single bit from the buffer as boolean
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// let result = buffer.read_bool(5)?;
    /// assert_eq!(result, true);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    #[inline]
    pub fn read_bool(&self, position: usize) -> Result<bool> {
        let byte_index = position / 8;
        let bit_offset = position & 7;

        if position < self.bit_len() {
            let byte = self.slice[byte_index];
            if E::is_le() {
                let shifted = byte >> bit_offset as u8;
                Ok(shifted & 1u8 == 1)
            } else {
                let shifted = byte << bit_offset as u8;
                Ok(shifted & 0b1000_0000u8 == 0b1000_0000u8)
            }
        } else {
            Err(BitError::NotEnoughData {
                requested: 1,
                bits_left: self.bit_len().saturating_sub(position),
            })
        }
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_bool_unchecked(&self, position: usize) -> bool {
        let byte_index = position / 8;
        let bit_offset = position & 7;

        let byte = self.slice.get_unchecked(byte_index);
        let shifted = byte >> bit_offset;
        shifted & 1u8 == 1
    }

    /// Read a sequence of bits from the buffer as integer
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the buffer
    /// - [`ReadError::TooManyBits`]: to many bits requested for the chosen integer type
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// let result = buffer.read_int::<u16>(10, 9)?;
    /// assert_eq!(result, 0b100_0110_10);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    /// [`ReadError::TooManyBits`]: enum.ReadError.html#variant.TooManyBits
    #[inline]
    pub fn read_int<T>(&self, position: usize, count: usize) -> Result<T>
    where
        T: PrimInt + BitOrAssign + IsSigned + UncheckedPrimitiveInt + BitXor,
    {
        let type_bit_size = size_of::<T>() * 8;

        if type_bit_size < count {
            return Err(BitError::TooManyBits {
                requested: count,
                max: type_bit_size,
            });
        }

        if position + USIZE_BIT_SIZE > self.bit_len() {
            if position + count > self.bit_len() {
                return if position > self.bit_len() {
                    Err(BitError::IndexOutOfBounds {
                        pos: position,
                        size: self.bit_len(),
                    })
                } else {
                    Err(BitError::NotEnoughData {
                        requested: count,
                        bits_left: self.bit_len() - position,
                    })
                };
            }
            Ok(unsafe { self.read_int_unchecked(position, count, true) })
        } else {
            Ok(unsafe { self.read_int_unchecked(position, count, false) })
        }
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_int_unchecked<T>(&self, position: usize, count: usize, end: bool) -> T
    where
        T: PrimInt + BitOrAssign + IsSigned + UncheckedPrimitiveInt + BitXor,
    {
        let type_bit_size = size_of::<T>() * 8;
        let usize_bit_size = size_of::<usize>() * 8;

        let bit_offset = position & 7;

        let fit_usize = count + bit_offset < usize_bit_size;
        let value = if fit_usize {
            self.read_fit_usize(position, count, end)
        } else {
            self.read_no_fit_usize(position, count, end)
        };

        if count == type_bit_size {
            value
        } else {
            self.make_signed(value, count)
        }
    }

    #[inline]
    unsafe fn read_fit_usize<T>(&self, position: usize, count: usize, end: bool) -> T
    where
        T: PrimInt + BitOrAssign + IsSigned + UncheckedPrimitiveInt,
    {
        let raw = self.read_usize(position, count, end);
        T::from_unchecked(raw)
    }

    unsafe fn read_no_fit_usize<T>(&self, position: usize, count: usize, end: bool) -> T
    where
        T: PrimInt + BitOrAssign + IsSigned + UncheckedPrimitiveInt,
    {
        let mut left_to_read = count;
        let mut acc = T::zero();
        let max_read = (size_of::<usize>() - 1) * 8;
        let mut read_pos = position;
        let mut bit_offset = 0;
        while left_to_read > 0 {
            let bits_left = self.bit_len() - read_pos;
            let read = min(min(left_to_read, max_read), bits_left);
            let data = T::from_unchecked(self.read_usize(read_pos, read, end));
            if E::is_le() {
                acc |= data << bit_offset;
            } else {
                acc = acc << read;
                acc |= data;
            }
            bit_offset += read;
            read_pos += read;
            left_to_read -= read;
        }

        acc
    }

    fn make_signed<T>(&self, value: T, count: usize) -> T
    where
        T: PrimInt + BitOrAssign + IsSigned + UncheckedPrimitiveInt + BitXor,
    {
        if count == 0 {
            T::zero()
        } else if T::is_signed() {
            let sign_bit = value >> (count - 1) & T::one();
            if sign_bit == T::one() {
                value | (T::zero() - T::one()) ^ ((T::one() << count) - T::one())
            } else {
                value
            }
        } else {
            value
        }
    }

    /// Read a series of bytes from the buffer
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// assert_eq!(buffer.read_bytes(5, 3)?.to_vec(), &[0b0_1010_101, 0b0_1100_011, 0b1_1001_101]);
    /// assert_eq!(buffer.read_bytes(0, 8)?.to_vec(), &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ]);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    #[inline]
    pub fn read_bytes(&self, position: usize, byte_count: usize) -> Result<Cow<'a, [u8]>> {
        if position + byte_count * 8 > self.bit_len() {
            if position > self.bit_len() {
                return Err(BitError::IndexOutOfBounds {
                    pos: position,
                    size: self.bit_len(),
                });
            } else {
                return Err(BitError::NotEnoughData {
                    requested: byte_count * 8,
                    bits_left: self.bit_len() - position,
                });
            }
        }

        Ok(unsafe { self.read_bytes_unchecked(position, byte_count) })
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_bytes_unchecked(&self, position: usize, byte_count: usize) -> Cow<'a, [u8]> {
        let shift = position & 7;

        if shift == 0 {
            let byte_pos = position / 8;
            return Cow::Borrowed(&self.slice[byte_pos..byte_pos + byte_count]);
        }

        let mut data = Vec::with_capacity(byte_count);
        let mut byte_left = byte_count;
        let mut read_pos = position / 8;

        if E::is_le() {
            while byte_left > USIZE_SIZE - 1 {
                let raw = self.read_shifted_usize(read_pos, shift, false);
                let bytes = if E::is_le() {
                    raw.to_le_bytes()
                } else {
                    raw.to_be_bytes()
                };
                let read_bytes = USIZE_SIZE - 1;
                let usable_bytes = &bytes[0..read_bytes];
                data.extend_from_slice(usable_bytes);

                read_pos += read_bytes;
                byte_left -= read_bytes;
            }

            let bytes = self.read_shifted_usize(read_pos, shift, true).to_le_bytes();
            let usable_bytes = &bytes[0..byte_left];
            data.extend_from_slice(usable_bytes);
        } else {
            let mut pos = position;
            while byte_left > 0 {
                data.push(self.read_int_unchecked::<u8>(pos, 8, true));
                byte_left -= 1;
                pos += 8;
            }
        }

        Cow::Owned(data)
    }

    /// Read a series of bytes from the buffer as string
    ///
    /// You can either read a fixed number of bytes, or a dynamic length null-terminated string
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the buffer
    /// - [`ReadError::Utf8Error`]: the read bytes are not valid utf8
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0x48, 0x65, 0x6c, 0x6c,
    /// #     0x6f, 0x20, 0x77, 0x6f,
    /// #     0x72, 0x6c, 0x64, 0,
    /// #     0,    0,    0,    0
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// // Fixed length string
    /// assert_eq!(buffer.read_string(0, Some(13))?, "Hello world".to_owned());
    /// // fixed length with null padding
    /// assert_eq!(buffer.read_string(0, Some(16))?, "Hello world".to_owned());
    /// // null terminated
    /// assert_eq!(buffer.read_string(0, None)?, "Hello world".to_owned());
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    /// [`ReadError::Utf8Error`]: enum.ReadError.html#variant.Utf8Error
    #[inline]
    pub fn read_string(&self, position: usize, byte_len: Option<usize>) -> Result<Cow<'a, str>> {
        match byte_len {
            Some(byte_len) => {
                let bytes = self.read_bytes(position, byte_len)?;

                let string = match bytes {
                    Cow::Owned(bytes) => Cow::Owned(
                        String::from_utf8(bytes)?
                            .trim_end_matches(char::from(0))
                            .to_string(),
                    ),
                    Cow::Borrowed(bytes) => Cow::Borrowed(
                        std::str::from_utf8(bytes)
                            .map_err(|err| BitError::Utf8Error(err, bytes.len()))?
                            .trim_end_matches(char::from(0)),
                    ),
                };
                Ok(string)
            }
            None => {
                let bytes = self.read_string_bytes(position)?;
                let string = match bytes {
                    Cow::Owned(bytes) => Cow::Owned(String::from_utf8(bytes)?),
                    Cow::Borrowed(bytes) => Cow::Borrowed(
                        std::str::from_utf8(bytes)
                            .map_err(|err| BitError::Utf8Error(err, bytes.len()))?,
                    ),
                };
                Ok(string)
            }
        }
    }

    #[inline]
    fn find_null_byte(&self, byte_index: usize) -> usize {
        memchr::memchr(0, &self.slice[byte_index..])
            .map(|index| index + byte_index)
            .unwrap_or(self.slice.len()) // due to padding we always have 0 bytes at the end
    }

    #[inline]
    fn read_string_bytes(&self, position: usize) -> Result<Cow<'a, [u8]>> {
        let shift = position & 7;
        if shift == 0 {
            let byte_index = position / 8;
            Ok(Cow::Borrowed(
                &self.slice[byte_index..self.find_null_byte(byte_index)],
            ))
        } else {
            let mut acc = Vec::with_capacity(32);
            if E::is_le() {
                let mut byte_index = position / 8;
                loop {
                    // note: if less then a usize worth of data is left in the buffer, read_usize_bytes
                    // will automatically pad with null bytes, triggering the loop termination
                    // thus no separate logic for dealing with the end of the bytes is required
                    //
                    // This is safe because the final usize is filled with 0's, thus triggering the exit clause
                    // before reading any out of bounds
                    let shifted = unsafe { self.read_shifted_usize(byte_index, shift, true) };

                    let has_null = contains_zero_byte_non_top(shifted);
                    let bytes: [u8; USIZE_SIZE] = shifted.to_le_bytes();
                    let usable_bytes = &bytes[0..USIZE_SIZE - 1];

                    if has_null {
                        for i in 0..USIZE_SIZE - 1 {
                            if usable_bytes[i] == 0 {
                                acc.extend_from_slice(&usable_bytes[0..i]);
                                return Ok(Cow::Owned(acc));
                            }
                        }
                    }

                    acc.extend_from_slice(&usable_bytes[0..USIZE_SIZE - 1]);

                    byte_index += USIZE_SIZE - 1;
                }
            } else {
                let mut pos = position;
                loop {
                    let byte = self.read_int::<u8>(pos, 8)?;
                    pos += 8;
                    if byte == 0 {
                        return Ok(Cow::Owned(acc));
                    } else {
                        acc.push(byte);
                    }
                }
            }
        }
    }

    /// Read a sequence of bits from the buffer as float
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// let result = buffer.read_float::<f32>(10)?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    #[inline]
    pub fn read_float<T>(&self, position: usize) -> Result<T>
    where
        T: Float + UncheckedPrimitiveFloat,
    {
        let type_bit_size = size_of::<T>() * 8;
        if position + USIZE_BIT_SIZE > self.bit_len() {
            if position + type_bit_size > self.bit_len() {
                if position > self.bit_len() {
                    return Err(BitError::IndexOutOfBounds {
                        pos: position,
                        size: self.bit_len(),
                    });
                } else {
                    return Err(BitError::NotEnoughData {
                        requested: size_of::<T>() * 8,
                        bits_left: self.bit_len() - position,
                    });
                }
            }
            Ok(unsafe { self.read_float_unchecked(position, true) })
        } else {
            Ok(unsafe { self.read_float_unchecked(position, false) })
        }
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_float_unchecked<T>(&self, position: usize, end: bool) -> T
    where
        T: Float + UncheckedPrimitiveFloat,
    {
        if size_of::<T>() == 4 {
            let int = if size_of::<T>() < USIZE_SIZE {
                self.read_fit_usize::<u32>(position, 32, end)
            } else {
                self.read_no_fit_usize::<u32>(position, 32, end)
            };
            T::from_f32_unchecked(f32::from_bits(int))
        } else {
            let int = self.read_no_fit_usize::<u64>(position, 64, end);
            T::from_f64_unchecked(f64::from_bits(int))
        }
    }

    pub(crate) fn get_sub_buffer(&self, bit_len: usize) -> Result<Self> {
        if bit_len > self.bit_len() {
            return Err(BitError::NotEnoughData {
                requested: bit_len,
                bits_left: self.bit_len(),
            });
        }

        Ok(BitReadBuffer {
            bytes: self.bytes.clone(),
            bit_len,
            endianness: PhantomData,
            slice: self.slice,
        })
    }
}

impl<'a, E: Endianness> From<&'a [u8]> for BitReadBuffer<'a, E> {
    fn from(bytes: &'a [u8]) -> Self {
        BitReadBuffer::new(bytes, E::endianness())
    }
}

impl<'a, E: Endianness> From<Vec<u8>> for BitReadBuffer<'a, E> {
    fn from(bytes: Vec<u8>) -> Self {
        BitReadBuffer::new_owned(bytes, E::endianness())
    }
}

impl<'a, E: Endianness> Clone for BitReadBuffer<'a, E> {
    fn clone(&self) -> Self {
        BitReadBuffer {
            bytes: self.bytes.clone(),
            bit_len: self.bit_len(),
            endianness: PhantomData,
            slice: self.slice,
        }
    }
}

impl<E: Endianness> Debug for BitReadBuffer<'_, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BitBuffer {{ bit_len: {}, endianness: {} }}",
            self.bit_len(),
            E::as_string()
        )
    }
}

impl<'a, E: Endianness> PartialEq for BitReadBuffer<'a, E> {
    fn eq(&self, other: &Self) -> bool {
        self.bit_len == other.bit_len && self.slice == other.slice
    }
}

/// Return `true` if `x` contains any zero byte except for the topmost byte.
///
/// From *Matters Computational*, J. Arndt
///
/// "The idea is to subtract one from each of the bytes and then look for
/// bytes where the borrow propagated all the way to the most significant
/// bit."
#[inline(always)]
fn contains_zero_byte_non_top(x: usize) -> bool {
    #[cfg(target_pointer_width = "64")]
    const LO_USIZE: usize = 0x0001_0101_0101_0101;
    #[cfg(target_pointer_width = "64")]
    const HI_USIZE: usize = 0x0080_8080_8080_8080;

    #[cfg(target_pointer_width = "32")]
    const LO_USIZE: usize = 0x000_10101;
    #[cfg(target_pointer_width = "32")]
    const HI_USIZE: usize = 0x0080_8080;

    x.wrapping_sub(LO_USIZE) & !x & HI_USIZE != 0
}
