/// Trait for specifying endianness of bit buffer
pub trait Endianness {
    fn is_le() -> bool;
    fn is_be() -> bool;
}

/// Trait for specifying that the bit buffer is big endian
pub struct BigEndian {}

/// Trait for specifying that the bit buffer is little endian
pub struct LittleEndian {}

macro_rules! impl_endianness {
    ($type:ty, $le:expr) => {
        impl Endianness for $type {
            #[inline]
            fn is_le() -> bool {
                $le
            }

            #[inline]
            fn is_be() -> bool {
                !$le
            }
        }
    };
}

impl_endianness!(BigEndian, false);
impl_endianness!(LittleEndian, true);
