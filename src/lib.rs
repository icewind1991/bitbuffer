//! Tools for reading data types of arbitrary bit length and might not be byte-aligned in the source data
//!
//! The main way of handling with the binary data is to first create a [`BitBuffer`]
//! ,wrap it into a [`BitStream`] and then read from the stream.
//!
//! Once you have a BitStream, there are 2 different approaches of reading data
//!
//! - read primitives, Strings and byte arrays, using [`read_bool`], [`read_int`], [`read_float`], [`read_bytes`] and [`read_string`]
//! - read any type implementing the  [`BitRead`] or [`BitReadSized`] traits using [`read`] and [`read_sized`]
//!   - [`BitRead`] is for types that can be read without requiring any size info (e.g. null-terminal strings, floats, whole integers, etc)
//!   - [`BitReadSized`] is for types that require external sizing information to be read (fixed length strings, arbitrary length integers
//!
//! The [`BitRead`] and [`BitReadSized`] traits can be used with `#[derive]` if all fields implement [`BitRead`] or [`BitReadSized`].
//!
//! # Examples
//!
//! ```
//! # use bitbuffer::Result;
//! use bitbuffer::{BitBuffer, LittleEndian, BitStream, BitRead};
//!
//! #[derive(BitRead)]
//! struct ComplexType {
//!     first: u8,
//!     #[size = 15]
//!     second: u16,
//!     third: bool,
//! }
//!
//! # fn main() -> Result<()> {
//! let bytes = vec![
//!     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
//!     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
//! ];
//! let buffer = BitBuffer::new(bytes, LittleEndian);
//! let mut stream = BitStream::new(buffer);
//! let value: u8 = stream.read_int(7)?;
//! let complex: ComplexType = stream.read()?;
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! [`BitBuffer`]: struct.BitBuffer.html
//! [`BitStream`]: struct.BitStream.html
//! [`read_bool`]: struct.BitStream.html#method.read_bool
//! [`read_int`]: struct.BitStream.html#method.read_int
//! [`read_float`]: struct.BitStream.html#method.read_float
//! [`read_bytes`]: struct.BitStream.html#method.read_bytes
//! [`read_string`]: struct.BitStream.html#method.read_string
//! [`read`]: struct.BitStream.html#method.read
//! [`read_sized`]: struct.BitStream.html#method.read_sized
//! [`BitRead`]: trait.BitRead.html
//! [`BitReadSized`]: trait.BitReadSized.html

#![warn(missing_docs)]

use std::error::Error;
use std::fmt;
use std::fmt::Display;
pub use std::string::FromUtf8Error;

pub use bitbuffer_derive::{BitRead, BitReadSized};
pub use buffer::BitBuffer;
pub use endianness::*;
pub use read::{BitRead, BitReadSized, LazyBitRead, LazyBitReadSized};
pub use stream::BitStream;

mod buffer;
mod endianness;
mod is_signed;
mod read;
mod stream;
mod unchecked_primitive;

/// Errors that can be returned when trying to read from a buffer
#[derive(Debug)]
pub enum ReadError {
    /// Too many bits requested to fit in the requested data type
    TooManyBits {
        /// The number of bits requested to read
        requested: usize,
        /// The number of bits that fit in the requested data type
        max: usize,
    },
    /// Not enough data in the buffer to read all requested bits
    NotEnoughData {
        /// The number of bits requested to read
        requested: usize,
        /// the number of bits left in the buffer
        bits_left: usize,
    },
    /// The requested position is outside the bounds of the stream or buffer
    IndexOutOfBounds {
        /// The requested position
        pos: usize,
        /// the number of bits in the buffer
        size: usize,
    },
    /// Unmatched discriminant found while trying to read an enum
    UnmatchedDiscriminant {
        /// The read discriminant
        discriminant: usize,
        /// The name of the enum that is trying to be read
        enum_name: String,
    },
    /// The read slice of bytes are not valid utf8
    Utf8Error(FromUtf8Error),
}

impl Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReadError::TooManyBits { requested, max } =>
                write!(f, "Too many bits requested to fit in the requested data type, requested to read {} bits while only {} fit in the datatype", requested, max),
            ReadError::NotEnoughData { requested, bits_left } =>
                write!(f, "Not enough data in the buffer to read all requested bits, requested to read {} bits while only {} bits are left", requested, bits_left),
            ReadError::IndexOutOfBounds { pos, size } =>
                write!(f, "The requested position is outside the bounds of the stream, requested position {} while the stream or buffer is only {} bits long", pos, size),
            ReadError::UnmatchedDiscriminant { discriminant, enum_name } =>
                write!(f, "Unmatched discriminant '{}' found while trying to read enum '{}'", discriminant, enum_name),
            ReadError::Utf8Error(err) => err.fmt(f)
        }
    }
}

impl From<FromUtf8Error> for ReadError {
    fn from(err: FromUtf8Error) -> ReadError {
        ReadError::Utf8Error(err)
    }
}

impl Error for ReadError {
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            ReadError::Utf8Error(err) => Some(err),
            _ => None,
        }
    }
}

/// Either the read bits in the requested format or a [`ReadError`](enum.ReadError.html)
pub type Result<T> = std::result::Result<T, ReadError>;

/// Get the number of bits required to read a type from stream
#[inline(always)]
pub fn bit_size_of<T: BitRead<LittleEndian>>() -> Option<usize> {
    T::bit_size()
}

/// Get the number of bits required to read a type from stream
#[inline(always)]
pub fn bit_size_of_sized<T: BitReadSized<LittleEndian>>(size: usize) -> Option<usize> {
    T::bit_size_sized(size)
}
