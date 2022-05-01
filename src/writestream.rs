use num_traits::{Float, PrimInt};
use std::cmp::min;
use std::mem::size_of;
use std::ops::{BitOrAssign, BitXor};

use crate::endianness::Endianness;
use crate::num_traits::{IsSigned, SplitFitUsize, UncheckedPrimitiveFloat, UncheckedPrimitiveInt};
use crate::writebuffer::WriteBuffer;
use crate::{BitError, BitReadStream, BitWrite, BitWriteSized, Result};
use std::fmt::Debug;

const USIZE_SIZE: usize = size_of::<usize>();
const USIZE_BITS: usize = USIZE_SIZE * 8;

/// Stream that provides an a way to write non bit aligned adata
///
/// # Examples
///
/// ```
/// use bitbuffer::{BitWriteStream, LittleEndian};
/// # use bitbuffer::Result;
///
/// # fn main() -> Result<()> {
/// let mut data = Vec::new();
/// let mut stream = BitWriteStream::new(&mut data, LittleEndian);
///
/// stream.write_bool(false)?;
/// stream.write_int(123u16, 15)?;
/// # Ok(())
/// # }
/// ```
///
/// [`BitBuffer`]: struct.BitBuffer.html
pub struct BitWriteStream<'a, E>
where
    E: Endianness,
{
    buffer: WriteBuffer<'a, E>,
}

impl<'a, E> BitWriteStream<'a, E>
where
    E: Endianness,
{
    /// Create a new write stream
    ///
    /// # Examples
    ///
    /// ```
    /// use bitbuffer::{BitWriteStream, LittleEndian};
    ///
    /// let mut data = Vec::new();
    /// let mut stream = BitWriteStream::new(&mut data, LittleEndian);
    /// ```
    pub fn new(data: &'a mut Vec<u8>, endianness: E) -> Self {
        BitWriteStream {
            buffer: WriteBuffer::new(data, endianness),
        }
    }

    /// Create a new write stream
    ///
    /// Note that the resulting stream will panic when trying to write more data then fits
    /// in the provided slice.
    pub fn from_slice(data: &'a mut [u8], endianness: E) -> Self {
        BitWriteStream {
            buffer: WriteBuffer::for_slice(data, endianness),
        }
    }
}

