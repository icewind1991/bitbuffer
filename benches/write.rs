use bitbuffer::{BitWriteStream, LittleEndian};
use iai::black_box;

fn write_int_le() {
    let mut out = Vec::with_capacity(128);
    {
        let mut write = BitWriteStream::new(&mut out, LittleEndian);
        for i in 0..128 {
            write.write_sized(&black_box(i), 7).unwrap();
        }
    }
    black_box(out);
}

iai::main!(write_int_le);
