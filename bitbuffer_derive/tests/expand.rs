#![allow(dead_code)]
#![allow(unreachable_patterns)]
#![allow(unused_imports)]

use bitbuffer::{BitReadStream, Endianness};
use bitbuffer_derive::{BitWrite, BitWriteSized};

#[derive(BitWrite)]
struct TestSizeExpression {
    size: u8,
    #[size = "size + 2"]
    str: String,
}
