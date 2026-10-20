#!/usr/bin/env python3
"""
Veyron OS ISO Image Builder
Synthesizes a bootable ISO-9660 El Torito image with Limine UEFI and BIOS boot records.
"""

import os
import sys
import struct
import shutil

SECTOR_SIZE = 2048

def build_iso(root_dir: str, output_iso: str):
    print(f"Building bootable ISO: {output_iso}...")
    
    # In Limine, creating a hybrid ISO is achieved by packaging the boot files:
    # 1. System area (sectors 0..15: 32 KB)
    # 2. Primary Volume Descriptor (sector 16)
    # 3. Boot Record Volume Descriptor (sector 17)
    # 4. Volume Descriptor Set Terminator (sector 18)
    # 5. Boot catalog (sector 19)
    # 6. EFI boot image (limine-uefi-cd.bin)
    
    limine_bin_dir = os.path.join("boot", "limine", "limine-binary")
    uefi_cd_bin = os.path.join(limine_bin_dir, "limine-uefi-cd.bin")
    bios_cd_bin = os.path.join(limine_bin_dir, "limine-bios-cd.bin")
    
    iso_data = bytearray(64 * 1024 * 1024) # 64 MB ISO
    
    # Sector 16: Primary Volume Descriptor
    pvd = bytearray(SECTOR_SIZE)
    pvd[0] = 1 # Type: Primary Volume Descriptor
    pvd[1:6] = b"CD001"
    pvd[6] = 1 # Version
    pvd[40:72] = b"VEYRON_OS                       " # Volume ID (32 chars)
    # Volume space size: 64 MB / 2048 = 32768 sectors (both endian)
    total_iso_sectors = len(iso_data) // SECTOR_SIZE
    pvd[80:84] = struct.pack("<I", total_iso_sectors)
    pvd[84:88] = struct.pack(">I", total_iso_sectors)
    pvd[120:124] = struct.pack("<I", 1) # Volume set size
    pvd[124:128] = struct.pack(">I", 1)
    pvd[128:130] = struct.pack("<H", 1) # Volume sequence number
    pvd[130:132] = struct.pack(">H", 1)
    pvd[132:134] = struct.pack("<H", SECTOR_SIZE) # Logical block size
    pvd[134:136] = struct.pack(">H", SECTOR_SIZE)
    iso_data[16 * SECTOR_SIZE:17 * SECTOR_SIZE] = pvd
    
    # Sector 17: Boot Record Volume Descriptor (El Torito)
    brvd = bytearray(SECTOR_SIZE)
    brvd[0] = 0 # Boot Record
    brvd[1:6] = b"CD001"
    brvd[6] = 1
    brvd[7:39] = b"EL TORITO SPECIFICATION\0\0\0\0\0\0\0\0\0"
    brvd[71:75] = struct.pack("<I", 19) # Boot catalog sector pointer (Sector 19)
    iso_data[17 * SECTOR_SIZE:18 * SECTOR_SIZE] = brvd
    
    # Sector 18: Volume Descriptor Set Terminator
    vdt = bytearray(SECTOR_SIZE)
    vdt[0] = 255
    vdt[1:6] = b"CD001"
    vdt[6] = 1
    iso_data[18 * SECTOR_SIZE:19 * SECTOR_SIZE] = vdt
    
    # Sector 19: Boot Catalog
    cat = bytearray(SECTOR_SIZE)
    # Validation Entry
    cat[0] = 1
    cat[1] = 0 # x86
    cat[28:30] = b"\x55\xAA"
    cat[30] = 0x55
    cat[31] = 0xAA
    
    # Initial / Default Entry (BIOS Boot)
    cat[32] = 0x88 # Bootable
    cat[33] = 0    # No emulation
    cat[34:36] = struct.pack("<H", 0) # Load segment
    cat[36] = 0    # System type: x86
    cat[38:40] = struct.pack("<H", 4) # 4 sectors (2048 * 4)
    cat[40:44] = struct.pack("<I", 20) # Sector 20
    
    # Section Header (EFI Boot)
    cat[64] = 0x91 # Section Header, Last entry
    cat[65] = 0xEF # EFI platform
    cat[66:68] = struct.pack("<H", 1) # 1 entry in section
    
    # Section Entry (EFI Image)
    cat[96] = 0x88 # Bootable
    cat[97] = 0    # No emulation
    cat[102:104] = struct.pack("<H", 1)
    
    # Write BIOS CD bin at Sector 20
    if os.path.exists(bios_cd_bin):
        with open(bios_cd_bin, "rb") as f:
            b_data = f.read()
            iso_data[20 * SECTOR_SIZE:20 * SECTOR_SIZE + len(b_data)] = b_data
            
    # Write UEFI CD bin starting at Sector 64
    efi_lba = 64
    cat[104:108] = struct.pack("<I", efi_lba)
    if os.path.exists(uefi_cd_bin):
        with open(uefi_cd_bin, "rb") as f:
            u_data = f.read()
            iso_data[efi_lba * SECTOR_SIZE:efi_lba * SECTOR_SIZE + len(u_data)] = u_data
            
    iso_data[19 * SECTOR_SIZE:20 * SECTOR_SIZE] = cat
    
    with open(output_iso, "wb") as f:
        f.write(iso_data)
        
    print(f"Wrote {len(iso_data)} bytes to {output_iso}.")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python mkiso.py <root_dir> <output.iso>")
        sys.exit(1)
    build_iso(sys.argv[1], sys.argv[2])
