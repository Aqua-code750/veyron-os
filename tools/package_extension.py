#!/usr/bin/env python3
"""
Packages Veyron OS Chrome Extension into a clean zip file ready for Chrome Web Store upload.
"""

import os
import zipfile

def package():
    zip_filename = "veyron-os-extension.zip"
    if os.path.exists(zip_filename):
        os.remove(zip_filename)

    # Core extension files to include
    include_files = [
        "manifest.json",
        "index.html",
        "app.js",
        "background.js"
    ]
    
    with zipfile.ZipFile(zip_filename, "w", zipfile.ZIP_DEFLATED) as zf:
        for f in include_files:
            if os.path.exists(f):
                zf.write(f, arcname=f)
                print(f"Added {f}")
        
        # Add icons folder
        icons_dir = "icons"
        if os.path.exists(icons_dir):
            for icon in os.listdir(icons_dir):
                full_path = os.path.join(icons_dir, icon)
                if os.path.isfile(full_path):
                    arc_name = f"icons/{icon}"
                    zf.write(full_path, arcname=arc_name)
                    print(f"Added {arc_name}")

    print(f"\n[SUCCESS] Packaged Chrome Web Store release: {zip_filename} ({os.path.getsize(zip_filename)} bytes)")

if __name__ == "__main__":
    package()
