use crate::Endianness;
use num_traits::PrimInt;
use std::array::TryFromSliceError;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::ops::{BitOrAssign, BitXor};

/// some extra number traits

/// Allow casting floats unchecked
pub trait UncheckedPrimitiveFloat: Sized {
    type BYTES: AsRef<[u8]> + for<'a> TryFrom<&'a [u8], Error = TryFromSliceError>;
    type INT: PrimInt + BitOrAssign + IsSigned + UncheckedPrimitiveInt + BitXor + Debug + IntoBytes;

    fn from_f32_unchecked(n: f32) -> Self;
    fn from_f64_unchecked(n: f64) -> Self;
    fn to_bytes<E: Endianness>(self) -> Self::BYTES;
    fn from_bytes<E: Endianness>(bytes: Self::BYTES) -> Self;
    fn to_int(self) -> Self::INT;
    fn from_int(int: Self::INT) -> Self;
}

impl UncheckedPrimitiveFloat for f32 {
    type BYTES = [u8; 4];
    type INT = u32;

    #[inline(always)]
    fn from_f32_unchecked(n: f32) -> Self {
        n
    }
    #[inline(always)]
    fn from_f64_unchecked(n: f64) -> Self {
        n as f32
    }
    fn to_bytes<E: Endianness>(self) -> Self::BYTES {
        if E::is_le() {
            self.to_le_bytes()
        } else {
            self.to_be_bytes()
        }
    }
    fn from_bytes<E: Endianness>(bytes: Self::BYTES) -> Self {
        if E::is_le() {
            Self::from_le_bytes(bytes)
        } else {
            Self::from_be_bytes(bytes)
        }
    }
    fn to_int(self) -> Self::INT {
        Self::INT::from_le_bytes(self.to_le_bytes())
    }
    fn from_int(int: Self::INT) -> Self {
        Self::from_le_bytes(int.to_le_bytes())
    }
}

impl UncheckedPrimitiveFloat for f64 {
    type BYTES = [u8; 8];
    type INT = u64;

    #[inline(always)]
    fn from_f32_unchecked(n: f32) -> Self {
        n as f64
    }
    #[inline(always)]
    fn from_f64_unchecked(n: f64) -> Self {
        n
    }
    fn to_bytes<E: Endianness>(self) -> Self::BYTES {
        if E::is_le() {
            self.to_le_bytes()
        } else {
            self.to_be_bytes()
        }
    }
    fn from_bytes<E: Endianness>(bytes: Self::BYTES) -> Self {
        if E::is_le() {
            Self::from_le_bytes(bytes)
        } else {
            Self::from_be_bytes(bytes)
        }
    }
    fn to_int(self) -> Self::INT {
        Self::INT::from_le_bytes(self.to_le_bytes())
    }
    fn from_int(int: Self::INT) -> Self {
        Self::from_le_bytes(int.to_le_bytes())
    }
}

/// Allow casting integers unchecked
pub trait UncheckedPrimitiveInt: Sized {
    fn from_u8_unchecked(n: u8) -> Self;
    fn from_i8_unchecked(n: i8) -> Self;
    fn from_u16_unchecked(n: u16) -> Self;
    fn from_i16_unchecked(n: i16) -> Self;
    fn from_u32_unchecked(n: u32) -> Self;
    fn from_i32_unchecked(n: i32) -> Self;
    fn from_u64_unchecked(n: u64) -> Self;
    fn from_i64_unchecked(n: i64) -> Self;
    fn from_u128_unchecked(n: u128) -> Self;
    fn from_i128_unchecked(n: i128) -> Self;
    fn from_usize_unchecked(n: usize) -> Self;
    fn from_isize_unchecked(n: isize) -> Self;

