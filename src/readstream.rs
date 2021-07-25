use std::mem::size_of;
use std::ops::BitOrAssign;

use num_traits::{Float, PrimInt};

use crate::endianness::Endianness;
use crate::num_traits::{IsSigned, UncheckedPrimitiveFloat, UncheckedPrimitiveInt};
use crate::readbuffer::Data;
use crate::BitReadBuffer;
use crate::{BitError, BitRead, BitReadSized, Result};
use std::borrow::Cow;
use std::cmp::min;

/// Stream that provides an easy way to iterate trough a [`BitBuffer`]
///
/// # Examples
///
/// ```
/// use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian};
///
/// let bytes = vec![
///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
/// ];
/// let buffer = BitReadBuffer::new(&bytes, LittleEndian);
/// let mut stream = BitReadStream::new(buffer);
/// ```
///
/// [`BitBuffer`]: struct.BitBuffer.html
#[derive(Debug)]
pub struct BitReadStream<'a, E>
where
    E: Endianness,
{
    buffer: BitReadBuffer<'a, E>,
    start_pos: usize,
    pos: usize,
}

impl<'a, E> BitReadStream<'a, E>
where
    E: Endianness,
{
    /// Create a new stream from a [`BitBuffer`]
    ///
    /// # Examples
    ///
    /// ```
    /// use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian};
    ///
    /// let bytes = vec![
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// let mut stream = BitReadStream::new(buffer);
    /// ```
    ///
    /// [`BitBuffer`]: struct.BitBuffer.html
    pub fn new(buffer: BitReadBuffer<'a, E>) -> Self {
        BitReadStream {
            start_pos: 0,
            pos: 0,
            buffer,
        }
    }

    /// Read a single bit from the stream as boolean
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the stream
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// assert_eq!(stream.read_bool()?, true);
    /// assert_eq!(stream.read_bool()?, false);
    /// assert_eq!(stream.pos(), 2);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    #[inline]
    pub fn read_bool(&mut self) -> Result<bool> {
        let result = self.buffer.read_bool(self.pos);
        if result.is_ok() {
            self.pos += 1;
        }
        result
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_bool_unchecked(&mut self) -> bool {
        let result = self.buffer.read_bool_unchecked(self.pos);
        self.pos += 1;
        result
    }

    /// Read a sequence of bits from the stream as integer
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the stream
    /// - [`ReadError::TooManyBits`]: to many bits requested for the chosen integer type
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// assert_eq!(stream.read_int::<u16>(3)?, 0b101);
    /// assert_eq!(stream.read_int::<u16>(3)?, 0b110);
    /// assert_eq!(stream.pos(), 6);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    /// [`ReadError::TooManyBits`]: enum.ReadError.html#variant.TooManyBits
    #[inline]
    pub fn read_int<T>(&mut self, count: usize) -> Result<T>
    where
        T: PrimInt + BitOrAssign + IsSigned + UncheckedPrimitiveInt,
    {
        let result = self.buffer.read_int(self.pos, count);
        if result.is_ok() {
            self.pos += count;
        }
        result
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_int_unchecked<T>(&mut self, count: usize, end: bool) -> T
    where
        T: PrimInt + BitOrAssign + IsSigned + UncheckedPrimitiveInt,
    {
        let result = self.buffer.read_int_unchecked(self.pos, count, end);
        self.pos += count;
        result
    }

    /// Read a sequence of bits from the stream as float
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the stream
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// let result = stream.read_float::<f32>()?;
    /// assert_eq!(stream.pos(), 32);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    #[inline]
    pub fn read_float<T>(&mut self) -> Result<T>
    where
        T: Float + UncheckedPrimitiveFloat,
    {
        let count = size_of::<T>() * 8;
        let result = self.buffer.read_float(self.pos);
        if result.is_ok() {
            self.pos += count;
        }
        result
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_float_unchecked<T>(&mut self, end: bool) -> T
    where
        T: Float + UncheckedPrimitiveFloat,
    {
        let count = size_of::<T>() * 8;
        let result = self.buffer.read_float_unchecked(self.pos, end);
        self.pos += count;
        result
    }

    /// Read a series of bytes from the stream
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the stream
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # use std::borrow::Borrow;
    /// let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// assert_eq!(stream.read_bytes(3)?.to_vec(), &[0b1011_0101, 0b0110_1010, 0b1010_1100]);
    /// assert_eq!(stream.pos(), 24);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    #[inline]
    pub fn read_bytes(&mut self, byte_count: usize) -> Result<Cow<'a, [u8]>> {
        let count = byte_count * 8;
        let result = self.buffer.read_bytes(self.pos, byte_count);
        if result.is_ok() {
            self.pos += count;
        }
        result
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_bytes_unchecked(&mut self, byte_count: usize) -> Cow<'a, [u8]> {
        let count = byte_count * 8;
        let result = self.buffer.read_bytes_unchecked(self.pos, byte_count);
        self.pos += count;
        result
    }

    /// Read a series of bytes from the stream as utf8 string
    ///
    /// You can either read a fixed number of bytes, or a dynamic length null-terminated string
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the stream
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
    /// # let mut stream = BitReadStream::new(buffer);
    /// // Fixed length string
    /// stream.set_pos(0);
    /// assert_eq!(stream.read_string(Some(11))?, "Hello world".to_owned());
    /// assert_eq!(11 * 8, stream.pos());
    /// // fixed length with null padding
    /// stream.set_pos(0);
    /// assert_eq!(stream.read_string(Some(16))?, "Hello world".to_owned());
    /// assert_eq!(16 * 8, stream.pos());
    /// // null terminated
    /// stream.set_pos(0);
    /// assert_eq!(stream.read_string(None)?, "Hello world".to_owned());
    /// assert_eq!(12 * 8, stream.pos()); // 1 more for the terminating null byte
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    /// [`ReadError::Utf8Error`]: enum.ReadError.html#variant.Utf8Error
    #[inline]
    pub fn read_string(&mut self, byte_len: Option<usize>) -> Result<Cow<'a, str>> {
        let max_length = self.bits_left() / 8;

        let result = self.buffer.read_string(self.pos, byte_len).map_err(|err| {
            // still advance the stream on malformed utf8
            if let BitError::Utf8Error(_, len) = &err {
                self.pos += match byte_len {
                    Some(len) => len * 8,
                    None => min((len + 1) * 8, max_length * 8),
                };
            }
            err
        })?;
        let read = match byte_len {
            Some(len) => len * 8,
            None => (result.len() + 1) * 8,
        };

        // due to how sub buffer/streams work, the result string can be longer than the current stream
        // (but not the top level buffer)
        // thus we trim the resulting string to make sure it fits in the source stream
        if read > self.bits_left() {
            // find the maximum well-formed utf8 string that fits in max_len
            let mut acc = String::new();
            for c in result.chars() {
                if acc.len() + c.len_utf8() > max_length {
                    break;
                }
                acc.push(c);
            }
            self.pos += acc.len() * 8;
            return Ok(Cow::Owned(acc));
        }
        self.pos += read;
        Ok(result)
    }

    /// Read a sequence of bits from the stream as a BitStream
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the stream
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// let mut bits = stream.read_bits(3)?;
    /// assert_eq!(stream.pos(), 3);
    /// assert_eq!(bits.pos(), 0);
    /// assert_eq!(bits.bit_len(), 3);
    /// assert_eq!(stream.read_int::<u8>(3)?, 0b110);
    /// assert_eq!(bits.read_int::<u8>(3)?, 0b101);
    /// assert_eq!(true, bits.read_int::<u8>(1).is_err());
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    pub fn read_bits(&mut self, count: usize) -> Result<Self> {
        let result = BitReadStream {
            buffer: self.buffer.get_sub_buffer(self.pos + count)?,
            start_pos: self.pos,
            pos: self.pos,
        };
        self.pos += count;
        Ok(result)
    }

    /// Skip a number of bits in the stream
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`]: not enough bits available in the stream to skip
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// stream.skip_bits(3)?;
    /// assert_eq!(stream.pos(), 3);
    /// assert_eq!(stream.read_int::<u8>(3)?, 0b110);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    pub fn skip_bits(&mut self, count: usize) -> Result<()> {
        if count <= self.bits_left() {
            self.pos += count;
            Ok(())
        } else {
            Err(BitError::NotEnoughData {
                requested: count,
                bits_left: self.bits_left(),
            })
        }
    }

    /// Set the position of the stream
    ///
    /// # Errors
    ///
    /// - [`ReadError::IndexOutOfBounds`]: new position is outside the bounds of the stream
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// stream.set_pos(3)?;
    /// assert_eq!(stream.pos(), 3);
    /// assert_eq!(stream.read_int::<u8>(3)?, 0b110);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::IndexOutOfBounds`]: enum.ReadError.html#variant.IndexOutOfBounds
    pub fn set_pos(&mut self, pos: usize) -> Result<()> {
        if pos > self.bit_len() {
            return Err(BitError::IndexOutOfBounds {
                pos,
                size: self.bit_len(),
            });
        }
        self.pos = pos + self.start_pos;
        Ok(())
    }

    /// Get the length of the stream in bits
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// assert_eq!(stream.bit_len(), 64);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn bit_len(&self) -> usize {
        self.buffer.bit_len() - self.start_pos
    }

    /// Get the current position in the stream
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// assert_eq!(stream.pos(), 0);
    /// stream.skip_bits(5)?;
    /// assert_eq!(stream.pos(), 5);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn pos(&self) -> usize {
        self.pos - self.start_pos
    }

    /// Get the number of bits left in the stream
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// assert_eq!(stream.bits_left(), 64);
    /// stream.skip_bits(5)?;
    /// assert_eq!(stream.bits_left(), 59);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn bits_left(&self) -> usize {
        self.bit_len() - self.pos()
    }

    /// Read a value based on the provided type
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// let int: u8 = stream.read()?;
    /// assert_eq!(int, 0b1011_0101);
    /// let boolean: bool = stream.read()?;
    /// assert_eq!(false, boolean);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// use bitbuffer::BitRead;
    /// #
    /// #[derive(BitRead, Debug, PartialEq)]
    /// struct ComplexType {
    ///     first: u8,
    ///     #[size = 15]
    ///     second: u16,
    ///     third: bool,
    /// }
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// let data: ComplexType = stream.read()?;
    /// assert_eq!(data, ComplexType {
    ///     first: 0b1011_0101,
    ///     second: 0b010_1100_0110_1010,
    ///     third: true,
    /// });
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn read<T: BitRead<'a, E>>(&mut self) -> Result<T> {
        T::read(self)
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_unchecked<T: BitRead<'a, E>>(&mut self, end: bool) -> Result<T> {
        T::read_unchecked(self, end)
    }

    /// Read a value based on the provided type and size
    ///
    /// The meaning of the size parameter differs depending on the type that is being read
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// let int: u8 = stream.read_sized(7)?;
    /// assert_eq!(int, 0b011_0101);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitReadBuffer::new(&bytes, LittleEndian);
    /// # let mut stream = BitReadStream::new(buffer);
    /// let data: Vec<u16> = stream.read_sized(3)?;
    /// assert_eq!(data, vec![0b0110_1010_1011_0101, 0b1001_1001_1010_1100, 0b1001_1001_1001_1001]);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn read_sized<T: BitReadSized<'a, E>>(&mut self, size: usize) -> Result<T> {
        T::read(self, size)
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn read_sized_unchecked<T: BitReadSized<'a, E>>(
        &mut self,
        size: usize,
        end: bool,
    ) -> Result<T> {
        T::read_unchecked(self, size, end)
    }

    /// Check if we can read a number of bits from the stream
    pub fn check_read(&self, count: usize) -> Result<bool> {
        if self.bits_left() < count + 64 {
            if self.bits_left() < count {
                Err(BitError::NotEnoughData {
                    requested: count,
                    bits_left: self.bits_left(),
                })
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    /// Create an owned copy of this stream
    pub fn to_owned(&self) -> BitReadStream<'static, E> {
        match self.buffer.bytes {
            Data::Owned(_) => BitReadStream {
                // already owned, so buffer.to_owned is a cheap rc clone
                buffer: self.buffer.to_owned(),
                start_pos: self.pos,
                pos: self.pos,
            },
            Data::Borrowed(bytes) => {
                // instead of calling buffer.to_owned blindly, we only copy the bytes that this stream covers
                let byte_pos = self.start_pos / 8;
                let bit_offset = self.start_pos & 7;

                let end = self.buffer.bit_len() / 8 + 1;
                let end = min(end, self.buffer.byte_len());

                let sub_bytes = bytes[byte_pos..end].to_vec();
                let buffer = BitReadBuffer::from(sub_bytes)
                    .get_sub_buffer(self.buffer.bit_len() - self.start_pos + bit_offset)
                    .unwrap();

                BitReadStream {
                    buffer,
                    start_pos: bit_offset,
                    pos: bit_offset + (self.pos - self.start_pos),
                }
            }
        }
    }
}

impl<'a, E: Endianness> Clone for BitReadStream<'a, E> {
    fn clone(&self) -> Self {
        BitReadStream {
            buffer: self.buffer.clone(),
            start_pos: self.pos,
            pos: self.pos,
        }
    }
}

impl<'a, E: Endianness> PartialEq for BitReadStream<'a, E> {
    fn eq(&self, other: &Self) -> bool {
        // clones so we can mut
        let mut self_clone = self.clone();
        self_clone.set_pos(0).ok();
        let mut other_clone = other.clone();
        other_clone.set_pos(0).ok();

        if self_clone.bits_left() != other_clone.bits_left() {
            return false;
        }

        while self_clone.bits_left() > 32 {
            if self_clone.read::<u32>().ok() != other_clone.read().ok() {
                return false;
            }
        }

        while self_clone.bits_left() > 0 {
            if self_clone.read::<bool>().ok() != other_clone.read().ok() {
                return false;
            }
        }

        true
    }
}

impl<'a, E: Endianness> From<BitReadBuffer<'a, E>> for BitReadStream<'a, E> {
    fn from(buffer: BitReadBuffer<'a, E>) -> Self {
        BitReadStream::new(buffer)
    }
}

