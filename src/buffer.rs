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
    pub fn new(bytes: Vec<u8>, _endianness: E) -> Self {
        let byte_len = bytes.len();
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

    fn read_usize_bytes(&self, byte_index: usize) -> [u8; USIZE_SIZE] {
        if byte_index + USIZE_SIZE <= self.bytes.len() {
            self.bytes[byte_index..byte_index + USIZE_SIZE]
                .try_into()
                .unwrap()
        } else {
            let mut bytes = [0; USIZE_SIZE];
            let copy_bytes = self.bytes.len() - byte_index;
            bytes[0..copy_bytes].copy_from_slice(&self.bytes[byte_index..byte_index + copy_bytes]);
            bytes
        }
    }

    fn read_usize(&self, position: usize, count: usize) -> usize {
        let byte_index = position / 8;
        let bit_offset = position & 7;
        let usize_bit_size = size_of::<usize>() * 8;

        let bytes = self.read_usize_bytes(byte_index);

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
    pub fn read_bool(&self, position: usize) -> Result<bool> {
        let byte_index = position / 8;
        let bit_offset = position & 7;

        self.bytes
            .get(byte_index)
            .ok_or_else(|| ReadError::NotEnoughData {
                requested: 1,
                bits_left: self.bit_len() - position,
            })
            .map(|byte| {
                let shifted = byte >> bit_offset;
                shifted & 1u8 == 1
            })
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
        let usize_bit_size = size_of::<usize>() * 8;

        if type_bit_size < count {
            return Err(ReadError::TooManyBits {
                requested: count,
                max: type_bit_size,
            });
        }

        if position + count > self.bit_len() {
            if position > self.bit_len() {
                return Err(ReadError::IndexOutOfBounds {
                    pos: position,
                    size: self.bit_len(),
                });
            } else {
                return Err(ReadError::NotEnoughData {
                    requested: count,
                    bits_left: self.bit_len() - position,
                });
            }
        }

        let bit_offset = position & 7;

        let fit_usize = count + bit_offset < usize_bit_size;
        let value = if fit_usize {
            self.read_fit_usize(position, count)
        } else {
            self.read_no_fit_usize(position, count)
        };

        if count == type_bit_size {
            Ok(value)
        } else {
            Ok(self.make_signed(value, count))
        }
    }

    /// Panics:
    ///
    /// requires position + count to not go out of bounds of the buffer: ((position + count) / 8) <= self.bytes.len()
    #[inline]
    fn read_fit_usize<T>(&self, position: usize, count: usize) -> T
    where
        T: PrimInt + BitOrAssign + IsSigned + UncheckedPrimitiveInt,
    {
        let raw = self.read_usize(position, count);
        T::from_unchecked(raw)
    }

    /// Panics:
    ///
    /// requires position + count to not go out of bounds of the buffer: ((position + count) / 8) <= self.bytes.len()
    fn read_no_fit_usize<T>(&self, position: usize, count: usize) -> T
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

        if position & 7 == 0 {
            let byte_pos = position / 8;
            return Ok(self.bytes[byte_pos..byte_pos + byte_count].to_vec());
        }

        let mut data = Vec::with_capacity(byte_count);
        let mut byte_left = byte_count;
        let max_read = size_of::<usize>() - 1;
        let mut read_pos = position;
        while byte_left > 0 {
            let read = min(byte_left, max_read);
            let raw_bytes = self.read_usize(read_pos, read * 8);
            let bytes: [u8; USIZE_SIZE] = if E::is_le() {
                raw_bytes.to_le_bytes()
            } else {
                raw_bytes.to_be_bytes()
            };
            let usable_bytes = if E::is_le() {
                &bytes[0..read]
            } else {
                &bytes[USIZE_SIZE - read..USIZE_SIZE]
            };
            data.extend_from_slice(usable_bytes);
            byte_left -= read;
            read_pos += read * 8;
        }
        Ok(data)
    }

    /// Read a series of bytes from the buffer as string
    ///
    /// You can either read a fixed number of bytes, or a dynamic length null-terminated string
    ///
    /// # Features
    ///
    /// To disable the overhead of checking if the read bytes are valid you can enable the `unchecked_utf8`
    /// feature of the crate to use `String::from_utf8_unchecked` instead of `String::from_utf8`
    /// to create the string from the read bytes.
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

    fn read_string_bytes(&self, position: usize) -> Result<Vec<u8>> {
        let mut acc = Vec::with_capacity(32);
        let mut pos = position;
        loop {
            let read = min((USIZE_SIZE - 1) * 8, self.bit_len() - pos);
            let raw_bytes = self.read_usize(pos, read);
            let bytes: [u8; USIZE_SIZE] = if E::is_le() {
                raw_bytes.to_le_bytes()
            } else {
                raw_bytes.to_be_bytes()
            };

            let bytes_read = read / 8;

            let (start, end) = if E::is_le() {
                (0usize, bytes_read)
            } else {
                (USIZE_SIZE - bytes_read, USIZE_SIZE)
            };

            for i in start..end {
                if bytes[i] == 0 {
                    acc.extend_from_slice(&bytes[start..i]);
                    return Ok(acc);
                }
            }
            acc.extend_from_slice(&bytes[start..end]);

            if bytes_read < (USIZE_SIZE - 1) {
                return Ok(acc);
            }

            pos += read;
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

        if size_of::<T>() == 4 {
            let int = if size_of::<T>() < USIZE_SIZE {
                self.read_fit_usize::<u32>(position, 32)
            } else {
                self.read_no_fit_usize::<u32>(position, 32)
            };
            Ok(T::from_f32_unchecked(f32::from_bits(int)))
        } else {
            let int = self.read_no_fit_usize::<u64>(position, 64);
            Ok(T::from_f64_unchecked(f64::from_bits(int)))
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
