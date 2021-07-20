/// Trait for specifying endianness of bit buffer
pub trait Endianness: private::Sealed {
    /// Get the endianness as string, either LittleEndian or BigEndian
    fn as_string() -> &'static str {
        if Self::is_le() {
            "LittleEndian"
        } else {
            "BigEndian"
        }
    }

    /// Input is little endian
    fn is_le() -> bool;
    /// Input is big endian
    fn is_be() -> bool;
    /// Get an instance of the endianness
    fn endianness() -> Self;
}

/// Marks the buffer or stream as big endian
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct BigEndian;

/// Marks the buffer or stream as little endian
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct LittleEndian;

macro_rules! impl_endianness {
    ($type:ty, $le:expr, $instance:expr) => {
        impl Endianness for $type {
            #[inline(always)]
            fn is_le() -> bool {
                $le
            }

            #[inline(always)]
            fn is_be() -> bool {
                !$le
            }

            fn endianness() -> Self {
                $instance
            }
        }
    };
}

impl_endianness!(BigEndian, false, BigEndian);
impl_endianness!(LittleEndian, true, LittleEndian);

mod private {
    pub trait Sealed {}

    // Implement for those same types, but no others.
    impl Sealed for super::BigEndian {}

    impl Sealed for super::LittleEndian {}
}
