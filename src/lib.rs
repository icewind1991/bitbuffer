#![feature(test)]

extern crate test;

use std::cmp::min;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ReadError {
    TooManyBits {
        requested: usize,
        max: usize,
    },
    NotEnoughData {
        requested: usize,
        bits_left: usize,
    },
}

pub type Result<T> = std::result::Result<T, ReadError>;

pub struct BitBuffer<'a> {
    bytes: &'a [u8],
    order: ByteOrder,
    bit_len: usize,
}

macro_rules! bitreader_unsigned_be {
    ($buffer:expr, $type:ty, $position:expr, $count:expr) => {
        {
            let size = std::mem::size_of::<$type>() * 8;
            if $count > size {
                return Err(ReadError::TooManyBits { requested: $count, max: size });
            }

            let bits_left = $buffer.bit_len() - $position;

            if $count > bits_left {
                return Err(ReadError::NotEnoughData { requested: $count, bits_left});
            }

            let mut value: $type = 0;

            //let mut i = 0;
            //let mut offset = $position;

//            while i < $count {
//                let remaining = $count - i;
//                let bit_offset = (offset & 7) as u8;
//                let byte_index = (offset / 8) as usize;
//                let byte = $buffer.bytes[byte_index];
//
//                // how much can we read from the current byte
//                let read = min(remaining, 8 - bit_offset);
//
//                println!("{}, {}", remaining, bit_offset);
//                println!("read {} bits from {:#010b}", read, byte);
//
//                let mask = !(0xFFu8.wrapping_shl(read as u32));
//                let shift =  8 - read - bit_offset;
//                let shifted = byte.wrapping_shr(shift as u32);
//                let read_bits = shifted & mask;
//                println!("{:#010b} >> {} = {:#010b}", byte, shift, shifted);
//                println!("{:#010b} & {:#010b} = {:#010b}", mask, shifted, read_bits);
//
//                println!("{:#010b} << {} | {:#010b}", value, read, read_bits);
//                value = value << read | read_bits as $type;
//
//                offset += read as usize;
//                i += read;
//            }

            for i in $position..($position + $count as usize ) {
                let byte_index = (i / 8) as usize;
                let byte = $buffer.bytes[byte_index];
                let shift = 7 - (i % 8);
                let bit = (byte >> shift) as $type & 1;
                value = (value << 1) | bit;
            }

            Ok(value)
        }
    }
}

macro_rules! bitreader_unsigned_le {
    ($buffer:expr, $type:ty, $position:expr, $count:expr) => {
        {
            let size = std::mem::size_of::<$type>() * 8;
            if $count > size {
                return Err(ReadError::TooManyBits { requested: $count, max: size });
            }

            let bits_left = $buffer.bit_len() - $position;

            if $count > bits_left {
                return Err(ReadError::NotEnoughData { requested: $count, bits_left});
            }

            let mut value: $type = 0;

            let mut i = 0;
            let mut offset = $position;

            while i < $count {
                let remaining = $count - i;
                let bit_offset = offset & 7;
                let byte_index = offset / 8;
                let byte = $buffer.bytes[byte_index];

                // how much can we read from the current byte
                let read = min(remaining, 8 - bit_offset);

                //let mask = if read == 8 {0xFFu8} else {!(0xFFu8 << read)};
                let mask = if read == 8 {0xFFu8} else {!(0xFFu8 << read)};
                let shift =  bit_offset;
                let shifted = byte >> shift;
                let read_bits = shifted & mask;

                value |= read_bits as $type << i;

                offset += read as usize;
                i += read;
            }

            Ok(value)
        }
    }
}


macro_rules! make_signed {
    ($unsigned:expr, $type:ty, $count:expr) => {
        {
            let sign_bits = $unsigned >> ($count - 1) & 1;
            let high_bits = 0 - sign_bits as $type;
            high_bits << $count | $unsigned as $type
        }
    }
}

impl<'a> BitBuffer<'a> {
    /// Construct a new BitBuffer from a byte slice.
    pub fn new(bytes: &'a [u8], order: ByteOrder) -> BitBuffer<'a> {
        BitBuffer {
            bytes,
            order,
            bit_len: bytes.len() * 8,
        }
    }

    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    pub fn read_u8(&self, position: usize, count: usize) -> Result<u8> {
        match self.order {
            ByteOrder::LittleEndian => bitreader_unsigned_le!(self, u8, position, count),
            ByteOrder::BigEndian => bitreader_unsigned_be!(self, u8, position, count)
        }
    }

    pub fn read_u16(&self, position: usize, count: usize) -> Result<u16> {
        bitreader_unsigned_le!(self, u16, position, count)
//        match self.order {
//            ByteOrder::LittleEndian => bitreader_unsigned_le!(self, u16, position, count),
//            ByteOrder::BigEndian => bitreader_unsigned_be!(self, u16, position, count)
//        }
    }

    pub fn read_u32(&self, position: usize, count: usize) -> Result<u32> {
        match self.order {
            ByteOrder::LittleEndian => bitreader_unsigned_le!(self, u32, position, count),
            ByteOrder::BigEndian => bitreader_unsigned_be!(self, u32, position, count)
        }
    }

    pub fn read_i8(&self, position: usize, count: usize) -> Result<i8> {
        let unsigned = self.read_u8(position, count)?;
        Ok(make_signed!(unsigned, i8, count))
    }

    pub fn read_i16(&self, position: usize, count: usize) -> Result<i16> {
        let unsigned = self.read_u16(position, count)?;
        Ok(make_signed!(unsigned, i16, count))
    }

    pub fn read_i32(&self, position: usize, count: usize) -> Result<i32> {
        let unsigned = self.read_u32(position, count)?;
        Ok(make_signed!(unsigned, i32, count))
    }
}