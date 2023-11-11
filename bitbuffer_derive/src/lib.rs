//! Automatically generate `BitRead` and `BitReadSized` implementations for structs and enums
//!
//! # Structs
//!
//! The implementation can be derived for a struct as long as every field in the struct implements `BitRead` or `BitReadSized`
//!
//! The struct is read field by field in the order they are defined in, if the size for a field is set `stream.read_sized()`
//! will be used, otherwise `stream_read()` will be used.
//!
//! The size for a field can be set using 3 different methods
//!  - set the size as an integer using the `size` attribute,
//!  - use a previously defined field as the size using the `size` attribute
//!  - read a set number of bits as an integer, using the resulting value as size using the `read_bits` attribute
//!
//! When deriving `BitReadSized` the input size can be used in the size attribute as the `input_size` field.
//!
//! ## Examples
//!
//! ```
//! use bitbuffer::BitRead;
//!
//! #[derive(BitRead)]
//! struct TestStruct {
//!     foo: u8,
//!     str: String,
//!     #[size = 2] // when `size` is set, the attributed will be read using `read_sized`
//!     truncated: String,
//!     bar: u16,
//!     float: f32,
//!     #[size = 3]
//!     asd: u8,
//!     #[size_bits = 2] // first read 2 bits as unsigned integer, then use the resulting value as size for the read
//!     dynamic_length: u8,
//!     #[size = "asd"] // use a previously defined field as size
//!     previous_field: u8,
//! }
//! ```
//!
//! ```
//! use bitbuffer::BitReadSized;
//!
//! #[derive(BitReadSized, PartialEq, Debug)]
//! struct TestStructSized {
//!     foo: u8,
//!     #[size = "input_size"]
//!     string: String,
//!     #[size = "input_size"]
//!     int: u8,
//! }
//! ```
//!
//! # Enums
//!
//! The implementation can be derived for an enum as long as every variant of the enum either has no field, or an unnamed field that implements `BitRead` or `BitReadSized`
//!
//! The enum is read by first reading a set number of bits as the discriminant of the enum, then the variant for the read discriminant is read.
//!
//! For details about setting the input size for fields implementing `BitReadSized` see the block about size in the `Structs` section above.
//!
//! The discriminant for the variants defaults to incrementing by one for every field, starting with `0`.
//! You can overwrite the discriminant for a field, which will also change the discriminant for every following field.
//!
//! ## Examples
//!
//! ```
//! # use bitbuffer::BitRead;
//! #
//! #[derive(BitRead)]
//! #[discriminant_bits = 2]
//! enum TestBareEnum {
//!     Foo,
//!     Bar,
//!     Asd = 3, // manually set the discriminant value for a field
//! }
//! ```
//!
//! ```
//! # use bitbuffer::BitRead;
//! #
//! #[derive(BitRead)]
//! #[discriminant_bits = 2]
//! enum TestUnnamedFieldEnum {
//!     #[size = 5]
//!     Foo(i8),
//!     Bar(bool),
//!     #[discriminant = 3] // since rust only allows setting the discriminant on field-less enums, you can use an attribute instead
//!     Asd(u8),
//! }
//! ```
//!
//! ```
//! # use bitbuffer::BitReadSized;
//! #
//! #[derive(BitReadSized, PartialEq, Debug)]
//! #[discriminant_bits = 2]
//! enum TestUnnamedFieldEnumSized {
//!     #[size = 5]
//!     Foo(i8),
//!     Bar(bool),
//!     #[discriminant = 3]
//!     #[size = "input_size"]
//!     Asd(u8),
//! }
//! ```
//!
//! # Alignment
//!
//! You can request alignment for a struct, enum or a field using #[align] attribute.
//!
//! ```
//! # use bitbuffer::BitRead;
//! #
//! #[derive(BitRead)]
//! #[align] // align the reader before starting to read the struct
//! struct TestAlignStruct {
//!    #[size = 1]
//!    foo: u8,
//!    #[align] // align the reader before reading the field
//!    bar: u8,
//! }
//! ```
//!
//! It can also be applied to non-unit enum variants:
//!
//! ```
//! # use bitbuffer::BitRead;
//! #
//! #[derive(BitRead)]
//! #[align] // align the reader before starting to read the enum
//! #[discriminant_bits = 2]
//! enum TestAlignEnum {
//!     Foo(u8),
//!     #[align] // align the reader before reading the variant (but after reading the discriminant)
//!     Bar(u8),
//! }
//! ```
//!
//! # Endianness
//!
//! If the struct that `BitRead` or `BitReadSized` is derived for requires a Endianness type parameter, you need to tell the derive macro the name of the type parameter used
//!
//! ```
//! # use bitbuffer::{BitRead, Endianness, BitReadStream};
//! #
//! #[derive(BitRead)]
//! #[endianness = "E"]
//! struct EndiannessStruct<'a, E: Endianness> {
//!     size: u8,
//!     #[size = "size"]
//!     stream: BitReadStream<'a, E>,
//! }
//! ```
//!
//! This is also required if you specify which endianness the struct has
//! ```
//! # use bitbuffer::{BitRead, BigEndian, BitReadStream};
//! #
//! #[derive(BitRead)]
//! #[endianness = "BigEndian"]
//! struct EndiannessStruct<'a> {
//!     size: u8,
//!     #[size = "size"]
//!     stream: BitReadStream<'a, BigEndian>,
//! }
//! ```
//!
mod discriminant;
mod params;
mod read;
mod size_hint;
mod write;