impl<'a, E> BitWriteStream<'a, E>
where
    E: Endianness,
{
    /// The number of written bits in the buffer
    pub fn bit_len(&self) -> usize {
        self.buffer.bit_len()
    }

    /// The number of written bytes in the buffer
    pub fn byte_len(&self) -> usize {
        (self.buffer.bit_len() + 7) / 8
    }

    fn push_non_fit_bits<I>(&mut self, bits: I, count: usize)
    where
        I: ExactSizeIterator,
        I: DoubleEndedIterator<Item = (usize, u8)>,
    {
        self.buffer.push_non_fit_bits(bits, count)
    }

    /// Push up to an usize worth of bits
    fn push_bits(&mut self, bits: usize, count: usize) {
        if count > 0 {
            self.buffer.push_bits(bits, count)
        }
    }

    /// Write a boolean into the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # use bitbuffer::{BitWriteStream, LittleEndian};
    ///
    /// let mut data = Vec::new();
    /// let mut stream = BitWriteStream::new(&mut data, LittleEndian);
    /// stream.write_bool(true)?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn write_bool(&mut self, value: bool) -> Result<()> {
        self.buffer.push_bool(value);
        Ok(())
    }

    /// Write an integer into the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # use bitbuffer::{BitWriteStream, LittleEndian};
    ///
    /// let mut data = Vec::new();
    /// let mut stream = BitWriteStream::new(&mut data, LittleEndian);
    /// stream.write_int(123u16, 15)?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn write_int<T>(&mut self, value: T, count: usize) -> Result<()>
    where
        T: PrimInt
            + BitOrAssign
            + IsSigned
            + UncheckedPrimitiveInt
            + BitXor
            + Debug
            + SplitFitUsize,
    {
        let type_bit_size = size_of::<T>() * 8;

        if type_bit_size < count {
            return Err(BitError::TooManyBits {
                requested: count,
                max: type_bit_size,
            });
        }

        if type_bit_size < USIZE_BITS || count <= (USIZE_BITS - (self.bit_len() % 8)) {
            self.push_bits(value.into_usize_unchecked(), count);
        } else {
            self.push_non_fit_bits(value.split_fit_usize::<E>(), count)
        }

        Ok(())
    }

    /// Write a float into the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # use bitbuffer::{BitWriteStream, LittleEndian};
    ///
    /// let mut data = Vec::new();
    /// let mut stream = BitWriteStream::new(&mut data, LittleEndian);
    /// stream.write_float(123.15f32)?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn write_float<T>(&mut self, value: T) -> Result<()>
    where
        T: Float + UncheckedPrimitiveFloat,
    {
        if self.buffer.bit_len() & 7 == 0 {
            let bytes = value.to_bytes::<E>();
            self.buffer.extends_from_slice(bytes.as_ref());
        } else {
            self.write_int(value.to_int(), size_of::<T>() * 8)?;
        }

        Ok(())
    }

    /// Write a number of bytes into the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # use bitbuffer::{BitWriteStream, LittleEndian};
    ///
    /// let mut data = Vec::new();
    /// let mut stream = BitWriteStream::new(&mut data, LittleEndian);
    /// stream.write_bytes(&[0, 1, 2 ,3])?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        if self.buffer.bit_len() & 7 == 0 {
            self.buffer.extends_from_slice(bytes);
        } else {
            bytes
                .iter()
                .copied()
                .for_each(|chunk| self.push_bits(chunk as usize, 8));
        }
        Ok(())
    }

    /// Write bits from a read stream into the buffer
    #[inline]
    pub fn write_bits(&mut self, bits: &BitReadStream<E>) -> Result<()> {
        let mut bits = bits.clone();
        let bit_offset = self.bit_len() % 8;
        if bit_offset > 0 {
            let bit_count = min(8 - bit_offset, bits.bits_left());
            let start = bits.read_int::<u8>(bit_count)?;
            self.push_bits(start as usize, bit_count);
        }

        while bits.bits_left() > 32 {
            let chunk = bits.read::<u32>()?;
            self.push_bits(chunk as usize, 32);
        }

        if bits.bits_left() > 0 {
            let end_bits = bits.bits_left();
            let end = bits.read_int::<u32>(end_bits)?;
            self.push_bits(end as usize, end_bits);
        }
        Ok(())
    }

    /// Write a string into the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitbuffer::{BitReadBuffer, LittleEndian, Result};
    /// #
    /// # fn main() -> Result<()> {
    /// # use bitbuffer::{BitWriteStream, LittleEndian};
    ///
    /// let mut data = Vec::new();
    /// let mut stream = BitWriteStream::new(&mut data, LittleEndian);
    /// stream.write_string("zero terminated string", None)?;
    /// stream.write_string("fixed size string, zero padded", Some(64))?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn write_string(&mut self, string: &str, length: Option<usize>) -> Result<()> {
        match length {
            Some(length) => {
                if length < string.len() {
                    return Err(BitError::StringToLong {
                        string_length: string.len(),
                        requested_length: length,
                    });
                }
                self.write_bytes(string.as_bytes())?;
                for _ in 0..(length - string.len()) {
                    self.push_bits(0, 8)
                }
            }
            None => {
                self.write_bytes(string.as_bytes())?;
                self.push_bits(0, 8)
            }
        }
        Ok(())
    }

    /// Write the type to stream
    #[inline]
    pub fn write<T: BitWrite<E>>(&mut self, value: &T) -> Result<()> {
        value.write(self)
    }

    /// Write the type to stream
    #[inline]
    pub fn write_sized<T: BitWriteSized<E>>(&mut self, value: &T, length: usize) -> Result<()> {
        value.write_sized(self, length)
    }

    /// Write the length of a section before the section
    pub fn reserve_length<Err: From<BitError>, F: Fn(&mut BitWriteStream<E>) -> Result<(), Err>>(
        &mut self,
        length_bit_size: usize,
        body_fn: F,
    ) -> Result<(), Err> {
        self.reserve_int(length_bit_size, |stream| {
            let start = stream.bit_len();
            body_fn(stream)?;
            let end = stream.bit_len();
            Ok((end - start) as u64)
        })
    }

    /// Write the length in bytes of a section before the section, the section will be 0 padded to an even byte length
    pub fn reserve_byte_length<
        Err: From<BitError>,
        F: Fn(&mut BitWriteStream<E>) -> Result<(), Err>,
    >(
        &mut self,
        length_bit_size: usize,
        body_fn: F,
    ) -> Result<(), Err> {
        self.reserve_int(length_bit_size, |stream| {
            let start = stream.bit_len();
            body_fn(stream)?;
            let end = stream.bit_len();
            let bit_len = end - start;

            let pad_len = (8 - (bit_len & 7)) & 7;
            stream.push_bits(0, pad_len);

            let byte_len = (bit_len + pad_len) / 8;
            Ok(byte_len as u64)
        })
    }

    /// Reserve the length to write an integer
    pub fn reserve_int<Err: From<BitError>, F: Fn(&mut BitWriteStream<E>) -> Result<u64, Err>>(
        &mut self,
        count: usize,
        body_fn: F,
    ) -> Result<(), Err> {
        let start = self.bit_len();
        self.write_int(0u64, count)?;

        let head_int = body_fn(self)?;
        self.buffer.set_at(start, head_int, count);

        Ok(())
    }
}
