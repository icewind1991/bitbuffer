use std::cmp::min;
use std::fmt;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::BitOrAssign;

use num_traits::{Float, PrimInt};

use crate::endianness::Endianness;
use crate::is_signed::IsSigned;
use crate::unchecked_primitive::{UncheckedPrimitiveFloat, UncheckedPrimitiveInt};
use crate::{ReadError, Result};

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
    bytes: Vec<u8>,
    bit_len: usize,
    byte_len: usize,
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
            bytes,
            byte_len,
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
        self.byte_len
    }

    fn read_usize(&self, position: usize, count: usize) -> usize {
        let byte_index = position / 8;
        let bit_offset = position - byte_index * 8;
        let raw_container: &usize = unsafe {
            // this is safe here because we already have checks that we don't read past the slice
            let ptr = self.bytes.as_ptr().add(byte_index);
            std::mem::transmute(ptr)
        };
        let container = if E::is_le() {
            usize::from_le(*raw_container)
        } else {
            usize::from_be(*raw_container)
        };
        let shifted = if E::is_le() {
            container >> bit_offset
        } else {
            container >> (USIZE_SIZE * 8 - bit_offset - count)
        };
        let mask = !(usize::max_value() << count);
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

        if position >= self.bit_len {
            return Err(ReadError::NotEnoughData {
                requested: 1,
                bits_left: self.bit_len - position,
            });
        }

        let byte = self.bytes[byte_index];
        let shifted = byte >> bit_offset;
        Ok(shifted & 1u8 == 1)
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
    pub fn read_int<T>(&self, position: usize, count: usize) -> Result<T>
    where
        T: PrimInt + BitOrAssign + IsSigned + UncheckedPrimitiveInt,
    {
        let value = {
            let type_bit_size = size_of::<T>() * 8;
            let usize_bit_size = size_of::<usize>() * 8;

            if type_bit_size < count {
                return Err(ReadError::TooManyBits {
                    requested: count,
                    max: type_bit_size,
                });
            }

            if position + count > self.bit_len {
                if position > self.bit_len {
                    return Err(ReadError::IndexOutOfBounds {
                        pos: position,
                        size: self.bit_len,
                    });
                } else {
                    return Err(ReadError::NotEnoughData {
                        requested: count,
                        bits_left: self.bit_len - position,
                    });
                }
            }

            let bit_offset = position & 7;
            if size_of::<usize>() > size_of::<T>() || count + bit_offset < usize_bit_size {
                let raw = self.read_usize(position, count);
                let max_signed_value = (1 << (type_bit_size - 1)) - 1;
                if T::is_signed() && raw > max_signed_value {
                    return Ok(T::zero() - T::from_unchecked(raw & max_signed_value));
                } else {
                    T::from_unchecked(raw)
                }
            } else {
                let mut left_to_read = count;
                let mut partial = T::zero();
                let max_read = (size_of::<usize>() - 1) * 8;
                let mut read_pos = position;
                let mut bit_offset = 0;
                while left_to_read > 0 {
                    let bits_left = self.bit_len - read_pos;
                    let read = min(min(left_to_read, max_read), bits_left);
                    let data = T::from_unchecked(self.read_usize(read_pos, read));
                    if E::is_le() {
                        partial |= data << bit_offset;
                    } else {
                        partial = partial << read;
                        partial |= data;
                    }
                    bit_offset += read;
                    read_pos += read;
                    left_to_read -= read;
                }

                partial
            }
        };

        if T::is_signed() {
            let sign_bit = value >> (count - 1) & T::one();
            let absolute_value = value & !(T::max_value() << (count - 1));
            let sign = T::one() - sign_bit - sign_bit;
            Ok(absolute_value * sign)
        } else {
            Ok(value)
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
        if position + byte_count * 8 > self.bit_len {
            if position > self.bit_len {
                return Err(ReadError::IndexOutOfBounds {
                    pos: position,
                    size: self.bit_len,
                });
            } else {
                return Err(ReadError::NotEnoughData {
                    requested: byte_count * 8,
                    bits_left: self.bit_len - position,
                });
            }
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
        let bytes = self.read_string_bytes(position, byte_len)?;
        let raw_string = String::from_utf8(bytes)?;
        Ok(raw_string.trim_end_matches(char::from(0)).to_owned())
    }

    fn read_string_bytes(&self, position: usize, byte_len: Option<usize>) -> Result<Vec<u8>> {
        match byte_len {
            Some(len) => return self.read_bytes(position, len),
            None => {
                let mut acc = Vec::with_capacity(25);
                let mut pos = position;
                loop {
                    let read = min((USIZE_SIZE - 1) * 8, self.bit_len - pos);
                    let raw_bytes = self.read_usize(pos, read);
                    let bytes: [u8; USIZE_SIZE] = if E::is_le() {
                        raw_bytes.to_le_bytes()
                    } else {
                        raw_bytes.to_be_bytes()
                    };
                    for i in 0..(USIZE_SIZE - 1) {
                        // ony LE we use the first 7 bytes, on BE the last 7
                        let byte = if E::is_le() { bytes[i] } else { bytes[1 + i] };

                        if byte == 0 {
                            return Ok(acc);
                        }
                        acc.push(byte);
                    }
                    pos += read;
                }
            }
        };
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
        if position + size_of::<T>() * 8 > self.bit_len {
            if position > self.bit_len {
                return Err(ReadError::IndexOutOfBounds {
                    pos: position,
                    size: self.bit_len,
                });
            } else {
                return Err(ReadError::NotEnoughData {
                    requested: size_of::<T>() * 8,
                    bits_left: self.bit_len - position,
                });
            }
        }

        if size_of::<T>() == 4 {
            let int = if size_of::<T>() < USIZE_SIZE {
                self.read_usize(position, 32) as u32
            } else {
                self.read_int::<u32>(position, 32)?
            };
            Ok(T::from_f32_unchecked(f32::from_bits(int)))
        } else {
            let int = self.read_int::<u64>(position, 64)?;
            Ok(T::from_f64_unchecked(f64::from_bits(int)))
        }
    }
}

impl<E: Endianness> From<Vec<u8>> for BitBuffer<E> {
    fn from(bytes: Vec<u8>) -> Self {
        let byte_len = bytes.len();
        BitBuffer {
            bytes,
            byte_len,
            bit_len: byte_len * 8,
            endianness: PhantomData,
        }
    }
}

impl<E: Endianness> Debug for BitBuffer<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BitBuffer {{ bit_len: {}, endianness: {} }}",
            self.bit_len,
            if E::is_le() {
                "LittleEndian"
            } else {
                "BigEndian"
            }
        )
    }
}
