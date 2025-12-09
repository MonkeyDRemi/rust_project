use core::fmt;
use alloc::string::String;
use alloc::vec::Vec;

pub const SECTOR_SIZE: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    IoError,
    InvalidFat32Structure,
    FileNotFound,
    InvalidPath,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait Disk {
    fn read_sector(&self, sector_lba: u32, buffer: &mut [u8]) -> Result<(), Error>;
    fn write_sector(&mut self, sector_lba: u32, buffer: &[u8]) -> Result<(), Error>;
    fn sector_count(&self) -> u32;
}

pub struct Fat32<D: Disk> {
    disk: D,
}

impl<D: Disk> Fat32<D> {
    pub fn mount(disk: D) -> Result<Self, Error> {
        Ok(Fat32 { disk })
    }
}
