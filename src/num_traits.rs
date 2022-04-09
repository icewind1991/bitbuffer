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
    type INT: PrimInt
        + BitOrAssign
        + IsSigned
        + UncheckedPrimitiveInt
        + BitXor
        + Debug
        + SplitFitUsize;

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

pub trait SplitFitUsize {
    type Iter: Iterator<Item = (usize, u8)> + ExactSizeIterator + DoubleEndedIterator;

    fn split_fit_usize<E: Endianness>(self) -> Self::Iter;
}

use std::array;
use std::mem::size_of;

macro_rules! impl_split_fit {
    ($type:ty) => {
        impl SplitFitUsize for $type {
            type Iter = array::IntoIter<(usize, u8), 1>;

            fn split_fit_usize<E: Endianness>(self) -> Self::Iter {
                assert!(size_of::<Self>() < size_of::<usize>());
                [(self as usize, size_of::<Self>() as u8 * 8)].into_iter()
            }
        }
    };
}

macro_rules! impl_split_fit_signed {
    ($signed_type:ty, $unsigned_type:ty) => {
        impl SplitFitUsize for $signed_type {
            type Iter = <$unsigned_type as SplitFitUsize>::Iter;

            fn split_fit_usize<E: Endianness>(self) -> Self::Iter {
                let unsigned = <$unsigned_type>::from_ne_bytes(self.to_ne_bytes());
                unsigned.split_fit_usize::<E>()
            }
        }
    };
}

impl_split_fit!(u8);
impl_split_fit!(u16);
impl_split_fit!(i8);
impl_split_fit!(i16);
#[cfg(target_pointer_width = "64")]
impl_split_fit!(u32);

#[cfg(target_pointer_width = "32")]
impl SplitFitUsize for u32 {
    type Iter = array::IntoIter<(usize, u8), 2>;

    fn split_fit_usize<E: Endianness>(self) -> Self::Iter {
        Self::Iter::new(if E::is_le() {
            [
                ((self & (Self::MAX >> 8)) as usize, 24),
                ((self >> 24) as usize, 8),
            ]
        } else {
            [
                ((self >> 24) as usize, 8),
                ((self & (Self::MAX >> 8)) as usize, 24),
            ]
        })
    }
}

impl_split_fit_signed!(i32, u32);

impl SplitFitUsize for u64 {
    type Iter = array::IntoIter<(usize, u8), 3>;

    fn split_fit_usize<E: Endianness>(self) -> Self::Iter {
        (if E::is_le() {
            [
                ((self & (Self::MAX >> 40)) as usize, 24),
                ((self >> 24 & (Self::MAX >> 16)) as usize, 24),
                ((self >> 48) as usize, 16),
            ]
        } else {
            [
                ((self >> 48) as usize, 16),
                ((self >> 24 & (Self::MAX >> 16)) as usize, 24),
                ((self & (Self::MAX >> 40)) as usize, 24),
            ]
        })
        .into_iter()
    }
}

impl_split_fit_signed!(i64, u64);

impl SplitFitUsize for u128 {
    type Iter = array::IntoIter<(usize, u8), 6>;

    fn split_fit_usize<E: Endianness>(self) -> Self::Iter {
        (if E::is_le() {
            [
                ((self & (Self::MAX >> 104)) as usize, 24),
                ((self >> 24 & (Self::MAX >> 80)) as usize, 24),
                ((self >> 48 & (Self::MAX >> 56)) as usize, 24),
                ((self >> 72 & (Self::MAX >> 32)) as usize, 24),
                ((self >> 96 & (Self::MAX >> 8)) as usize, 24),
                ((self >> 120) as usize, 8),
            ]
        } else {
            [
                ((self >> 120) as usize, 8),
                ((self >> 96 & (Self::MAX >> 8)) as usize, 24),
                ((self >> 72 & (Self::MAX >> 32)) as usize, 24),
                ((self >> 48 & (Self::MAX >> 56)) as usize, 24),
                ((self >> 24 & (Self::MAX >> 80)) as usize, 24),
                ((self & (Self::MAX >> 104)) as usize, 24),
            ]
        })
        .into_iter()
    }
}

impl_split_fit_signed!(i128, u128);

impl SplitFitUsize for usize {
    type Iter = array::IntoIter<(usize, u8), 2>;

    fn split_fit_usize<E: Endianness>(self) -> Self::Iter {
        (if E::is_le() {
            [
                (
                    (self & (Self::MAX >> (usize::BITS - 8))) as usize,
                    usize::BITS as u8 - 8,
                ),
                ((self >> (usize::BITS - 8)) as usize, 8),
            ]
        } else {
            [
                ((self >> (usize::BITS - 8)) as usize, 8),
                (
                    (self & (Self::MAX >> (usize::BITS - 8))) as usize,
                    usize::BITS as u8 - 8,
                ),
            ]
        })
        .into_iter()
    }
}

impl_split_fit_signed!(isize, usize);
