#![allow(dead_code)]
#![allow(unreachable_patterns)]
#![allow(unused_imports)]

use bitbuffer::{BitReadStream, Endianness};
use bitbuffer_derive::{BitRead, BitReadSized, BitWrite, BitWriteSized};

#[derive(BitWrite, PartialEq, Debug)]
#[align]
struct AlignStruct(u8);
