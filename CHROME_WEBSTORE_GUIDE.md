# 🛒 How to Publish Veyron OS to the Chrome Web Store

Your extension is packaged and ready to submit to the official **Google Chrome Web Store**.

---

### Prerequisites
* A Google Account.
* A one-time **\$5 USD Developer Registration fee** paid to Google (if you haven't published extensions before).

---

### Step 1: Access the Chrome Developer Dashboard
1. Go to the [Chrome Web Store Developer Dashboard](https://chrome.google.com/webstore/devconsole).
2. Sign in with your Google account.
3. If this is your first extension, complete the registration and accept the developer agreement.

---

### Step 2: Upload the Extension Package
1. In the Developer Dashboard, click the **"Add new item"** or **"New Item"** button.
2. Drag and drop or browse to select the packaged file:
   ```text
   veyron-os-extension.zip
   ```
   *(Located in your project root directory)*
3. Google will unpack and validate your `manifest.json`.

---

### Step 3: Complete Store Listing Information

Fill out the listing details:
* **Description**:
  ```text
  Veyron OS is a lightweight, modern Windows 11-style desktop operating system running directly inside your browser.

  Features:
  - Browser Inside a Browser: Browse websites in an embedded desktop window.
  - Full Window Manager: Draggable, resizable, minimizable windows with smooth animations.
  - Complete App Suite: File Explorer, Notepad, Paint Studio, Terminal (CMD), Calculator, Task Manager, and DOOM 3D.
  - Windows 11 Fluent Design: Mica glassmorphism, authentic Bloom wallpaper, and centered taskbar.
  - Built-in Sound System: Procedural Web Audio startup chime and click effects.
  - High-resolution "Fancy V" branding and custom icons.
  ```
* **Category**: `Productivity` or `Fun`.
* **Language**: `English`.

---

### Step 4: Graphic Assets
Upload the following required visuals:
* **Store Icon**: Upload `icons/icon128.png` (128x128).
* **Screenshots**: Take 1-2 screenshots of Veyron OS in your browser (1280x800 or 640x400) showing the desktop with open windows (e.g. Browser showing Wikipedia, Terminal, and Notepad).
* **Small Promo Tile (Optional)**: 440x280 png.

---

### Step 5: Privacy Practices
1. In the **Privacy tab**:
   * Single purpose description: `Provides a modern Windows 11-style desktop experience directly in your browser's new tab.`
   * Permission justification:
     * `storage`: Used to save desktop notes, notepad files, and user preferences locally on the user's machine.
     * `tabs`: Used to launch Veyron OS in a new tab when the toolbar button is clicked.
   * Data usage declaration: Check that **no user data is collected or transmitted to external servers**.

---

### Step 6: Submit for Review
1. Click **"Submit for review"**.
2. Google typically reviews new extensions within **24 to 48 hours**.
3. Once approved, your extension will be live for millions of users worldwide!
