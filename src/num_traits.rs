/// some extra number traits

/// Allow casting floats unchecked
pub trait UncheckedPrimitiveFloat: Sized {
    fn from_f32_unchecked(n: f32) -> Self;
    fn from_f64_unchecked(n: f64) -> Self;
}

impl UncheckedPrimitiveFloat for f32 {
    #[inline(always)]
    fn from_f32_unchecked(n: f32) -> Self {
        n
    }
    #[inline(always)]
    fn from_f64_unchecked(n: f64) -> Self {
        n as f32
    }
}

impl UncheckedPrimitiveFloat for f64 {
    #[inline(always)]
    fn from_f32_unchecked(n: f32) -> Self {
        n as f64
    }
    #[inline(always)]
    fn from_f64_unchecked(n: f64) -> Self {
        n
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
    fn into_bytes(self) -> Vec<u8>;
}

macro_rules! impl_into_bytes {
    ($type:ty) => {
        impl IntoBytes for $type {
            #[inline(always)]
            fn into_bytes(self) -> Vec<u8> {
                self.to_le_bytes().to_vec()
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

impl_into_bytes!(u8);
impl_into_bytes!(u16);
impl_into_bytes!(u32);
impl_into_bytes!(u64);
impl_into_bytes!(u128);
impl_into_bytes!(usize);
impl_into_bytes!(i8);
impl_into_bytes!(i16);
impl_into_bytes!(i32);
impl_into_bytes!(i64);
impl_into_bytes!(i128);
impl_into_bytes!(isize);
