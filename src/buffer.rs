use std::cmp::min;
use std::fmt;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{BitOrAssign, BitXor};
use std::rc::Rc;

use num_traits::{Float, PrimInt};

use crate::endianness::Endianness;
use crate::is_signed::IsSigned;
use crate::unchecked_primitive::{UncheckedPrimitiveFloat, UncheckedPrimitiveInt};
use crate::{ReadError, Result};
use std::convert::TryInto;

const USIZE_SIZE: usize = size_of::<usize>();

/// Buffer that allows reading integers of arbitrary bit length and non byte-aligned integers
///
/// # Examples
///
/// ```
/// use bitstream_reader::{BitBuffer, LittleEndian, Result};
///
/// # fn main() -> Result<()> {
/// let bytes = vec![
///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
/// ];
/// let buffer = BitBuffer::new(bytes, LittleEndian);
/// // read 7 bits as u8, starting from bit 3
/// let result: u8 = buffer.read_int(3, 7)?;
/// #
/// #     Ok(())
/// # }
/// ```
pub struct BitBuffer<E>
where
    E: Endianness,
{
    bytes: Rc<Vec<u8>>,
    bit_len: usize,
    endianness: PhantomData<E>,
}

impl<E> BitBuffer<E>
where
    E: Endianness,
{
    /// Create a new BitBuffer from a byte vector
    ///
    /// # Examples
    ///
    /// ```
    /// use bitstream_reader::{BitBuffer, LittleEndian};
    ///
    /// let bytes = vec![
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
    /// ```
    pub fn new(mut bytes: Vec<u8>, _endianness: E) -> Self {
        let byte_len = bytes.len();

        // pad with usize worth of bytes to ensure we can always read a full usize
        bytes.extend_from_slice(&0usize.to_le_bytes());
        BitBuffer {
            bytes: Rc::new(bytes),
            bit_len: byte_len * 8,
            endianness: PhantomData,
        }
    }
}

