//! Tools for reading integers of arbitrary bit length and non byte-aligned integers and other data types
//!
//!
//!
//! # Examples
//!
//! ```
//! use bitstream_reader::{BitBuffer, LittleEndian, BitStream};
//!
//! let bytes = vec![
//!     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
//!     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
//! ];
//! let buffer = BitBuffer::new(bytes, LittleEndian);
//! let stream = BitStream::new(buffer);

#![warn(missing_docs)]
//#![feature(test)]

// for bench on nightly
//extern crate test;

pub use buffer::BitBuffer;
pub use endianness::*;
pub use read::{Read, ReadSized};
pub use std::string::FromUtf8Error;
pub use stream::BitStream;

mod buffer;
mod endianness;
mod is_signed;
mod read;
mod stream;
#[cfg(test)]
mod tests;

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
    /// The requested position is outside the bounds of the buffer
    IndexOutOfBounds {
        /// The requested position
        pos: usize,
        /// the number of bits in the buffer
        size: usize,
    },
    /// The read slice of bytes are not valid utf8
    Utf8Error(FromUtf8Error),
}

impl From<FromUtf8Error> for ReadError {
    fn from(err: FromUtf8Error) -> ReadError {
        ReadError::Utf8Error(err)
    }
}

/// Either the read bits in the requested format or a [`ReadError`](enum.ReadError.html)
pub type Result<T> = std::result::Result<T, ReadError>;
