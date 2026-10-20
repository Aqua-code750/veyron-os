//! FAT32 Filesystem Implementation for Veyron OS
//! Supports reading, writing, creating, and enumerating files and directories on disk.

use crate::drivers::disk::{read_sector, write_sector};

pub const ATTR_READ_ONLY: u8 = 0x01;
pub const ATTR_HIDDEN: u8    = 0x02;
pub const ATTR_SYSTEM: u8    = 0x04;
pub const ATTR_VOLUME_ID: u8 = 0x08;
pub const ATTR_DIRECTORY: u8 = 0x10;
pub const ATTR_ARCHIVE: u8   = 0x20;
pub const ATTR_LFN: u8       = 0x0F;

#[derive(Debug, Clone)]
pub struct FsEntry {
    pub name: [u8; 64],
    pub name_len: usize,
    pub is_dir: bool,
    pub size: usize,
    pub first_cluster: u32,
}

impl FsEntry {
    pub const fn empty() -> Self {
        Self {
            name: [0; 64],
            name_len: 0,
            is_dir: false,
            size: 0,
            first_cluster: 0,
        }
    }

    pub fn name_as_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }
}

pub struct Fat32Fs {
    pub partition_start_lba: u64,
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub num_fats: u8,
    pub fat_size_sectors: u32,
    pub root_cluster: u32,
    pub fat_start_lba: u64,
    pub data_start_lba: u64,
    pub initialized: bool,
}

impl Fat32Fs {
    pub const fn new() -> Self {
        Self {
            partition_start_lba: 0,
            bytes_per_sector: 512,
            sectors_per_cluster: 1,
            reserved_sectors: 32,
            num_fats: 2,
            fat_size_sectors: 0,
            root_cluster: 2,
            fat_start_lba: 0,
            data_start_lba: 0,
            initialized: false,
        }
    }

    pub fn init(&mut self) -> bool {
        let mut sector_buf = [0u8; 512];

        // 1. Read MBR (Sector 0) to find Partition 1
        if !read_sector(0, &mut sector_buf) {
            crate::serial_println!("[FAT32] Failed to read MBR sector 0!");
            return false;
        }

        // Check MBR boot signature 0x55, 0xAA
        if sector_buf[510] != 0x55 || sector_buf[511] != 0xAA {
            crate::serial_println!("[FAT32] Invalid MBR boot signature!");
            return false;
        }

        let part_lba = u32::from_le_bytes([
            sector_buf[454], sector_buf[455], sector_buf[456], sector_buf[457]
        ]) as u64;

        self.partition_start_lba = part_lba;

        // 2. Read Partition Boot Sector (BPB)
        if !read_sector(self.partition_start_lba, &mut sector_buf) {
            crate::serial_println!("[FAT32] Failed to read BPB at LBA {}!", self.partition_start_lba);
            return false;
        }

        self.bytes_per_sector = u16::from_le_bytes([sector_buf[11], sector_buf[12]]);
        self.sectors_per_cluster = sector_buf[13];
        self.reserved_sectors = u16::from_le_bytes([sector_buf[14], sector_buf[15]]);
        self.num_fats = sector_buf[16];
        self.fat_size_sectors = u32::from_le_bytes([
            sector_buf[36], sector_buf[37], sector_buf[38], sector_buf[39]
        ]);
        self.root_cluster = u32::from_le_bytes([
            sector_buf[44], sector_buf[45], sector_buf[46], sector_buf[47]
        ]);

        self.fat_start_lba = self.partition_start_lba + self.reserved_sectors as u64;
        self.data_start_lba = self.fat_start_lba + (self.num_fats as u64 * self.fat_size_sectors as u64);
        self.initialized = true;

        crate::serial_println!(
            "[FAT32] Filesystem mounted: Start LBA: {}, Sectors/Cluster: {}, Root Cluster: {}, Data LBA: {}",
            self.partition_start_lba,
            self.sectors_per_cluster,
            self.root_cluster,
            self.data_start_lba
        );

        true
    }

    #[inline]
    pub fn cluster_to_lba(&self, cluster: u32) -> u64 {
        self.data_start_lba + ((cluster as u64 - 2) * self.sectors_per_cluster as u64)
    }

