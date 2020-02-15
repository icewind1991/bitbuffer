use num_traits::{Float, PrimInt};
use std::iter::{once, repeat};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{BitOrAssign, BitXor};

use crate::endianness::Endianness;
use crate::num_traits::{IntoBytes, IsSigned, UncheckedPrimitiveFloat, UncheckedPrimitiveInt};
use crate::{ReadError, Result};

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
/// let mut stream = BitWriteStream::new(LittleEndian);
///
/// stream.write_bool(false)?;
/// stream.write_int(123u16, 15)?;
/// # Ok(())
/// # }
/// ```
///
/// [`BitBuffer`]: struct.BitBuffer.html
pub struct BitWriteStream<E>
where
    E: Endianness,
{
    bytes: Vec<u8>,
    bit_len: usize,
    endianness: PhantomData<E>,
}

impl<E> BitWriteStream<E>
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
    /// let mut stream = BitWriteStream::new(LittleEndian);
    /// ```
    pub fn new(_endianness: E) -> Self {
        BitWriteStream {
            bytes: Vec::new(),
            bit_len: 0,
            endianness: PhantomData,
        }
    }
}

impl<E> BitWriteStream<E>
where
    E: Endianness,
{
    /// The number of written bits in the buffer
    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    /// The number of written bytes in the buffer
    pub fn byte_len(&self) -> usize {
        self.bytes.len()
    }

    fn push_non_fit_bits<I>(&mut self, bits: I, count: usize)
    where
        I: ExactSizeIterator,
        I: DoubleEndedIterator<Item = u8>,
    {
        debug_assert!(bits.len() == count / 8);
        let counts = repeat(8)
            .take(bits.len() - 1)
            .chain(once(count - (bits.len() - 1) * 8));
        if E::is_le() {
            bits.zip(counts)
                .for_each(|(chunk, count)| self.push_bits(chunk as usize, count))
        } else {
            bits.rev()
                .zip(counts)
                .for_each(|(chunk, count)| self.push_bits(chunk as usize, count))
        }
    }

    /// Push up to an usize worth of bits
    fn push_bits(&mut self, bits: usize, count: usize) {
        debug_assert!(count < USIZE_BITS - 8);

        let bit_offset = self.bit_len & 7;
        let last_written_byte = self.bytes.pop().unwrap_or(0);
        let merged_byte_count = (count + bit_offset + 7) / 8;

        if E::is_le() {
            let merged = last_written_byte as usize | bits << bit_offset;
            self.bytes
                .extend_from_slice(&merged.to_le_bytes()[0..merged_byte_count]);
        } else {
            let merged = ((last_written_byte as usize) << (USIZE_BITS - 8))
                | bits << (USIZE_BITS - bit_offset - count);
            self.bytes
                .extend_from_slice(&merged.to_be_bytes()[0..merged_byte_count]);
        }
        self.bit_len += count;
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
    /// let mut stream = BitWriteStream::new(LittleEndian);
    /// stream.write_bool(true)?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn write_bool(&mut self, value: bool) -> Result<()> {
        self.push_bits(value as usize, 1);
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
    /// let mut stream = BitWriteStream::new(LittleEndian);
    /// stream.write_int(123u16, 15)?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn write_int<T>(&mut self, value: T, count: usize) -> Result<()>
    where
        T: PrimInt + BitOrAssign + IsSigned + UncheckedPrimitiveInt + BitXor + IntoBytes,
    {
        let type_bit_size = size_of::<T>() * 8;

        if type_bit_size < count {
            return Err(ReadError::TooManyBits {
                requested: count,
                max: type_bit_size,
            });
        }

        if type_bit_size < USIZE_BITS {
            if T::is_signed() {
                todo!()
            } else {
                self.push_bits(value.into_usize_unchecked(), count);
            }
        } else {
            self.push_non_fit_bits(value.into_bytes(), count)
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
    /// let mut stream = BitWriteStream::new(LittleEndian);
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
        if size_of::<T>() == 4 {
            if size_of::<T>() < USIZE_SIZE {
                self.push_bits(value.to_f32().unwrap().to_bits() as usize, 32);
            } else {
                self.push_non_fit_bits(value.to_f32().unwrap().to_bits().into_bytes(), 32)
            };
        } else {
            self.push_non_fit_bits(value.to_f64().unwrap().to_bits().into_bytes(), 64)
        }

        Ok(())
    }

    /// Convert the write buffer into the written bytes
    pub fn finish(self) -> Vec<u8> {
        self.bytes
    }
}