extern crate proc_macro;

use crate::read::{Read, ReadSized};
use crate::write::{Write, WriteSized};
use proc_macro2::{Span, TokenStream};
use std::fmt::Display;
use syn::{parse_macro_input, DeriveInput, Error, Result};

/// See the [crate documentation](index.html) for details
#[proc_macro_derive(
    BitRead,
    attributes(
        bitbuffer,
        size,
        size_bits,
        discriminant_bits,
        discriminant,
        endianness,
        align
    )
)]
pub fn derive_bitread(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_trait::<Read>(input)
}

/// See the [crate documentation](index.html) for details
#[proc_macro_derive(
    BitReadSized,
    attributes(
        bitbuffer,
        size,
        size_bits,
        discriminant_bits,
        discriminant,
        endianness,
        align
    )
)]
pub fn derive_bitread_sized(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_trait::<ReadSized>(input)
}

/// See the [crate documentation](index.html) for details
#[proc_macro_derive(
    BitWrite,
    attributes(
        bitbuffer,
        size,
        size_bits,
        discriminant_bits,
        discriminant,
        endianness,
        align
    )
)]
pub fn derive_bitwrite(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_trait::<Write>(input)
}

/// See the [crate documentation](index.html) for details
#[proc_macro_derive(
    BitWriteSized,
    attributes(
        bitbuffer,
        size,
        size_bits,
        discriminant_bits,
        discriminant,
        endianness,
        align
    )
)]
pub fn derive_bitwrite_sized(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_trait::<WriteSized>(input)
}

/// Basic wrapper for error handling
fn derive_trait<Trait: Derivable>(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    derive_trait_inner::<Trait>(input)
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

fn derive_trait_inner<Trait: Derivable>(input: DeriveInput) -> Result<TokenStream> {
    let params = Trait::Params::parse(&input)?;
    Trait::derive(params)
}

trait Derivable {
    type Params: DeriveParams;

    fn derive(params: Self::Params) -> Result<TokenStream>;
}

trait DeriveParams: Sized {
    fn parse(input: &DeriveInput) -> Result<Self>;
}

fn err<R, Msg: Display>(msg: Msg, span: Span) -> Result<R> {
    Err(Error::new(span, msg))
}
