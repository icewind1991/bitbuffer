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
}

/// Marks the buffer or stream as big endian
#[derive(Debug)]
pub struct BigEndian;

/// Marks the buffer or stream as little endian
#[derive(Debug)]
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

mod private {
    pub trait Sealed {}

    // Implement for those same types, but no others.
    impl Sealed for super::BigEndian {}

    impl Sealed for super::LittleEndian {}
}
