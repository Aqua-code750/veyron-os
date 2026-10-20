#!/usr/bin/env python3
"""
Veyron OS Disk Image Builder
Creates a bootable GPT/MBR disk image with a FAT32 partition.
Supports subdirectories, long filenames (LFN), and installs Limine bootloader.
"""

import os
import sys
import struct
import math
import subprocess

SECTOR_SIZE = 512
DISK_SIZE_MB = 128
TOTAL_SECTORS = (DISK_SIZE_MB * 1024 * 1024) // SECTOR_SIZE
PARTITION_START_LBA = 2048  # 1 MB offset
PARTITION_SECTORS = TOTAL_SECTORS - PARTITION_START_LBA
SECTORS_PER_CLUSTER = 1     # 512-byte clusters: ensures > 65525 clusters for FAT32 spec compliance
CLUSTER_SIZE = SECTORS_PER_CLUSTER * SECTOR_SIZE
RESERVED_SECTORS = 32
NUM_FATS = 2

class FAT32Builder:
    def __init__(self, partition_sectors):
        self.total_sectors = partition_sectors
        self.sectors_per_cluster = SECTORS_PER_CLUSTER
        self.cluster_size = CLUSTER_SIZE
        self.reserved_sectors = RESERVED_SECTORS
        self.num_fats = NUM_FATS
        
        # Calculate FAT size
        # data sectors = total_sectors - reserved_sectors - num_fats * fat_size
        # clusters = data_sectors // sectors_per_cluster
        # fat_size = ceil((clusters + 2) * 4 / 512)
        # Solve conservatively:
        raw_clusters = self.total_sectors // self.sectors_per_cluster
        self.fat_size = (raw_clusters * 4 + SECTOR_SIZE - 1) // SECTOR_SIZE
        
        self.root_cluster = 2
        self.fat_entries = [0x0FFFFF8, 0x0FFFFFFF, 0x0FFFFFFF] # 0, 1, and root cluster 2 (end of chain)
        self.clusters = {} # cluster_index -> bytearray of cluster_size
        self.clusters[2] = bytearray(self.cluster_size) # root dir cluster
        
    def alloc_cluster(self):
        idx = len(self.fat_entries)
        self.fat_entries.append(0x0FFFFFFF) # EOC
        self.clusters[idx] = bytearray(self.cluster_size)
        return idx

    def _lfn_checksum(self, short_name: bytes) -> int:
        c = 0
        for b in short_name:
            c = (((c & 1) << 7) + (c >> 1) + b) & 0xFF
        return c

    def _make_short_name(self, filename: str, is_dir: bool, existing_names: set) -> bytes:
        filename = filename.upper()
        parts = filename.split('.')
        base = parts[0]
        ext = parts[1] if len(parts) > 1 else ""
        
        # Clean chars
        clean_base = "".join(c for c in base if c.isalnum() or c in "_-~")
        clean_ext = "".join(c for c in ext if c.isalnum() or c in "_-~")
        
        if len(parts) == 1 and len(base) <= 8 and clean_base == base:
            s = f"{clean_base:<8}{'':<3}".encode('ascii')
            if s not in existing_names:
                return s
        elif len(parts) == 2 and len(base) <= 8 and len(ext) <= 3 and clean_base == base and clean_ext == ext:
            s = f"{clean_base:<8}{clean_ext:<3}".encode('ascii')
            if s not in existing_names:
                return s
                
        # Needs ~1 mangling
        c = 1
        while True:
            suffix = f"~{c}"
            trimmed_base = clean_base[:8 - len(suffix)]
            cand = f"{trimmed_base}{suffix:<{8-len(trimmed_base)}}{clean_ext[:3]:<3}".encode('ascii')
            if cand not in existing_names:
                return cand
            c += 1

    def _build_dir_entries(self, name: str, is_dir: bool, start_cluster: int, size: int, existing_names: set) -> list:
        short_name = self._make_short_name(name, is_dir, existing_names)
        existing_names.add(short_name)
        
        entries = []
        chk = self._lfn_checksum(short_name)
        
        # Build LFN entries if name doesn't match short name exactly
        parts = name.split('.')
        is_exact = (len(parts) == 1 and len(parts[0]) <= 8 and name.isupper()) or \
                   (len(parts) == 2 and len(parts[0]) <= 8 and len(parts[1]) <= 3 and name.isupper())
        
        if not is_exact:
            chars = list(name)
            chunks = []
            while chars:
                chunks.append(chars[:13])
                chars = chars[13:]
            
            for i, chunk in enumerate(chunks):
                seq = i + 1
                if i == len(chunks) - 1:
                    seq |= 0x40 # Last LFN entry marker
                
                # Format chunk: 5 chars, 6 chars, 2 chars
                uchars = [ord(c) for c in chunk]
                while len(uchars) < 13:
                    if len(uchars) == len(chunk):
                        uchars.append(0)
                    else:
                        uchars.append(0xFFFF)
                        
                part1 = struct.pack("<5H", *uchars[0:5])
                part2 = struct.pack("<6H", *uchars[5:11])
                part3 = struct.pack("<2H", *uchars[11:13])
                
                lfn_entry = struct.pack("<BBBB B", seq, 0x0F, 0, chk, 0) # seq, attr=0x0F, type=0, chk, zero
                # Reconstruct full 32-byte LFN:
                # 0: seq, 1-10: name1 (5 utf16), 11: attr(0x0f), 12: type(0), 13: chk, 14-25: name2(6 utf16), 26-27: cluster(0), 28-31: name3(2 utf16)
                lfn_record = bytearray(32)
                lfn_record[0] = seq
                lfn_record[1:11] = part1
                lfn_record[11] = 0x0F
                lfn_record[12] = 0x00
                lfn_record[13] = chk
                lfn_record[14:26] = part2
                lfn_record[26:28] = b'\x00\x00'
                lfn_record[28:32] = part3
                entries.append(bytes(lfn_record))
                
            entries.reverse() # LFN entries appear in reverse sequence order
            
        # Standard 32-byte directory entry
        attr = 0x10 if is_dir else 0x20
        hi_cluster = (start_cluster >> 16) & 0xFFFF
        lo_cluster = start_cluster & 0xFFFF
        
        rec = bytearray(32)
        rec[0:11] = short_name
        rec[11] = attr
        rec[20:22] = struct.pack("<H", hi_cluster)
        rec[26:28] = struct.pack("<H", lo_cluster)
        rec[28:32] = struct.pack("<I", size)
        entries.append(bytes(rec))
        return entries

    def add_file_data(self, data: bytes) -> tuple:
        if len(data) == 0:
            return (0, 0)
        
        num_clusters = (len(data) + self.cluster_size - 1) // self.cluster_size
        clusters = [self.alloc_cluster() for _ in range(num_clusters)]
        for i in range(num_clusters - 1):
            self.fat_entries[clusters[i]] = clusters[i + 1]
        self.fat_entries[clusters[-1]] = 0x0FFFFFFF
        
        offset = 0
        for c in clusters:
            chunk = data[offset:offset + self.cluster_size]
            self.clusters[c][:len(chunk)] = chunk
            offset += len(chunk)
            
        return (clusters[0], len(data))

    def write_dir(self, parent_cluster: int, name: str) -> int:
        dir_cluster = self.alloc_cluster()
        # Initialize . and ..
        rec_dot = bytearray(32)
        rec_dot[0:11] = b".          "
        rec_dot[11] = 0x10
        rec_dot[20:22] = struct.pack("<H", (dir_cluster >> 16) & 0xFFFF)
        rec_dot[26:28] = struct.pack("<H", dir_cluster & 0xFFFF)
        
        rec_dotdot = bytearray(32)
        rec_dotdot[0:11] = b"..         "
        rec_dotdot[11] = 0x10
        p_clust = parent_cluster if parent_cluster != self.root_cluster else 0
        rec_dotdot[20:22] = struct.pack("<H", (p_clust >> 16) & 0xFFFF)
        rec_dotdot[26:28] = struct.pack("<H", p_clust & 0xFFFF)
        
        self.clusters[dir_cluster][0:32] = rec_dot
        self.clusters[dir_cluster][32:64] = rec_dotdot
        return dir_cluster

    def append_to_directory(self, dir_cluster: int, entry_bytes: bytes):
        cluster = dir_cluster
        while True:
            buf = self.clusters[cluster]
            # Find free 32-byte slot
            for off in range(0, len(buf), 32):
                if buf[off] == 0x00 or buf[off] == 0xE5:
                    if off + len(entry_bytes) <= len(buf):
                        buf[off:off + len(entry_bytes)] = entry_bytes
                        return
            # Need to follow FAT or allocate next cluster
            next_c = self.fat_entries[cluster]
            if next_c >= 0x0FFFFFF8:
                new_c = self.alloc_cluster()
                self.fat_entries[cluster] = new_c
                self.clusters[new_c][:len(entry_bytes)] = entry_bytes
                return
            cluster = next_c

    def populate_from_dir(self, host_dir: str, target_dir_cluster: int = 2):
        existing_names = set()
        items = sorted(os.listdir(host_dir))
        for item in items:
            full_path = os.path.join(host_dir, item)
            if os.path.isdir(full_path):
                child_cluster = self.write_dir(target_dir_cluster, item)
                entries = self._build_dir_entries(item, True, child_cluster, 0, existing_names)
                for e in entries:
                    self.append_to_directory(target_dir_cluster, e)
                self.populate_from_dir(full_path, child_cluster)
            else:
                with open(full_path, "rb") as f:
                    content = f.read()
                start_c, sz = self.add_file_data(content)
                entries = self._build_dir_entries(item, False, start_c, sz, existing_names)
                for e in entries:
                    self.append_to_directory(target_dir_cluster, e)

    def serialize_partition(self) -> bytes:
        part = bytearray(self.total_sectors * SECTOR_SIZE)
        
        # 1. Boot sector (BPB)
        bpb = bytearray(SECTOR_SIZE)
        bpb[0:3] = b"\xEB\x58\x90" # jmp short + nop
        bpb[3:11] = b"MSWIN4.1"   # OEM name
        bpb[11:13] = struct.pack("<H", SECTOR_SIZE)
        bpb[13] = self.sectors_per_cluster
        bpb[14:16] = struct.pack("<H", self.reserved_sectors)
        bpb[16] = self.num_fats
        bpb[17:19] = b"\x00\x00"  # Root entries (0 for FAT32)
        bpb[19:21] = b"\x00\x00"  # Small sectors count (0)
        bpb[21] = 0xF8            # Media descriptor (fixed disk)
        bpb[22:24] = b"\x00\x00"  # Sectors per FAT (0 for FAT32)
        bpb[24:26] = struct.pack("<H", 63) # Sectors per track
        bpb[26:28] = struct.pack("<H", 255) # Heads
        bpb[28:32] = struct.pack("<I", PARTITION_START_LBA) # Hidden sectors
        bpb[32:36] = struct.pack("<I", self.total_sectors)  # Large sectors count
        
        # FAT32 extended BPB
        bpb[36:40] = struct.pack("<I", self.fat_size)
        bpb[40:42] = b"\x00\x00"  # Ext flags
        bpb[42:44] = b"\x00\x00"  # FS version 0:0
        bpb[44:48] = struct.pack("<I", self.root_cluster)
        bpb[48:50] = struct.pack("<H", 1) # FSInfo sector
        bpb[50:52] = struct.pack("<H", 6) # Backup boot sector
        bpb[64] = 0x80            # Drive number
        bpb[66] = 0x29            # Boot signature
        bpb[67:71] = b"\x12\x34\x56\x78" # Volume ID
        bpb[71:82] = b"VEYRON OS  " # Volume label
        bpb[82:90] = b"FAT32   " # System ID
        bpb[510:512] = b"\x55\xAA" # Boot signature
        part[0:512] = bpb
        part[6 * SECTOR_SIZE:7 * SECTOR_SIZE] = bpb # Backup BPB
        
        # 2. FSInfo sector
        fsinfo = bytearray(SECTOR_SIZE)
        fsinfo[0:4] = b"RRaA"
        fsinfo[484:488] = b"rrAa"
        fsinfo[488:492] = struct.pack("<I", 0xFFFFFFFF) # Free cluster count
        fsinfo[492:496] = struct.pack("<I", len(self.fat_entries) - 1) # Next free cluster
        fsinfo[510:512] = b"\x55\xAA"
        part[1 * SECTOR_SIZE:2 * SECTOR_SIZE] = fsinfo
        part[7 * SECTOR_SIZE:8 * SECTOR_SIZE] = fsinfo
        
        # 3. FAT tables
        fat_bytes = bytearray(self.fat_size * SECTOR_SIZE)
        for i, val in enumerate(self.fat_entries):
            off = i * 4
            fat_bytes[off:off+4] = struct.pack("<I", val & 0x0FFFFFFF)
            
        fat1_offset = self.reserved_sectors * SECTOR_SIZE
        fat2_offset = (self.reserved_sectors + self.fat_size) * SECTOR_SIZE
        part[fat1_offset:fat1_offset + len(fat_bytes)] = fat_bytes
        part[fat2_offset:fat2_offset + len(fat_bytes)] = fat_bytes
        
        # 4. Data clusters
        data_start_offset = (self.reserved_sectors + self.num_fats * self.fat_size) * SECTOR_SIZE
        for c_idx, c_data in self.clusters.items():
            c_offset = data_start_offset + (c_idx - 2) * self.cluster_size
            if c_offset + len(c_data) <= len(part):
                part[c_offset:c_offset + len(c_data)] = c_data
                
        return bytes(part)