impl<'a, E: Endianness> From<&'a [u8]> for BitReadStream<'a, E> {
    fn from(bytes: &'a [u8]) -> Self {
        BitReadStream::new(BitReadBuffer::from(bytes))
    }
}

#[cfg(feature = "serde")]
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};

#[cfg(feature = "serde")]
impl<'a, E: Endianness> Serialize for BitReadStream<'a, E> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut stream = self.clone();
        let mut data = stream.read_bytes(self.bits_left() / 8).unwrap().to_vec();
        if stream.bits_left() > 0 {
            data.push(stream.read_sized(stream.bits_left()).unwrap());
        }

        let mut s = serializer.serialize_struct("BitReadStream", 3)?;
        s.serialize_field("data", &data)?;
        s.serialize_field("bit_length", &self.bit_len())?;
        s.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, E: Endianness> Deserialize<'de> for BitReadStream<'static, E> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Data,
            BitLength,
        }

        use std::marker::PhantomData;
        struct ReadStreamVisitor<E>(PhantomData<E>);

        impl<'de, E: Endianness> Visitor<'de> for ReadStreamVisitor<E> {
            type Value = BitReadStream<'static, E>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct BitReadStream")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let data = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let bit_length = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let mut buffer = BitReadBuffer::new_owned(data, E::endianness());
                buffer.truncate(bit_length).map_err(de::Error::custom)?;
                Ok(BitReadStream::new(buffer))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut data = None;
                let mut bit_length = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Data => {
                            if data.is_some() {
                                return Err(de::Error::duplicate_field("secs"));
                            }
                            data = Some(map.next_value()?);
                        }
                        Field::BitLength => {
                            if bit_length.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            bit_length = Some(map.next_value()?);
                        }
                    }
                }
                let data = data.ok_or_else(|| de::Error::missing_field("data"))?;
                let bit_length =
                    bit_length.ok_or_else(|| de::Error::missing_field("bit_length"))?;
                let mut buffer = BitReadBuffer::new_owned(data, E::endianness());
                buffer.truncate(bit_length).map_err(de::Error::custom)?;
                Ok(BitReadStream::new(buffer))
            }
        }

        const FIELDS: &'static [&'static str] = &["data", "bit_length"];
        deserializer.deserialize_struct("BitReadStream", FIELDS, ReadStreamVisitor(PhantomData))
    }
}

#[cfg(feature = "serde")]
#[test]
fn test_serde_roundtrip() {
    use crate::LittleEndian;

    let mut buffer = BitReadBuffer::new_owned(vec![55; 8], LittleEndian);
    buffer.truncate(61).unwrap();
    let stream = BitReadStream::new(buffer);
    assert_eq!(61, stream.bit_len());

    let json = serde_json::to_string(&stream).unwrap();

    dbg!(&json);

    let result: BitReadStream<LittleEndian> = serde_json::from_str(&json).unwrap();

    assert_eq!(result, stream);
}