    fn into_u8_unchecked(self) -> u8;
    fn into_i8_unchecked(self) -> i8;
    fn into_u16_unchecked(self) -> u16;
    fn into_i16_unchecked(self) -> i16;
    fn into_u32_unchecked(self) -> u32;
    fn into_i32_unchecked(self) -> i32;
    fn into_u64_unchecked(self) -> u64;
    fn into_i64_unchecked(self) -> i64;
    fn into_u128_unchecked(self) -> u128;
    fn into_i128_unchecked(self) -> i128;
    fn into_usize_unchecked(self) -> usize;
    fn into_isize_unchecked(self) -> isize;

    fn from_unchecked<N: UncheckedPrimitiveInt>(n: N) -> Self;
}

macro_rules! impl_unchecked_int {
    ($type:ty, $conv:ident) => {
        impl UncheckedPrimitiveInt for $type {
            #[inline(always)]
            fn from_u8_unchecked(n: u8) -> Self {
                n as $type
            }
            #[inline(always)]
            fn from_i8_unchecked(n: i8) -> Self {
                n as $type
            }
            #[inline(always)]
            fn from_u16_unchecked(n: u16) -> Self {
                n as $type
            }
            #[inline(always)]
            fn from_i16_unchecked(n: i16) -> Self {
                n as $type
            }
            #[inline(always)]
            fn from_u32_unchecked(n: u32) -> Self {
                n as $type
            }
            #[inline(always)]
            fn from_i32_unchecked(n: i32) -> Self {
                n as $type
            }
            #[inline(always)]
            fn from_u64_unchecked(n: u64) -> Self {
                n as $type
            }
            #[inline(always)]
            fn from_i64_unchecked(n: i64) -> Self {
                n as $type
            }
            #[inline(always)]
            fn from_u128_unchecked(n: u128) -> Self {
                n as $type
            }
            #[inline(always)]
            fn from_i128_unchecked(n: i128) -> Self {
                n as $type
            }
            #[inline(always)]
            fn from_usize_unchecked(n: usize) -> Self {
                n as $type
            }
            #[inline(always)]
            fn from_isize_unchecked(n: isize) -> Self {
                n as $type
            }

            fn into_u8_unchecked(self) -> u8 {
                self as u8
            }
            #[inline(always)]
            fn into_i8_unchecked(self) -> i8 {
                self as i8
            }
            #[inline(always)]
            fn into_u16_unchecked(self) -> u16 {
                self as u16
            }
            #[inline(always)]
            fn into_i16_unchecked(self) -> i16 {
                self as i16
            }
            #[inline(always)]
            fn into_u32_unchecked(self) -> u32 {
                self as u32
            }
            #[inline(always)]
            fn into_i32_unchecked(self) -> i32 {
                self as i32
            }
            #[inline(always)]
            fn into_u64_unchecked(self) -> u64 {
                self as u64
            }
            #[inline(always)]
            fn into_i64_unchecked(self) -> i64 {
                self as i64
            }
            #[inline(always)]
            fn into_u128_unchecked(self) -> u128 {
                self as u128
            }
            #[inline(always)]
            fn into_i128_unchecked(self) -> i128 {
                self as i128
            }
            #[inline(always)]
            fn into_usize_unchecked(self) -> usize {
                self as usize
            }
            #[inline(always)]
            fn into_isize_unchecked(self) -> isize {
                self as isize
            }

            #[inline(always)]
            fn from_unchecked<N: UncheckedPrimitiveInt>(n: N) -> Self {
                n.$conv()
            }
        }
    };
}

impl_unchecked_int!(u8, into_u8_unchecked);
impl_unchecked_int!(i8, into_i8_unchecked);
impl_unchecked_int!(u16, into_u16_unchecked);
impl_unchecked_int!(i16, into_i16_unchecked);
impl_unchecked_int!(u32, into_u32_unchecked);
impl_unchecked_int!(i32, into_i32_unchecked);
impl_unchecked_int!(u64, into_u64_unchecked);
impl_unchecked_int!(i64, into_i64_unchecked);
impl_unchecked_int!(u128, into_u128_unchecked);
impl_unchecked_int!(i128, into_i128_unchecked);
impl_unchecked_int!(usize, into_usize_unchecked);
impl_unchecked_int!(isize, into_isize_unchecked);

