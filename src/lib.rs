//! Tools for reading and writing data types of arbitrary bit length and might not be byte-aligned in the source data
//!
//! The main way of reading the binary data is to first create a [`BitReadBuffer`]
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
//! For writing the data you wrap the output `Vec` into a [`BitWriteStream`] which can then be used in a manner similar to the [`BitReadStream`]
//!
//! - write primitives, Strings and byte arrays, using [`write_bool`], [`write_int`], [`write_float`], [`write_bytes`] and [`write_string`]
//! - write any type implementing the  [`BitWrite`] or [`BitWriteSized`] traits using [`write`] and [`write_sized`]
//!   - [`BitWrite`] is for types that can be written without requiring any size info (e.g. null-terminal strings, floats, whole integers, etc)
//!   - [`BitWriteSized`] is for types that require external sizing information to be written (fixed length strings, arbitrary length integers
//!
//! Just like the read counterparts, [`BitWrite`] and [`BitWriteSized`] traits can be used with `#[derive]` if all fields implement [`BitWrite`] or [`BitWriteSized`].
//!
//! # Examples
//!
//! ```
//! # use bitbuffer::Result;
//! use bitbuffer::{BitReadBuffer, LittleEndian, BitReadStream, BitRead, BitWrite, BitWriteStream};
//!
//! #[derive(BitRead, BitWrite)]
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
//!
//! let mut write_bytes = vec![];
//! let mut write_stream = BitWriteStream::new(&mut write_bytes, LittleEndian);
//! write_stream.write_int(12, 7)?;
//! write_stream.write(&ComplexType {
//!     first: 55,
//!     second: 12,
//!     third: true
//! })?;
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! [`read_bool`]: BitReadStream::read_bool
//! [`read_int`]: BitReadStream::read_int
//! [`read_float`]: BitReadStream::read_float
//! [`read_bytes`]: BitReadStream::read_bytes
//! [`read_string`]: BitReadStream::read_string
//! [`read`]: BitReadStream::read
//! [`read_sized`]: BitReadStream::read_sized
//! [`write_bool`]: BitWriteStream::write_bool
//! [`write_int`]: BitWriteStream::write_int
//! [`write_float`]: BitWriteStream::write_float
//! [`write_bytes`]: BitWriteStream::write_bytes
//! [`write_string`]: BitWriteStream::write_string
//! [`write`]: BitWriteStream::write
//! [`write_sized`]: BitWriteStream::write_sized

#![warn(missing_docs)]

use thiserror::Error;

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

/// A number of traits to help being generic over numbers
pub mod num_traits;
mod read;
mod readbuffer;
mod readstream;
mod write;
mod writebuffer;
mod writestream;

/// Errors that can be returned when trying to read from or write to a buffer
#[derive(Debug, Error)]
pub enum BitError {
    /// Too many bits requested to fit in the requested data type
    #[error(
        "Too many bits requested to fit in the requested data type, requested to read {} bits while only {} fit in the datatype",
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
        "Not enough data in the buffer to read all requested bits, requested to read {} bits while only {} bits are left",
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
        "The requested position is outside the bounds of the stream, requested position {} while the stream or buffer is only {} bits long",
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
        "Unmatched discriminant '{}' found while trying to read enum '{}'",
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
    #[error("The read slice of bytes are not valid utf8: {}", _0)]
    Utf8Error(Utf8Error, usize),
    /// The string that was requested to be written does not fit in the specified fixed length
    #[error(
        "The string that was requested to be written does not fit in the specified fixed length, string is {} bytes long, while a size of {} has been specified",
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

/// Either the read bits in the requested format or a [`BitError`]
pub type Result<T, E = BitError> = std::result::Result<T, E>;

/// Get the number of bits required to read a type from stream
///
/// If the number of bits needed can not be determined beforehand `None` is returned
#[inline(always)]
pub fn bit_size_of<'a, T: BitRead<'a, LittleEndian>>() -> Option<usize> {
    T::bit_size()
}

/// Get the number of bits required to read a type from stream given an input size
///
/// If the number of bits needed can not be determined beforehand `None` is returned
#[inline(always)]
pub fn bit_size_of_sized<'a, T: BitReadSized<'a, LittleEndian>>(size: usize) -> Option<usize> {
    T::bit_size_sized(size)
}
