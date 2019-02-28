use std::mem::size_of;
use std::ops::BitOrAssign;
use std::rc::Rc;

use num_traits::{Float, PrimInt};

use crate::endianness::Endianness;
use crate::is_signed::IsSigned;
use crate::BitBuffer;
use crate::{BitRead, BitReadSized, ReadError, Result};

/// Stream that provides an easy way to iterate trough a [`BitBuffer`]
///
/// # Examples
///
/// ```
/// use bitstream_reader::{BitBuffer, BitStream, LittleEndian};
///
/// let bytes = vec![
///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
/// ];
/// let buffer = BitBuffer::new(bytes, LittleEndian);
/// let mut stream = BitStream::new(buffer);
/// ```
///
/// [`BitBuffer`]: struct.BitBuffer.html
pub struct BitStream<E>
where
    E: Endianness,
{
    buffer: Rc<BitBuffer<E>>,
    start_pos: usize,
    pos: usize,
    bit_len: usize,
}

impl<E> BitStream<E>
where
    E: Endianness,
{
    /// Create a new stream for a [`BitBuffer`]
    ///
    /// # Examples
    ///
    /// ```
    /// use bitstream_reader::{BitBuffer, BitStream, LittleEndian};
    ///
    /// let bytes = vec![
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
    /// let mut stream = BitStream::new(buffer);
    /// ```
    ///
    /// [`BitBuffer`]: struct.BitBuffer.html
    pub fn new(buffer: BitBuffer<E>) -> Self {
        BitStream {
            start_pos: 0,
            pos: 0,
            bit_len: buffer.bit_len(),
            buffer: Rc::new(buffer),
        }
    }

    fn verify_bits_left(&self, count: usize) -> Result<()> {
        if self.bits_left() < count {
            Err(ReadError::NotEnoughData {
                bits_left: self.bits_left(),
                requested: count,
            })
        } else {
            Ok(())
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
    /// # use bitstream_reader::{BitBuffer, BitStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitStream::new(buffer);
    /// assert_eq!(stream.read_bool()?, true);
    /// assert_eq!(stream.read_bool()?, false);
    /// assert_eq!(stream.pos(), 2);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    pub fn read_bool(&mut self) -> Result<bool> {
        self.verify_bits_left(1)?;
        let result = self.buffer.read_bool(self.pos);
        if result.is_ok() {
            self.pos += 1;
        }
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
    /// # use bitstream_reader::{BitBuffer, BitStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitStream::new(buffer);
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
    pub fn read_int<T>(&mut self, count: usize) -> Result<T>
    where
        T: PrimInt + BitOrAssign + IsSigned,
    {
        self.verify_bits_left(count)?;
        let result = self.buffer.read_int(self.pos, count);
        if result.is_ok() {
            self.pos += count;
        }
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
    /// # use bitstream_reader::{BitBuffer, BitStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitStream::new(buffer);
    /// let result = stream.read_float::<f32>()?;
    /// assert_eq!(stream.pos(), 32);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    pub fn read_float<T>(&mut self) -> Result<T>
    where
        T: Float,
    {
        let count = size_of::<T>() * 8;
        self.verify_bits_left(count)?;
        let result = self.buffer.read_float(self.pos);
        if result.is_ok() {
            self.pos += count;
        }
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
    /// # use bitstream_reader::{BitBuffer, BitStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitStream::new(buffer);
    /// assert_eq!(stream.read_bytes(3)?, &[0b1011_0101, 0b0110_1010, 0b1010_1100]);
    /// assert_eq!(stream.pos(), 24);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    pub fn read_bytes(&mut self, byte_count: usize) -> Result<Vec<u8>> {
        let count = byte_count * 8;
        self.verify_bits_left(count)?;
        let result = self.buffer.read_bytes(self.pos, byte_count);
        if result.is_ok() {
            self.pos += count;
        }
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
    /// # let mut stream = BitStream::new(buffer);
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
    pub fn read_string(&mut self, byte_len: Option<usize>) -> Result<String> {
        let result = self.buffer.read_string(self.pos, byte_len)?;
        let read = match byte_len {
            Some(len) => len * 8,
            None => (result.len() + 1) * 8,
        };
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
    /// # use bitstream_reader::{BitBuffer, BitStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitStream::new(buffer);
    /// let mut bits = stream.read_bits(3)?;
    /// assert_eq!(stream.pos(), 3);
    /// assert_eq!(bits.pos(), 0);
    /// assert_eq!(bits.bit_len(), 3);
    /// assert_eq!(stream.read_int::<u8>(3)?, 0b110);
    /// assert_eq!(bits.read_int::<u8>(3)?, 0b101);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    pub fn read_bits(&mut self, count: usize) -> Result<Self> {
        self.verify_bits_left(count)?;
        let result = BitStream {
            buffer: Rc::clone(&self.buffer),
            start_pos: self.pos,
            pos: self.pos,
            bit_len: count,
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
    /// # use bitstream_reader::{BitBuffer, BitStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitStream::new(buffer);
    /// stream.skip(3)?;
    /// assert_eq!(stream.pos(), 3);
    /// assert_eq!(stream.read_int::<u8>(3)?, 0b110);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`ReadError::NotEnoughData`]: enum.ReadError.html#variant.NotEnoughData
    pub fn skip(&mut self, count: usize) -> Result<()> {
        self.verify_bits_left(count)?;
        self.pos += count;
        Ok(())
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
    /// # use bitstream_reader::{BitBuffer, BitStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitStream::new(buffer);
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
        if pos > self.bit_len {
            return Err(ReadError::IndexOutOfBounds {
                pos,
                size: self.bit_len,
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
    /// # use bitstream_reader::{BitBuffer, BitStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitStream::new(buffer);
    /// assert_eq!(stream.bit_len(), 64);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    /// Get the current position in the stream
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitstream_reader::{BitBuffer, BitStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitStream::new(buffer);
    /// assert_eq!(stream.pos(), 0);
    /// stream.skip(5)?;
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
    /// # use bitstream_reader::{BitBuffer, BitStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitStream::new(buffer);
    /// assert_eq!(stream.bits_left(), 64);
    /// stream.skip(5)?;
    /// assert_eq!(stream.bits_left(), 59);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn bits_left(&self) -> usize {
        self.bit_len - self.pos()
    }

    /// Read a value based on the provided type
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitstream_reader::{BitBuffer, BitStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitStream::new(buffer);
    /// let int: u8 = stream.read()?;
    /// assert_eq!(int, 0b1011_0101);
    /// let boolean: bool = stream.read()?;
    /// assert_eq!(false, boolean);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    /// #[inline(always)]
    pub fn read<T: BitRead<E>>(&mut self) -> Result<T> {
        T::read(self)
    }

    /// Read a value based on the provided type and size
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitstream_reader::{BitBuffer, BitStream, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # let bytes = vec![
    /// #     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    /// #     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// # ];
    /// # let buffer = BitBuffer::new(bytes, LittleEndian);
    /// # let mut stream = BitStream::new(buffer);
    /// let int: u8 = stream.read_sized(7)?;
    /// assert_eq!(int, 0b011_0101);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    #[inline(always)]
    pub fn read_sized<T: BitReadSized<E>>(&mut self, size: usize) -> Result<T> {
        T::read(self, size)
    }
}

impl<E: Endianness> Clone for BitStream<E> {
    fn clone(&self) -> Self {
        BitStream {
            buffer: Rc::clone(&self.buffer),
            start_pos: self.pos,
            pos: self.pos,
            bit_len: self.bit_len,
        }
    }
}

impl<E: Endianness> From<BitBuffer<E>> for BitStream<E> {
    fn from(buffer: BitBuffer<E>) -> Self {
        BitStream::new(buffer)
    }
}

impl<E: Endianness> From<Vec<u8>> for BitStream<E> {
    fn from(bytes: Vec<u8>) -> Self {
        BitStream::new(BitBuffer::from(bytes))
    }
}
