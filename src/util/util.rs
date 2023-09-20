use std::fs::File;
use std::io::{self, Read};

pub fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn sign_extend<T>(value: T, size: usize) -> usize
where
    T: Into<usize> + std::ops::Shl<usize, Output = T> + std::ops::Shr<usize, Output = T> + Copy,
{
    let shift = std::mem::size_of::<T>() * 8 - size;
    (((value << shift) >> shift).into()) as usize
}
