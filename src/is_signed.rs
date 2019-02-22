pub trait IsSigned {
    fn is_signed() -> bool;
}

macro_rules! impl_is_signed {
    ($type:ty, $signed:expr) => {
        impl IsSigned for $type {
            #[inline]
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
impl_is_signed!(usize, false);
impl_is_signed!(i8, true);
impl_is_signed!(i16, true);
impl_is_signed!(i32, true);
impl_is_signed!(i64, true);
impl_is_signed!(isize, true);
