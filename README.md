[![Crates.io](https://img.shields.io/crates/v/bitbuffer.svg)](https://crates.io/crates/bitbuffer)
[![Documentation](https://docs.rs/bitbuffer/badge.svg)](https://docs.rs/bitbuffer/)
[![Dependency status](https://deps.rs/repo/github/icewind1991/bitbuffer/status.svg)](https://deps.rs/repo/github/icewind1991/bitbuffer)

# bitbuffer

Tools for reading and writing data types of arbitrary bit length and might not be byte-aligned in the source data

The main way of reading the binary data is to first create a `BitReadBuffer` ,wrap it into a `BitReadStream` and then read from the stream.

Once you have a BitStream, there are 2 different approaches of reading data

 - read primitives, Strings and byte arrays, using `read_bool`, `read_int`, `read_float`, `read_bytes` and `read_string`
 - read any type implementing the  `BitRead` or `BitReadSized` traits using `read` and `read_sized`
   - `BitRead` is for types that can be read without requiring any size info (e.g. null-terminal strings, floats, whole integers, etc)
   - `BitReadSized` is for types that require external sizing information to be read (fixed length strings, arbitrary length integers

The `BitRead` and `BitReadSized` traits can be used with `#[derive]` if all fields implement `BitRead` or `BitReadSized`.

For writing the data you wrap the output `Vec` into a `BitWriteStream` which can then be used in a manner similar to the `BitReadStream`

 - write primitives, Strings and byte arrays, using `write_bool`, `write_int`, `write_float`, `write_bytes` and `write_string`
 - write any type implementing the  `BitWrite` or `BitWriteSized` traits using `write` and `write_sized`
   - `BitWrite` is for types that can be written without requiring any size info (e.g. null-terminal strings, floats, whole integers, etc)
   - `BitWriteSized` is for types that require external sizing information to be written (fixed length strings, arbitrary length integers

Just like the read counterparts, `BitWrite` and `BitWriteSized` traits can be used with `#[derive]` if all fields implement `BitWrite` or `BitWriteSized`.

# Examples

```
use bitbuffer::{BitReadBuffer, LittleEndian, BitReadStream, BitRead, BitWrite, BitWriteStream};

#[derive(BitRead, BitWrite)]
struct ComplexType {
    first: u8,
    #[size = 15]
    second: u16,
    third: bool,
}

let bytes = vec![
    0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
];
let buffer = BitReadBuffer::new(&bytes, LittleEndian);
let mut stream = BitReadStream::new(buffer);
let value: u8 = stream.read_int(7)?;
let complex: ComplexType = stream.read()?;

let mut write_bytes = vec![];
let mut write_stream = BitWriteStream::new(&mut write_bytes, LittleEndian);
write_stream.write_int(12, 7)?;
write_stream.write(&ComplexType {
    first: 55,
    second: 12,
    third: true
})?;
 ```

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
