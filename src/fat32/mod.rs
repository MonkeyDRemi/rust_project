use core::fmt;
use alloc::string::String;
use alloc::vec::Vec;
use core::mem::size_of;

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
    info: FsInfo,
}

impl<D: Disk> Fat32<D> {
    pub fn mount(disk: D) -> Result<Self, Error> {
	if disk.sector_count() == 0 {
            return Err(Error::IoError);
        }

        let mut buffer = [0u8; SECTOR_SIZE];
        disk.read_sector(0, &mut buffer)?;
        
        let boot_sector = unsafe { cast_slice_to_struct::<BootSector>(&buffer) };
        
        if boot_sector.boot_signature != 0xAA55 {
            return Err(Error::InvalidFat32Structure);
        }

        let bpb = &boot_sector.bpb;
        
        let bytes_per_sector = bpb.bytes_per_sector as u32;
        if bytes_per_sector != 512 {
             return Err(Error::InvalidFat32Structure);
        }

        let reserved_sector_count = bpb.reserved_sector_count as u32;
        let num_fats = bpb.num_fats as u32;
        let fat_size = bpb.fat_size_32;
        let root_cluster = bpb.root_cluster;

        let first_fat_sector = reserved_sector_count;
        let fat_sectors = num_fats * fat_size;
        let first_data_sector = reserved_sector_count + fat_sectors;

        let total_sectors = bpb.total_sectors_32;
        let data_sectors = total_sectors - first_data_sector;
        let cluster_count = data_sectors / (bpb.sectors_per_cluster as u32);


        let fs_info = FsInfo {
            bytes_per_sector,
            sectors_per_cluster: bpb.sectors_per_cluster as u32,
            reserved_sector_count,
            num_fats,
            fat_size,
            root_cluster,
            first_fat_sector,
            first_data_sector,
            cluster_count,
        };


        Ok(Fat32 { 
	    disk,
	    info: fs_info, 
	})
    }

	fn cluster_to_lba(&self, cluster: u32) -> u32 {
            let cluster_offset = cluster - 2;
            self.info.first_data_sector + (cluster_offset * self.info.sectors_per_cluster)
    	}	

    	fn get_fat_entry(&self, cluster: u32) -> Result<u32, Error> {
            if cluster < 2 || cluster >= self.info.cluster_count + 2 {
                return Err(Error::InvalidFat32Structure);
        }
        
        let fat_entry_offset = cluster * 4; 
        let fat_sector_num = self.info.first_fat_sector + (fat_entry_offset / self.info.bytes_per_sector);
        let fat_entry_in_sector = fat_entry_offset % self.info.bytes_per_sector;
        let mut buffer = [0u8; SECTOR_SIZE];
        self.disk.read_sector(fat_sector_num, &mut buffer)?; 

        let entry_bytes: [u8; 4] = [
            buffer[fat_entry_in_sector as usize],
            buffer[(fat_entry_in_sector + 1) as usize],
            buffer[(fat_entry_in_sector + 2) as usize],
            buffer[(fat_entry_in_sector + 3) as usize],
        ];

        let entry = u32::from_le_bytes(entry_bytes);
        
        Ok(entry & 0x0FFFFFFF)
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


pub struct FsInfo {
    pub bytes_per_sector: u32,
    pub sectors_per_cluster: u32,
    pub reserved_sector_count: u32,
    pub num_fats: u32,
    pub fat_size: u32,
    pub root_cluster: u32,
    pub first_fat_sector: u32,
    pub first_data_sector: u32,
    pub cluster_count: u32,
}

/// # Safety
/// Le slice d'entrée doit être suffisamment grand pour contenir la structure T (`slice.len() >= size_of::<T>()`)
/// L'alignement de la structure T doit être valide dans le contexte `#[repr(packed)]` utilisé
/// La séquence d'octets dans le slice doit représenter une valeur valide pour la structure T

unsafe fn cast_slice_to_struct<T>(slice: &[u8]) -> &T {
    &*(slice.as_ptr() as *const T)
}
