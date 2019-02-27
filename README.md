# bitstream_reader

Reading bit sequences from a byte slice in rust

## Example

```rust
use bitstream_reader::{BitBuffer, LittleEndian};

let bytes: &[u8] = &[
    0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
];
let buffer: BitBuffer<LittleEndian> = BitBuffer::new(bytes);
let result = buffer.read::<u16>(10, 9).unwrap();
```

You can read up to a maximum of 64 bit.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.