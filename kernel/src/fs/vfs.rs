//! Virtual File System (VFS) Layer
//! Path resolution and abstraction across system volumes and user directories.

use super::fat32::{FAT32, FsEntry};

pub fn init() -> bool {
    unsafe {
        if FAT32.init() {
            crate::serial_println!("[VFS] VFS root mounted to FAT32 filesystem.");
            true
        } else {
            crate::serial_println!("[VFS] WARNING: Failed to mount root filesystem!");
            false
        }
    }
}

pub fn list_dir(path: &str, out: &mut [FsEntry]) -> usize {
    unsafe {
        if !FAT32.initialized {
            return 0;
        }

        let clean_path = path.trim_matches('/');
        if clean_path.is_empty() {
            // Root directory
            return FAT32.list_directory(FAT32.root_cluster, out);
        }

        // Traverse subdirectories
        let mut curr_cluster = FAT32.root_cluster;
        let mut temp_entries: [FsEntry; 64] = [const { FsEntry::empty() }; 64];

        for part in clean_path.split('/') {
            let count = FAT32.list_directory(curr_cluster, &mut temp_entries);
            let mut found = false;

            for i in 0..count {
                if temp_entries[i].is_dir && temp_entries[i].name_as_str().eq_ignore_ascii_case(part) {
                    curr_cluster = temp_entries[i].first_cluster;
                    found = true;
                    break;
                }
            }

            if !found {
                return 0; // directory not found
            }
        }

        FAT32.list_directory(curr_cluster, out)
    }
}

pub fn read_file(path: &str, out_buf: &mut [u8]) -> usize {
    unsafe {
        if !FAT32.initialized {
            return 0;
        }

        let clean_path = path.trim_matches('/');
        let (dir_path, filename) = match clean_path.rfind('/') {
            Some(idx) => (&clean_path[..idx], &clean_path[idx + 1..]),
            None => ("", clean_path),
        };

        let mut entries: [FsEntry; 64] = [const { FsEntry::empty() }; 64];
        let count = list_dir(dir_path, &mut entries);

        for i in 0..count {
            if !entries[i].is_dir && entries[i].name_as_str().eq_ignore_ascii_case(filename) {
                return FAT32.read_file(entries[i].first_cluster, entries[i].size, out_buf);
            }
        }

        0
    }
}
