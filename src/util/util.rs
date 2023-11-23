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

pub fn read_bit<T>(value: T, bit: usize) -> bool
where
    T: Into<u64>,
{
    let value = value.into();
    return (value & (1 << bit)) != 0;
}

pub fn write_bit<T>(value: T, bit: usize, bit_value: bool) -> T
where
    T: Into<usize> + From<usize>,
{
    let value: usize = value.into();
    let mask = 1 << bit;
    if bit_value {
        return T::from(value | mask);
    } else {
        return T::from(value & !mask);
    }
}

pub fn read_bits<T>(value: T, start: usize, end: usize) -> u64
where
    T: Into<u64>,
{
    let value = value.into();
    let mask = (1 << (end - start + 1)) - 1;
    return (value >> start) & mask;
}

pub fn write_bits<T>(value: T, start: usize, end: usize, bits: usize) -> T
where
    T: Into<usize> + From<usize>,
{
    let value: usize = value.into();
    let mask = (1 << (end - start + 1)) - 1;
    return T::from((value & !(mask << start)) | ((bits & mask) << start));
}

pub fn align_up(value: usize, align: usize) -> usize {
    return (value + align - 1) & !(align - 1);
}

pub fn align_down(value: usize, align: usize) -> usize {
    return value & !(align - 1);
}

pub const fn size_kib(size: usize) -> usize {
    return size * 1024;
}

pub const fn size_mib(size: usize) -> usize {
    return size * 1024 * 1024;
}
