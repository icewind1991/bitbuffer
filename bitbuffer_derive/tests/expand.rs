#![allow(dead_code)]
#![allow(unreachable_patterns)]
#![allow(unused_imports)]

use bitbuffer::{BitReadStream, Endianness};
use bitbuffer_derive::{BitRead, BitWrite, BitWriteSized};

#[derive(BitWrite)]
#[discriminant_bits = 4]
enum TestEnumRest {
    Foo,
    Bar,
    #[discriminant = "_"]
    Asd,
}
