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


#[repr(packed)]
#[allow(dead_code)]
pub struct BiosParameterBlock {
    pub bytes_per_sector: u16,        
    pub sectors_per_cluster: u8,       
    pub reserved_sector_count: u16,  
    pub num_fats: u8,                
    pub root_entry_count: u16,        
    pub total_sectors_16: u16,        
    pub media_descriptor: u8,          
    pub fat_size_16: u16,             
    pub sectors_per_track: u16,     
    pub num_heads: u16,             
    pub hidden_sectors: u32,        
    pub total_sectors_32: u32,     

    pub fat_size_32: u32,              
    pub ext_flags: u16,              
    pub fs_version: u16,          
    pub root_cluster: u32,            
    pub fs_info_sector: u16,           
    pub backup_boot_sector: u16,       
    pub reserved: [u8; 12],           
    pub drive_num: u8,                 
    pub reserved_1: u8,                
    pub boot_signature: u8,           
    pub volume_id: u32,                
    pub volume_label: [u8; 11],        
    pub fs_type: [u8; 8],              
}

#[repr(packed)]
#[allow(dead_code)]
pub struct BootSector {
    pub jmp_boot: [u8; 3],     
    pub oem_name: [u8; 8],          
    pub bpb: BiosParameterBlock,    
    _padding: [u8; 420],            
    pub boot_signature: u16,        
}
