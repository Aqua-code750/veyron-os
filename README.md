# 🏎️ Veyron OS

<p align="center">
  <img src="icons/icon128.png" width="128" height="128" alt="Veyron OS Logo" />
</p>

<p align="center">
  <b>A Modern, Windows 11-style Desktop Operating System Running Directly in Your Browser & as a Bare-Metal Kernel from Scratch.</b>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Platform-Chrome%20Extension%20%7C%20Web%20Desktop-00F0FF?style=for-the-badge&logo=googlechrome&logoColor=white" alt="Platform" />
  <img src="https://img.shields.io/badge/Kernel-Rust%20(no__std)-orange?style=for-the-badge&logo=rust&logoColor=white" alt="Rust Kernel" />
  <img src="https://img.shields.io/badge/Design-Windows%2011%20Fluent-0078D4?style=for-the-badge&logo=windows11&logoColor=white" alt="Windows 11" />
  <img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="License" />
</p>

---

## 🌟 Highlights

* **Browser Inside a Browser**: Browse real websites (Wikipedia, search engines, docs) inside an embedded browser window inside your Google Chrome desktop!
* **Google Chrome Extension (Manifest V3)**: Replaces your New Tab (`Ctrl + T`) or launches with a single click from the Chrome toolbar.
* **The "Fancy V" Identity**: Futuristic aerodynamic glowing cyan/azure emblem with 3D bevels, specular reflections, and animated boot aura.
* **Windows 11 Fluent Interface**: Real-time Mica and Acrylic glassmorphism, rounded window corners, soft shadows, and centered taskbar with active glow indicators.
* **Synthesized Audio Engine**: Zero-asset procedural audio synthesizer using Web Audio API for startup chimes and clicks.
* **From-Scratch Rust Kernel**: Includes a bare-metal `x86_64-unknown-none` kernel with UEFI Limine bootloader, 64-bit GDT/IDT, PMM, VMM, AHCI SATA driver, FAT32 filesystem, and native window compositor.

---

## 📱 Native Applications Included

| Application | Features |
| :--- | :--- |
| **🌐 Web Browser** | Full web browsing with URL bar, bookmarks, tabs, navigation history, and embedded site rendering. |
| **📁 File Explorer** | Windows 11 ribbon bar (`+ New`, `Cut`, `Copy`, `Delete`), Quick Access sidebar, and directory navigation. |
| **📝 Notepad** | Multi-line text editor with status bar (`Ln 1, Col 1`, `UTF-8`, `Windows CRLF`), word wrap, and file save. |
| **🎨 Paint Studio** | Full raster drawing suite with brush, pencil, eraser, shapes (lines, rectangles, circles), and palette swatches. |
| **💻 Terminal (CMD)** | Shell with command history, prompt navigation (`dir`, `help`, `cd`, `neofetch`, `matrix`, `ver`, `start <app>`). |
| **🔢 Calculator** | Windows 11-style calculator with memory, operator chaining, and clear buttons. |
| **📊 Task Manager** | Real-time animated CPU heartbeat performance waveform and running process list. |
| **🕹️ DOOM 3D** | Real-time 3D raycasting engine with textured walls, weapon animations, minimap HUD, and sound effects. |
| **⚙️ Settings** | Windows 11 settings cards: personalization (dark/light themes, wallpapers), system hardware info, and reset. |

---

## 🚀 Installation & Usage

### Option 1: Install as a Google Chrome Extension
1. Open Google Chrome and navigate to:
   ```text
   chrome://extensions
   ```
2. Enable **Developer mode** (toggle in the top-right corner).
3. Click **Load unpacked** in the top-left corner.
4. Select the repository root folder.
5. Open a **New Tab** (`Ctrl + T`) to launch Veyron OS!

### Option 2: Run Directly in Any Web Browser
Simply double-click `run_web.bat` or open `index.html` in Chrome, Edge, Firefox, Brave, or Safari.

### Option 3: Chrome Web Store Package
The release ZIP package is ready at:
```text
veyron-os-extension.zip
```
Follow the [Chrome Web Store Publishing Guide](CHROME_WEBSTORE_GUIDE.md) to upload to the Developer Dashboard.

---

## 📂 Project Structure

```text
├── manifest.json              # Chrome Extension Manifest V3 configuration
├── background.js             # Background service worker for extension actions
├── index.html                # Veyron OS Windows 11 Fluent web desktop
├── app.js                    # Core desktop window manager, filesystem & apps
├── icons/                    # Multi-resolution Fancy V extension icons
│   ├── icon16.png
│   ├── icon32.png
│   ├── icon48.png
│   ├── icon128.png
│   └── icon256.png
├── kernel/                   # From-scratch Rust x86_64 OS kernel
│   ├── src/
│   │   ├── main.rs           # Kernel entry point
│   │   ├── arch/x86_64/      # GDT, IDT, PIC, Syscall interrupts
│   │   ├── mm/               # PMM frame allocator, VMM paging, Heap
│   │   ├── drivers/          # AHCI SATA disk, PIT timer, PS/2 mouse/keyboard
│   │   ├── fs/               # FAT32 driver & VFS layer
│   │   ├── gui/              # Framebuffer compositor & Bloom wallpaper
│   │   └── net/              # Native TCP/IP stack
├── tools/                    # Packaging, disk creation & icon generation scripts
└── veyron-os-extension.zip   # Production Chrome Web Store upload zip
```

---

## 📜 License

Distributed under the **MIT License**. See `LICENSE` for more information.
