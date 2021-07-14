//! Tools for reading data types of arbitrary bit length and might not be byte-aligned in the source data
//!
//! The main way of handling with the binary data is to first create a [`BitReadBuffer`]
//! ,wrap it into a [`BitReadStream`] and then read from the stream.
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
//! use bitbuffer::{BitReadBuffer, LittleEndian, BitReadStream, BitRead};
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
//! let buffer = BitReadBuffer::new(&bytes, LittleEndian);
//! let mut stream = BitReadStream::new(buffer);
//! let value: u8 = stream.read_int(7)?;
//! let complex: ComplexType = stream.read()?;
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! [`BitReadBuffer`]: struct.BitReadBuffer.html
//! [`BitReadStream`]: struct.BitReadStream.html
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

use err_derive::Error;

pub use bitbuffer_derive::{BitRead, BitReadSized, BitWrite, BitWriteSized};
pub use endianness::*;
pub use read::{BitRead, BitReadSized, LazyBitRead, LazyBitReadSized};
pub use readbuffer::BitReadBuffer;
pub use readstream::BitReadStream;
use std::str::Utf8Error;
use std::string::FromUtf8Error;
pub use write::{BitWrite, BitWriteSized};
pub use writestream::BitWriteStream;

mod endianness;
mod num_traits;
mod read;
mod readbuffer;
mod readstream;
mod write;
mod writebuffer;
mod writestream;

/// Errors that can be returned when trying to read from a buffer
#[derive(Debug, Error)]
pub enum BitError {
    /// Too many bits requested to fit in the requested data type
    #[error(
        display = "Too many bits requested to fit in the requested data type, requested to read {} bits while only {} fit in the datatype",
        requested,
        max
    )]
    TooManyBits {
        /// The number of bits requested to read
        requested: usize,
        /// The number of bits that fit in the requested data type
        max: usize,
    },
    /// Not enough data in the buffer to read all requested bits
    #[error(
        display = "Not enough data in the buffer to read all requested bits, requested to read {} bits while only {} bits are left",
        requested,
        bits_left
    )]
    NotEnoughData {
        /// The number of bits requested to read
        requested: usize,
        /// the number of bits left in the buffer
        bits_left: usize,
    },
    /// The requested position is outside the bounds of the stream or buffer
    #[error(
        display = "The requested position is outside the bounds of the stream, requested position {} while the stream or buffer is only {} bits long",
        pos,
        size
    )]
    IndexOutOfBounds {
        /// The requested position
        pos: usize,
        /// the number of bits in the buffer
        size: usize,
    },
    /// Unmatched discriminant found while trying to read an enum
    #[error(
        display = "Unmatched discriminant '{}' found while trying to read enum '{}'",
        discriminant,
        enum_name
    )]
    UnmatchedDiscriminant {
        /// The read discriminant
        discriminant: usize,
        /// The name of the enum that is trying to be read
        enum_name: String,
    },
    /// The read slice of bytes are not valid utf8
    #[error(display = "The read slice of bytes are not valid utf8: {}", _0)]
    Utf8Error(Utf8Error, usize),
    /// The string that was requested to be written does not fit in the specified fixed length
    #[error(
        display = "The string that was requested to be written does not fit in the specified fixed length, string is {} bytes long, while a size of {} has been specified",
        string_length,
        requested_length
    )]
    StringToLong {
        /// Length of the string that was requested to be written
        string_length: usize,
        /// The requested fixed size to encode the string into
        requested_length: usize,
    },
}

impl From<FromUtf8Error> for BitError {
    fn from(err: FromUtf8Error) -> Self {
        BitError::Utf8Error(err.utf8_error(), err.as_bytes().len())
    }
}

/// Either the read bits in the requested format or a [`ReadError`](enum.ReadError.html)
pub type Result<T> = std::result::Result<T, BitError>;

/// Get the number of bits required to read a type from stream
#[inline(always)]
pub fn bit_size_of<'a, T: BitRead<'a, LittleEndian>>() -> Option<usize> {
    T::bit_size()
}

/// Get the number of bits required to read a type from stream
#[inline(always)]
pub fn bit_size_of_sized<'a, T: BitReadSized<'a, LittleEndian>>(size: usize) -> Option<usize> {
    T::bit_size_sized(size)
}