def build_disk_image(root_files_dir: str, output_img: str):
    print(f"Building {output_img} from {root_files_dir}...")
    builder = FAT32Builder(PARTITION_SECTORS)
    builder.populate_from_dir(root_files_dir)
    partition_data = builder.serialize_partition()
    
    # Create complete disk with MBR
    disk = bytearray(TOTAL_SECTORS * SECTOR_SIZE)
    
    # MBR Partition Entry 1 at offset 446 (0x1BE)
    # Status: 0x80 (bootable)
    # Type: 0x0C (FAT32 LBA)
    # First sector LBA: PARTITION_START_LBA
    # Sector count: PARTITION_SECTORS
    mbr_entry = bytearray(16)
    mbr_entry[0] = 0x80 # Bootable
    mbr_entry[1] = 0x20 # CHS start
    mbr_entry[2] = 0x21
    mbr_entry[3] = 0x00
    mbr_entry[4] = 0xEF # EFI System Partition (ESP)
    mbr_entry[5] = 0xFE # CHS end
    mbr_entry[6] = 0xC0
    mbr_entry[7] = 0xFE
    mbr_entry[8:12] = struct.pack("<I", PARTITION_START_LBA)
    mbr_entry[12:16] = struct.pack("<I", PARTITION_SECTORS)
    
    disk[446:462] = mbr_entry
    disk[510:512] = b"\x55\xAA" # MBR signature
    
    # Place partition at PARTITION_START_LBA
    part_offset = PARTITION_START_LBA * SECTOR_SIZE
    disk[part_offset:part_offset + len(partition_data)] = partition_data
    
    with open(output_img, "wb") as f:
        f.write(disk)
        
    print(f"Wrote {len(disk)} bytes to {output_img}.")
    
    # Install Limine BIOS bootloader if limine tool exists
    limine_exe = os.path.join("boot", "limine", "limine-binary", "limine-tool-windows-x86", "limine.exe")
    if os.path.exists(limine_exe):
        try:
            res = subprocess.run([limine_exe, "bios-install", output_img], capture_output=True, text=True)
            print("Limine bios-install output:", res.stdout.strip() or res.stderr.strip())
        except Exception as e:
            print("Notice: limine bios-install warning:", e)


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python mkdisk.py <staging_dir> <output.img>")
        sys.exit(1)
    build_disk_image(sys.argv[1], sys.argv[2])
