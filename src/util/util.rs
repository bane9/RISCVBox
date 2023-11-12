use std::fs::File;
use std::io::{self, Read};

pub fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn sign_extend<T>(value: T, size: usize) -> i64
where
    T: Into<i64>,
{
    let value = value.into();
    let sign_bit = 1 << (size - 1);
    if (value & sign_bit) != 0 {
        let sign_extend_mask = (u64::max_value() & !((1 << size) - 1)) as i64;
        return value | sign_extend_mask;
    } else {
        return value;
    };
}
