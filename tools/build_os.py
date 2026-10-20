#!/usr/bin/env python3
"""
Veyron OS Build Orchestrator
Builds kernel, stages disk files, creates FAT32 image with Limine bootloader.
"""

import os
import shutil
import subprocess
import sys

def run_command(cmd, cwd=None):
    print(f"[BUILD] Running: {' '.join(cmd)}")
    res = subprocess.run(cmd, cwd=cwd)
    if res.returncode != 0:
        print(f"[ERROR] Command failed with exit code {res.returncode}")
        sys.exit(res.returncode)

def main():
    root_dir = os.path.abspath(os.path.dirname(os.path.dirname(__file__)))
    os.chdir(root_dir)

    print("==================================================")
    print("           BUILDING VEYRON OS 2026                ")
    print("==================================================")

    # 1. Compile Kernel
    run_command(["cargo", "build"])

    kernel_bin = os.path.join(root_dir, "target", "x86_64-unknown-none", "debug", "veyron_kernel")
    if not os.path.exists(kernel_bin):
        print(f"[ERROR] Kernel binary not found at {kernel_bin}")
        sys.exit(1)

    # 2. Prepare staging directory
    staging = os.path.join(root_dir, "build", "root")
    if os.path.exists(staging):
        shutil.rmtree(staging)

    os.makedirs(os.path.join(staging, "EFI", "BOOT"), exist_ok=True)
    os.makedirs(os.path.join(staging, "boot"), exist_ok=True)
    os.makedirs(os.path.join(staging, "System"), exist_ok=True)
    os.makedirs(os.path.join(staging, "Users", "Default", "Desktop"), exist_ok=True)
    os.makedirs(os.path.join(staging, "Users", "Default", "Documents"), exist_ok=True)
    os.makedirs(os.path.join(staging, "Users", "Default", "Downloads"), exist_ok=True)
    os.makedirs(os.path.join(staging, "Users", "Default", "Pictures"), exist_ok=True)
    os.makedirs(os.path.join(staging, "Apps"), exist_ok=True)

    # 3. Copy Limine EFI bootloader and BIOS assets
    limine_bin_dir = os.path.join(root_dir, "boot", "limine", "limine-binary")
    shutil.copy2(os.path.join(limine_bin_dir, "BOOTX64.EFI"), os.path.join(staging, "EFI", "BOOT", "BOOTX64.EFI"))
    shutil.copy2(os.path.join(limine_bin_dir, "limine-bios.sys"), os.path.join(staging, "boot", "limine-bios.sys"))

    # 4. Copy Kernel
    shutil.copy2(kernel_bin, os.path.join(staging, "boot", "veyron_kernel"))

    # 5. Write limine.conf
    limine_conf_content = """timeout: 0
/Veyron OS
    protocol: limine
    resolution: 1280x800x32
    kernel_path: boot():/boot/veyron_kernel
"""
    with open(os.path.join(staging, "limine.conf"), "w", encoding="utf-8") as f:
        f.write(limine_conf_content)
    with open(os.path.join(staging, "boot", "limine.conf"), "w", encoding="utf-8") as f:
        f.write(limine_conf_content)

    # 6. Add initial welcome file
    welcome_txt = """Welcome to Veyron OS!
A modern, lightweight Windows 11-style desktop operating system built FROM SCRATCH.
"""
    with open(os.path.join(staging, "Users", "Default", "Desktop", "Welcome.txt"), "w", encoding="utf-8") as f:
        f.write(welcome_txt)

    # 7. Create Disk Image (veyron.img)
    out_img = os.path.join(root_dir, "veyron.img")
    run_command([sys.executable, os.path.join(root_dir, "tools", "mkdisk.py"), staging, out_img])

    # 8. Create Bootable ISO Image (veyron.iso)
    out_iso = os.path.join(root_dir, "veyron.iso")
    run_command([sys.executable, os.path.join(root_dir, "tools", "mkiso.py"), staging, out_iso])

    print("\n[SUCCESS] Veyron OS images generated:")
    print(f"  -> Disk Image: {out_img}")
    print(f"  -> Bootable ISO: {out_iso}")

if __name__ == "__main__":
    main()
