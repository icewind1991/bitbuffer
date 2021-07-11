#![allow(dead_code)]
#![allow(unreachable_patterns)]
#![allow(unused_imports)]

use bitbuffer::{BitReadStream, Endianness};
use bitbuffer_derive::{BitWrite, BitWriteSized};

#[derive(BitWrite)]
struct TestStruct {
    foo: u8,
    str: String,
}

#[derive(BitWrite)]
struct UnnamedSize(u8, #[size = 5] String, bool);
