use lazy_static::lazy_static;
use std::fs::File;
use std::io::{self, Read};
use std::time::SystemTime;

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
    let value = value.into() as usize;
    let sign_bit = 1 << (size - 1);
    if value & sign_bit != 0 {
        (!((1 << size) - 1) | value) as i64
    } else {
        value as i64
    }
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

lazy_static! {
    static ref START_TIME: SystemTime = SystemTime::now();
}

pub fn ms_since_program_start() -> u64 {
    let _ = *START_TIME;
    let now = SystemTime::now();
    let duration_since_start = now
        .duration_since(*START_TIME)
        .expect("Time went backwards");
    duration_since_start.as_millis() as u64
}
