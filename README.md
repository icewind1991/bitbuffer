[![Crates.io](https://img.shields.io/crates/v/bitstream_reader.svg)](https://crates.io/crates/bitstream_reader)
[![Documentation](https://docs.rs/bitstream_reader/badge.svg)](https://docs.rs/bitstream_reader/)
[![Dependency status](https://deps.rs/repo/github/icewind1991/bitstream_reader/status.svg)](https://deps.rs/repo/github/icewind1991/bitstream_reader)
[![Build Status](https://travis-ci.org/icewind1991/bitstream_reader.svg?branch=master)](https://travis-ci.org/icewind1991/bitstream_reader)

# bitstream_reader

Tools for reading data types of arbitrary bit length and might not be byte-aligned in the source data

The main way of handling with the binary data is to first create a `BitBuffer`
,wrap it into a `BitStream` and then read from the stream.

If performance is critical, working directly on the BitBuffer can be faster.

Once you have a BitStream, there are 2 different approaches of reading data

- read primitives, Strings and byte arrays, using `read_bool`, `read_int`, `read_float`, `read_byes` and `read_string`
- read any type implementing the  `BitRead` or `BitReadSized` traits using `read` and `read_sized`
  - `BitRead` is for types that can be read without requiring any size info (e.g. null-terminal strings, floats, whole integers, etc)
  - `BitReadSized` is for types that require external sizing information to be read (fixed length strings, arbitrary length integers

The `BitRead` and `BitReadSized` traits can be used with `#[derive]` if all fields implement `BitRead` or `BitReadSized`.

## Examples

```rust
use bitstream_reader::{BitBuffer, LittleEndian, BitStream, BitRead};

#[derive(BitRead)]
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
let buffer = BitBuffer::new(bytes, LittleEndian);
let mut stream = BitStream::new(buffer);
let value: u8 = stream.read_int(7)?;
let complex: ComplexType = stream.read()?;
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
