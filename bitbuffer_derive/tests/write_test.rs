#![allow(dead_code)]
#![allow(unreachable_patterns)]

use bitbuffer::{
    bit_size_of, bit_size_of_sized, BigEndian, BitReadBuffer, BitReadStream, BitWrite, Endianness,
    LittleEndian,
};

#[derive(BitWrite)]
struct TestStruct {
    foo: u8,
    str: String,
    #[size = 2]
    truncated: String,
    bar: u16,
    float: f32,
    #[size = 3]
    asd: u8,
    #[size_bits = 2]
    dynamic: u8,
    #[size = "asd"]
    previous_field: u8,
}

#[derive(BitWrite)]
#[discriminant_bits = 2]
enum TestBareEnum {
    Foo,
    Bar,
    Asd = 3,
}

#[derive(BitWrite)]
#[discriminant_bits = 2]
enum TestUnnamedFieldEnum {
    #[size = 5]
    Foo(i8),
    Bar(bool),
    #[discriminant = 3]
    Asd(u8),
}
