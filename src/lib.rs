#![warn(missing_docs)]
#![feature(test)]

//! Tools for reading integers of arbitrary bit length and non byte-aligned integers and other data types

// for bench on nightly
extern crate test;

pub use buffer::{BitBuffer, IsPadded};
pub use stream::BitStream;
pub use endianness::*;

mod buffer;
mod stream;
mod endianness;
mod is_signed;
#[cfg(test)]
mod tests;

/// Errors that can be returned when trying to read from a buffer
#[derive(Debug, PartialEq, Copy, Clone)]
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
}

/// Either the read bits in the requested format or a [`ReadError`](enum.ReadError.html)
pub type Result<T> = std::result::Result<T, ReadError>;