pub trait IsSigned {
    fn is_signed() -> bool;
}

macro_rules! impl_is_signed {
    ($type:ty, $signed:expr) => {
        impl IsSigned for $type {
            #[inline(always)]
            fn is_signed() -> bool {
                $signed
            }
        }
    };
}

pub trait IntoBytes: Sized {
    type BytesIter: DoubleEndedIterator<Item = u8> + ExactSizeIterator;
    type U16Iter: DoubleEndedIterator<Item = u16> + ExactSizeIterator;

    fn into_bytes(self) -> Self::BytesIter;

    fn into_u16(self) -> Self::U16Iter;
}

macro_rules! impl_into_bytes {
    ($type:ty, $bytes:expr, 1 ) => {
        impl IntoBytes for $type {
            type BytesIter = std::array::IntoIter<u8, $bytes>;
            type U16Iter = std::array::IntoIter<u16, 1>;

            #[inline(always)]
            fn into_bytes(self) -> Self::BytesIter {
                Self::BytesIter::new(self.to_le_bytes())
            }

            #[inline(always)]
            fn into_u16(self) -> Self::U16Iter {
                Self::U16Iter::new([self as u16])
            }
        }
    };
    ($type:ty, $bytes:expr, $shorts:expr ) => {
        impl IntoBytes for $type {
            type BytesIter = std::array::IntoIter<u8, $bytes>;
            type U16Iter = std::array::IntoIter<u16, { $shorts }>;

            #[inline(always)]
            fn into_bytes(self) -> Self::BytesIter {
                Self::BytesIter::new(self.to_le_bytes())
            }

            #[inline(always)]
            fn into_u16(self) -> Self::U16Iter {
                use std::convert::TryInto;
                use std::mem::align_of;

                let bytes = self.to_le_bytes();
                if align_of::<Self>() >= align_of::<u16>() {
                    let (head, aligned, tail) = unsafe { bytes[..].align_to::<u16>() };
                    debug_assert_eq!(0, head.len());
                    debug_assert_eq!(0, tail.len());
                    Self::U16Iter::new(aligned.try_into().unwrap())
                } else {
                    let mut shorts = [0; $shorts];
                    let mut chunks = bytes.chunks(2).zip(shorts.iter_mut());
                    while let Some((&[a, b], short)) = chunks.next() {
                        *short = (b as u16) << 8 | a as u16;
                    }
                    Self::U16Iter::new(shorts)
                }
            }
        }
    };
}

impl_is_signed!(u8, false);
impl_is_signed!(u16, false);
impl_is_signed!(u32, false);
impl_is_signed!(u64, false);
impl_is_signed!(u128, false);
impl_is_signed!(usize, false);
impl_is_signed!(i8, true);
impl_is_signed!(i16, true);
impl_is_signed!(i32, true);
impl_is_signed!(i64, true);
impl_is_signed!(i128, true);
impl_is_signed!(isize, true);

impl_into_bytes!(u8, 1, 1);
impl_into_bytes!(u16, 2, 1);
impl_into_bytes!(u32, 4, 2);
impl_into_bytes!(u64, 8, 4);
impl_into_bytes!(u128, 16, 8);

#[cfg(target_pointer_width = "64")]
impl_into_bytes!(usize, 8, 4);
#[cfg(target_pointer_width = "32")]
impl_into_bytes!(usize, 4, 2);

impl_into_bytes!(i8, 1, 1);
impl_into_bytes!(i16, 2, 1);
impl_into_bytes!(i32, 4, 2);
impl_into_bytes!(i64, 8, 4);
impl_into_bytes!(i128, 16, 8);

#[cfg(target_pointer_width = "64")]
impl_into_bytes!(isize, 8, 4);
#[cfg(target_pointer_width = "32")]
impl_into_bytes!(isize, 4, 2);
