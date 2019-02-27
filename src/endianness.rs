/// Trait for specifying endianness of bit buffer
pub trait Endianness {
    /// Input is little endian
    fn is_le() -> bool;
    /// Input is big endian
    fn is_be() -> bool;
}

/// Marks the buffer or stream as big endian
pub struct BigEndian;

/// Marks the buffer or stream as little endian
pub struct LittleEndian;

macro_rules! impl_endianness {
    ($type:ty, $le:expr) => {
        impl Endianness for $type {
            #[inline(always)]
            fn is_le() -> bool {
                $le
            }

            #[inline(always)]
            fn is_be() -> bool {
                !$le
            }
        }
    };
}

impl_endianness!(BigEndian, false);
impl_endianness!(LittleEndian, true);