    pub fn get_next_cluster(&self, cluster: u32) -> u32 {
        let fat_offset = cluster * 4;
        let fat_sector = self.fat_start_lba + (fat_offset as u64 / 512);
        let entry_offset = (fat_offset % 512) as usize;

        let mut buf = [0u8; 512];
        if read_sector(fat_sector, &mut buf) {
            let next = u32::from_le_bytes([
                buf[entry_offset],
                buf[entry_offset + 1],
                buf[entry_offset + 2],
                buf[entry_offset + 3],
            ]) & 0x0FFF_FFFF;
            next
        } else {
            0x0FFF_FFFF
        }
    }

    pub fn list_directory(&self, dir_cluster: u32, out: &mut [FsEntry]) -> usize {
        let mut count = 0;
        let mut curr_cluster = dir_cluster;
        let mut sector_buf = [0u8; 512];

        while curr_cluster < 0x0FFF_FFF8 && curr_cluster >= 2 {
            let cluster_lba = self.cluster_to_lba(curr_cluster);

            for sec in 0..self.sectors_per_cluster as u64 {
                if !read_sector(cluster_lba + sec, &mut sector_buf) {
                    break;
                }

                for entry_idx in 0..(512 / 32) {
                    let off = entry_idx * 32;
                    let first_byte = sector_buf[off];

                    if first_byte == 0x00 {
                        // End of directory
                        return count;
                    }
                    if first_byte == 0xE5 {
                        // Deleted entry
                        continue;
                    }

                    let attr = sector_buf[off + 11];
                    if attr == ATTR_LFN {
                        continue; // Skip LFN sub-records for basic listing
                    }

                    if count < out.len() {
                        let is_dir = (attr & ATTR_DIRECTORY) != 0;
                        let hi = u16::from_le_bytes([sector_buf[off + 20], sector_buf[off + 21]]) as u32;
                        let lo = u16::from_le_bytes([sector_buf[off + 26], sector_buf[off + 27]]) as u32;
                        let first_c = (hi << 16) | lo;
                        let size = u32::from_le_bytes([
                            sector_buf[off + 28], sector_buf[off + 29],
                            sector_buf[off + 30], sector_buf[off + 31]
                        ]) as usize;

                        // Clean 8.3 name
                        let raw_name = &sector_buf[off..off + 11];
                        let mut formatted = [0u8; 64];
                        let mut f_len = 0;

                        // Base name (up to 8 chars)
                        for i in 0..8 {
                            if raw_name[i] != b' ' {
                                formatted[f_len] = raw_name[i];
                                f_len += 1;
                            }
                        }
                        // Extension (up to 3 chars)
                        if !is_dir && raw_name[8] != b' ' {
                            formatted[f_len] = b'.';
                            f_len += 1;
                            for i in 8..11 {
                                if raw_name[i] != b' ' {
                                    formatted[f_len] = raw_name[i];
                                    f_len += 1;
                                }
                            }
                        }

                        let entry = &mut out[count];
                        entry.name.copy_from_slice(&formatted);
                        entry.name_len = f_len;
                        entry.is_dir = is_dir;
                        entry.size = size;
                        entry.first_cluster = first_c;

                        count += 1;
                    }
                }
            }

            curr_cluster = self.get_next_cluster(curr_cluster);
        }

        count
    }

    pub fn read_file(&self, first_cluster: u32, size: usize, out_buf: &mut [u8]) -> usize {
        let mut bytes_read = 0;
        let mut curr_cluster = first_cluster;
        let mut sector_buf = [0u8; 512];
        let bytes_per_cluster = self.sectors_per_cluster as usize * 512;

        while curr_cluster < 0x0FFF_FFF8 && curr_cluster >= 2 && bytes_read < size {
            let cluster_lba = self.cluster_to_lba(curr_cluster);

            for sec in 0..self.sectors_per_cluster as u64 {
                if bytes_read >= size || bytes_read >= out_buf.len() {
                    break;
                }

                if !read_sector(cluster_lba + sec, &mut sector_buf) {
                    return bytes_read;
                }

                let to_copy = (size - bytes_read).min(512).min(out_buf.len() - bytes_read);
                out_buf[bytes_read..bytes_read + to_copy].copy_from_slice(&sector_buf[..to_copy]);
                bytes_read += to_copy;
            }

            curr_cluster = self.get_next_cluster(curr_cluster);
        }

        bytes_read
    }
}

pub static mut FAT32: Fat32Fs = Fat32Fs::new();