impl<E> BitBuffer<E>
where
    E: Endianness,
{
    /// The available number of bits in the buffer
    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    /// The available number of bytes in the buffer
    pub fn byte_len(&self) -> usize {
        self.bytes.len()
    }

    unsafe fn read_usize_bytes(&self, byte_index: usize) -> [u8; USIZE_SIZE] {
        debug_assert!(byte_index + USIZE_SIZE <= self.bytes.len());
        // this is safe because all calling paths check that byte_index is less than the unpadded
        // length (because they check based on bit_len), so with padding byte_index + USIZE_SIZE is
        // always within bounds
        self.bytes
            .get_unchecked(byte_index..byte_index + USIZE_SIZE)
            .try_into()
            .unwrap()
    }

    /// note that only the bottom USIZE - 1 bytes are usable
    unsafe fn read_shifted_usize(&self, byte_index: usize, shift: usize) -> usize {
        let raw_bytes: [u8; USIZE_SIZE] = self.read_usize_bytes(byte_index);
        let raw_usize: usize = usize::from_le_bytes(raw_bytes);
        raw_usize >> shift
    }

    unsafe fn read_usize(&self, position: usize, count: usize) -> usize {
        let byte_index = position / 8;
        let bit_offset = position & 7;
        let usize_bit_size = size_of::<usize>() * 8;

        let bytes: [u8; USIZE_SIZE] = self.read_usize_bytes(byte_index);

        let container = if E::is_le() {
            usize::from_le_bytes(bytes)
        } else {
            usize::from_be_bytes(bytes)
        };

        let shifted = if E::is_le() {
            container >> bit_offset
        } else {
            container >> (usize_bit_size - bit_offset - count)
        };
        let mask = !(std::usize::MAX << count);
        shifted & mask
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
    /// # use bitstream_reader::{BitBuffer, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitBuffer::new(bytes, LittleEndian);
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
            let byte = self.bytes[byte_index];
            let shifted = byte >> bit_offset;
            Ok(shifted & 1u8 == 1)
        } else {
            Err(ReadError::NotEnoughData {
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

        let byte = self.bytes.get_unchecked(byte_index);
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
    /// # use bitstream_reader::{BitBuffer, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitBuffer::new(bytes, LittleEndian);
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
            return Err(ReadError::TooManyBits {
                requested: count,
                max: type_bit_size,
            });
        }

        if position + count > self.bit_len() {
            return if position > self.bit_len() {
                Err(ReadError::IndexOutOfBounds {
                    pos: position,
                    size: self.bit_len(),
                })
            } else {
                Err(ReadError::NotEnoughData {
                    requested: count,
                    bits_left: self.bit_len() - position,
                })
            };
        }

        Ok(unsafe { self.read_int_unchecked(position, count) })
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_int_unchecked<T>(&self, position: usize, count: usize) -> T
    where
        T: PrimInt + BitOrAssign + IsSigned + UncheckedPrimitiveInt + BitXor,
    {
        let type_bit_size = size_of::<T>() * 8;
        let usize_bit_size = size_of::<usize>() * 8;

        let bit_offset = position & 7;

        let fit_usize = count + bit_offset < usize_bit_size;
        let value = if fit_usize {
            self.read_fit_usize(position, count)
        } else {
            self.read_no_fit_usize(position, count)
        };

        if count == type_bit_size {
            value
        } else {
            self.make_signed(value, count)
        }
    }

    #[inline]
    unsafe fn read_fit_usize<T>(&self, position: usize, count: usize) -> T
    where
        T: PrimInt + BitOrAssign + IsSigned + UncheckedPrimitiveInt,
    {
        let raw = self.read_usize(position, count);
        T::from_unchecked(raw)
    }

    unsafe fn read_no_fit_usize<T>(&self, position: usize, count: usize) -> T
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
            let data = T::from_unchecked(self.read_usize(read_pos, read));
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
        if T::is_signed() {
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
    /// # use bitstream_reader::{BitBuffer, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitBuffer::new(bytes, LittleEndian);
    /// assert_eq!(buffer.read_bytes(5, 3)?, &[0b0_1010_101, 0b0_1100_011, 0b1_1001_101]);
    /// assert_eq!(buffer.read_bytes(0, 8)?, &[
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
    pub fn read_bytes(&self, position: usize, byte_count: usize) -> Result<Vec<u8>> {
        if position + byte_count * 8 > self.bit_len() {
            if position > self.bit_len() {
                return Err(ReadError::IndexOutOfBounds {
                    pos: position,
                    size: self.bit_len(),
                });
            } else {
                return Err(ReadError::NotEnoughData {
                    requested: byte_count * 8,
                    bits_left: self.bit_len() - position,
                });
            }
        }

        Ok(unsafe { self.read_bytes_unchecked(position, byte_count) })
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_bytes_unchecked(&self, position: usize, byte_count: usize) -> Vec<u8> {
        let shift = position & 7;

        if shift == 0 {
            let byte_pos = position / 8;
            return self.bytes[byte_pos..byte_pos + byte_count].to_vec();
        }

        let mut data = Vec::with_capacity(byte_count);
        let mut byte_left = byte_count;
        let mut read_pos = position / 8;
        while byte_left > USIZE_SIZE - 1 {
            let bytes = self.read_shifted_usize(read_pos, shift).to_le_bytes();
            let read_bytes = USIZE_SIZE - 1;
            let usable_bytes = &bytes[0..read_bytes];
            data.extend_from_slice(usable_bytes);

            read_pos += read_bytes;
            byte_left -= read_bytes;
        }

        let bytes = self.read_shifted_usize(read_pos, shift).to_le_bytes();
        let usable_bytes = &bytes[0..byte_left];
        data.extend_from_slice(usable_bytes);

        data
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
    /// # use bitstream_reader::{BitBuffer, BitStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0x48, 0x65, 0x6c, 0x6c,
    /// #     0x6f, 0x20, 0x77, 0x6f,
    /// #     0x72, 0x6c, 0x64, 0,
    /// #     0,    0,    0,    0
    /// # ];
    /// # let buffer = BitBuffer::new(bytes, LittleEndian);
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
    pub fn read_string(&self, position: usize, byte_len: Option<usize>) -> Result<String> {
        match byte_len {
            Some(byte_len) => {
                let bytes = self.read_bytes(position, byte_len)?;
                let raw_string = String::from_utf8(bytes)?;
                Ok(raw_string.trim_end_matches(char::from(0)).to_owned())
            }
            None => {
                let bytes = self.read_string_bytes(position)?;
                String::from_utf8(bytes).map_err(ReadError::from)
            }
        }
    }

    #[inline]
    fn find_null_byte(&self, byte_index: usize) -> usize {
        memchr::memchr(0, &self.bytes[byte_index..])
            .map(|index| index + byte_index)
            .unwrap() // due to padding we always have 0 bytes at the end
    }

    #[inline]
    fn read_string_bytes(&self, position: usize) -> Result<Vec<u8>> {
        let shift = position & 7;
        if shift == 0 {
            let byte_index = position / 8;
            Ok(self.bytes[byte_index..self.find_null_byte(byte_index)].to_vec())
        } else {
            let mut acc = Vec::with_capacity(32);
            let mut byte_index = position / 8;
            loop {
                // note: if less then a usize worth of data is left in the buffer, read_usize_bytes
                // will automatically pad with null bytes, triggering the loop termination
                // thus no separate logic for dealing with the end of the bytes is required
                //
                // This is safe because the final usize is filled with 0's, thus triggering the exit clause
                // before reading any out of bounds
                let shifted = unsafe { self.read_shifted_usize(byte_index, shift) };

                let has_null = contains_zero_byte_non_top(shifted);
                let bytes: [u8; USIZE_SIZE] = shifted.to_le_bytes();
                let usable_bytes = &bytes[0..USIZE_SIZE - 1];

                if has_null {
                    for i in 0..USIZE_SIZE - 1 {
                        if usable_bytes[i] == 0 {
                            acc.extend_from_slice(&usable_bytes[0..i]);
                            return Ok(acc);
                        }
                    }
                }

                acc.extend_from_slice(&usable_bytes[0..USIZE_SIZE - 1]);

                byte_index += USIZE_SIZE - 1;
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
    /// # use bitstream_reader::{BitBuffer, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitBuffer::new(bytes, LittleEndian);
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
        if position + type_bit_size > self.bit_len() {
            if position > self.bit_len() {
                return Err(ReadError::IndexOutOfBounds {
                    pos: position,
                    size: self.bit_len(),
                });
            } else {
                return Err(ReadError::NotEnoughData {
                    requested: size_of::<T>() * 8,
                    bits_left: self.bit_len() - position,
                });
            }
        }

        Ok(unsafe { self.read_float_unchecked(position) })
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_float_unchecked<T>(&self, position: usize) -> T
    where
        T: Float + UncheckedPrimitiveFloat,
    {
        if size_of::<T>() == 4 {
            let int = if size_of::<T>() < USIZE_SIZE {
                self.read_fit_usize::<u32>(position, 32)
            } else {
                self.read_no_fit_usize::<u32>(position, 32)
            };
            T::from_f32_unchecked(f32::from_bits(int))
        } else {
            let int = self.read_no_fit_usize::<u64>(position, 64);
            T::from_f64_unchecked(f64::from_bits(int))
        }
    }

    pub(crate) fn get_sub_buffer(&self, bit_len: usize) -> Result<Self> {
        if bit_len > self.bit_len() {
            return Err(ReadError::NotEnoughData {
                requested: bit_len,
                bits_left: self.bit_len(),
            });
        }

        Ok(BitBuffer {
            bytes: Rc::clone(&self.bytes),
            bit_len,
            endianness: PhantomData,
        })
    }
}

impl<E: Endianness> From<Vec<u8>> for BitBuffer<E> {
    fn from(bytes: Vec<u8>) -> Self {
        let byte_len = bytes.len();
        BitBuffer {
            bytes: Rc::new(bytes),
            bit_len: byte_len * 8,
            endianness: PhantomData,
        }
    }
}

impl<E: Endianness> Clone for BitBuffer<E> {
    fn clone(&self) -> Self {
        BitBuffer {
            bytes: Rc::clone(&self.bytes),
            bit_len: self.bit_len(),
            endianness: PhantomData,
        }
    }
}

impl<E: Endianness> Debug for BitBuffer<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BitBuffer {{ bit_len: {}, endianness: {} }}",
            self.bit_len(),
            E::as_string()
        )
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
