#![allow(dead_code)]
#![allow(unreachable_patterns)]

use bitbuffer_derive::BitRead;

#[derive(BitRead)]
struct TestStruct {
    foo: u8,
    str: String,
}
